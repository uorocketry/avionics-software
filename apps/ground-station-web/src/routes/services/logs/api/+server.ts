import { error } from '@sveltejs/kit';
import http from 'node:http';

const DOCKER_SOCKET = '/var/run/docker.sock';

function dockerRequest(path: string): Promise<{ status: number; text: () => Promise<string>; ok: boolean; }> {
    return new Promise((resolve, reject) => {
        const req = http.request({
            socketPath: DOCKER_SOCKET,
            path,
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
}

async function getContainerLogs(containerName: string, tail: string, grep?: string): Promise<string> {
    try {
        // First, get container info to find the container ID
        const containersRes = await dockerRequest('/containers/json?all=true');
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

        // Get logs from the container
        const logsRes = await dockerRequest(`/containers/${container.Id}/logs?stdout=1&stderr=1&tail=${tail}&timestamps=0`);
        if (!logsRes.ok) {
            throw new Error(`Failed to get logs: ${logsRes.status}`);
        }

        let logs = await logsRes.text();

        // Apply grep filter if specified
        if (grep) {
            try {
                const grepRegex = new RegExp(grep, 'i');
                logs = logs.split('\n').filter(line => grepRegex.test(line)).join('\n');
            } catch (e) {
                throw new Error('Invalid regex pattern');
            }
        }

        return logs;
    } catch (e: any) {
        throw new Error(`Failed to get container logs: ${e.message}`);
    }
}

export async function GET({ url }) {
    const service = url.searchParams.get('service');
    const tail = url.searchParams.get('tail') || '50';
    const grep = url.searchParams.get('grep');

    if (!service) {
        throw error(400, 'service parameter is required');
    }

    try {
        const logs = await getContainerLogs(service, tail, grep || undefined);

        return new Response(logs, {
            headers: {
                'Content-Type': 'text/plain; charset=utf-8'
            }
        });
    } catch (e: any) {
        console.error('Error fetching logs:', e);
        throw error(500, e.message || 'Failed to fetch logs');
    }
}
