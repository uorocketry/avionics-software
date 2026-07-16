// src/routes/logs/+server.ts
import { error } from '@sveltejs/kit';
import http from 'node:http';

const DOCKER_SOCKET = '/var/run/docker.sock';

function dockerRequestStream(path: string): Promise<{ status: number; stream: NodeJS.ReadableStream; ok: boolean; }> {
    return new Promise((resolve, reject) => {
        const req = http.request({
            socketPath: DOCKER_SOCKET,
            path,
            method: 'GET'
        }, (res) => {
            const status = res.statusCode ?? 500;
            resolve({
                status,
                ok: status >= 200 && status < 300,
                stream: res
            });
        });
        req.on('error', reject);
        req.end();
    });
}

async function findContainer(containerName: string): Promise<string> {
    // First, get container info to find the container ID
    const containersRes = await new Promise<{ status: number; text: () => Promise<string>; ok: boolean; }>((resolve, reject) => {
        const req = http.request({
            socketPath: DOCKER_SOCKET,
            path: '/containers/json?all=true',
            method: 'GET'
        }, (res) => {
            const chunks: Buffer[] = [];
            res.on('data', (c) => chunks.push(Buffer.isBuffer(c) ? c : Buffer.from(c)));
            res.on('end', () => {
                const buf = Buffer.concat(chunks);
                const status = res.statusCode ?? 500;
                resolve({
                    status,
                    ok: status >= 200 && status < 300,
                    text: async () => buf.toString('utf8')
                });
            });
        });
        req.on('error', reject);
        req.end();
    });

    if (!containersRes.ok) {
        throw new Error(`Failed to list containers: ${containersRes.status}`);
    }

    const containers = JSON.parse(await containersRes.text());
    const container = containers.find((c: any) =>
        c.Names?.some((name: string) => name.includes(containerName)) ||
        c.Labels?.['com.docker.compose.service'] === containerName
    );

    if (!container) {
        throw new Error(`Container not found: ${containerName}`);
    }

    return container.Id;
}

export async function GET({ url }) {
    const service = url.searchParams.get('service');
    const tail = url.searchParams.get('tail') || '50';
    const grep = url.searchParams.get('grep');

    if (!service) {
        throw error(400, 'service parameter is required');
    }

    // Set up Server-Sent Events with true real-time streaming
    const stream = new ReadableStream({
        start(controller) {
            let isControllerClosed = false;
            let dockerStream: any = null;

            // Safe function to enqueue data
            const safeEnqueue = (data: string) => {
                if (!isControllerClosed && controller.desiredSize !== null) {
                    try {
                        controller.enqueue(new TextEncoder().encode(data));
                    } catch (e) {
                        console.error('Error enqueueing data:', e);
                        isControllerClosed = true;
                    }
                }
            };

            // Safe function to close controller
            const safeClose = () => {
                if (!isControllerClosed) {
                    try {
                        isControllerClosed = true;
                        controller.close();
                    } catch (e) {
                        console.error('Error closing controller:', e);
                    }
                }
            };

            // Set a timeout to prevent hanging streams
            const timeoutId = setTimeout(() => {
                if (!isControllerClosed) {
                    console.log('Log stream timeout reached, closing');
                    safeEnqueue(`data: ${JSON.stringify({ type: 'error', line: 'Stream timeout reached' })}\n\n`);
                    safeClose();
                }
            }, 300000); // 5 minutes timeout

            // Send heartbeat every 30 seconds to keep connection alive
            const heartbeatId = setInterval(() => {
                if (!isControllerClosed) {
                    safeEnqueue(`data: ${JSON.stringify({ type: 'heartbeat', timestamp: Date.now() })}\n\n`);
                }
            }, 30000);

            // Cleanup function
            const cleanup = () => {
                // Clear timeout
                if (timeoutId) {
                    clearTimeout(timeoutId);
                }

                // Clear heartbeat
                if (heartbeatId) {
                    clearInterval(heartbeatId);
                }

                // Close Docker stream if open
                if (dockerStream && !dockerStream.destroyed) {
                    try {
                        dockerStream.destroy();
                    } catch (e) {
                        console.error('Error destroying Docker stream:', e);
                    }
                }

                safeClose();
            };

            // Start streaming process
            (async () => {
                try {
                    // Send initial connection message
                    safeEnqueue(`data: ${JSON.stringify({ type: 'connected', message: 'Connected to log stream' })}\n\n`);

                    // Find the container
                    const containerId = await findContainer(service);
                    safeEnqueue(`data: ${JSON.stringify({ type: 'info', line: `Found container: ${containerId}` })}\n\n`);

                    // Get logs stream from the container with follow=1 for real-time streaming
                    const logsRes = await dockerRequestStream(`/containers/${containerId}/logs?stdout=1&stderr=1&tail=${tail}&timestamps=0&follow=1`);

                    if (!logsRes.ok) {
                        throw new Error(`Failed to get logs: ${logsRes.status}`);
                    }

                    dockerStream = logsRes.stream;

                    // Create regex filter if specified
                    let filter: RegExp | null = null;
                    if (grep) {
                        try {
                            filter = new RegExp(grep, 'i');
                        } catch (e) {
                            safeEnqueue(`data: ${JSON.stringify({ type: 'error', line: 'Invalid regex pattern, using literal search' })}\n\n`);
                            // Fallback to literal search
                            const needle = grep.toLowerCase();
                            filter = new RegExp(needle.replace(/[.*+?^${}()|[\]\\]/g, '\\$&'), 'i');
                        }
                    }

                    // Stream logs in real-time
                    dockerStream.on('data', (chunk: Buffer) => {
                        if (isControllerClosed) return;

                        const lines = chunk.toString().split('\n').filter(line => line.trim());

                        for (const line of lines) {
                            if (isControllerClosed) break;

                            // Apply filter if specified
                            if (filter && !filter.test(line)) continue;

                            // Send the log line as an SSE event
                            safeEnqueue(`data: ${JSON.stringify({ type: 'log', line })}\n\n`);
                        }
                    });

                    // Handle stream end
                    dockerStream.on('end', () => {
                        if (!isControllerClosed) {
                            safeEnqueue(`data: ${JSON.stringify({ type: 'closed', code: 0 })}\n\n`);
                            cleanup();
                        }
                    });

                    // Handle stream errors
                    dockerStream.on('error', (err: Error) => {
                        if (!isControllerClosed) {
                            console.error('Docker stream error:', err);
                            safeEnqueue(`data: ${JSON.stringify({ type: 'error', line: err.message })}\n\n`);
                            cleanup();
                        }
                    });

                } catch (e: any) {
                    if (!isControllerClosed) {
                        console.error('Error setting up log stream:', e);
                        safeEnqueue(`data: ${JSON.stringify({ type: 'error', line: e.message })}\n\n`);
                        cleanup();
                    }
                }
            })();

            // Handle client disconnect
            url.searchParams.get('_disconnect') && cleanup();
        }
    });

    return new Response(stream, {
        headers: {
            'Content-Type': 'text/event-stream',
            'Cache-Control': 'no-cache',
            'Connection': 'keep-alive',
            'Access-Control-Allow-Origin': '*',
            'Access-Control-Allow-Headers': 'Cache-Control'
        }
    });
}
