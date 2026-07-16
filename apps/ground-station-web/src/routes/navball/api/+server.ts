import type { RequestHandler } from './$types';
import { getDbClient } from '$lib/server/db';

// Module-scope state
let latestQuatData: any = {
	time_stamp: Math.floor(Date.now() / 1000),
	status: 'INITIALIZING',
	quaternion_w: 1,
	quaternion_x: 0,
	quaternion_y: 0,
	quaternion_z: 0
};
const activeControllers = new Set<ReadableStreamDefaultController>();
const encoder = new TextEncoder();
let dbPollingIntervalId: NodeJS.Timeout | null = null;
const DB_POLL_INTERVAL_MS = 500;

async function fetchAndUpdateQuatData() {
	try {
		const db = getDbClient();
		const result = await db.execute({
			sql: `SELECT time_stamp, quaternion_w, quaternion_x, quaternion_y, quaternion_z, status FROM SbgEkfQuat ORDER BY id DESC LIMIT 1`
		});
		latestQuatData = result.rows[0] ?? {
			time_stamp: Math.floor(Date.now() / 1000),
			status: 'NO_DATA',
			quaternion_w: 1,
			quaternion_x: 0,
			quaternion_y: 0,
			quaternion_z: 0
		};
	} catch (e: any) {
		console.error('Error fetching EKF quaternion in central poller:', e);
		latestQuatData = { ...latestQuatData, status: 'DB_ERROR', errorDetails: e.message };
	}

	// Broadcast + prune closed sockets
	const message = `data: ${JSON.stringify(latestQuatData)}\n\n`;
	for (const controller of Array.from(activeControllers)) {
		try {
			controller.enqueue(encoder.encode(message));
		} catch {
			try {
				controller.close();
			} catch {}
			activeControllers.delete(controller);
		}
	}
}

function ensurePoller() {
	if (dbPollingIntervalId) return;
	void fetchAndUpdateQuatData();
	dbPollingIntervalId = setInterval(fetchAndUpdateQuatData, DB_POLL_INTERVAL_MS);
}

function stopPollerWhenIdle() {
	if (activeControllers.size === 0 && dbPollingIntervalId) {
		clearInterval(dbPollingIntervalId);
		dbPollingIntervalId = null;
	}
}

export const GET: RequestHandler = (event) => {
	let controllerForThisClient: ReadableStreamDefaultController;

	const stream = new ReadableStream({
		start(controller) {
			controllerForThisClient = controller;
			activeControllers.add(controllerForThisClient);
			ensurePoller();
			console.log('SSE client connected. Total clients:', activeControllers.size);

			// Initial event flush
			controller.enqueue(encoder.encode(`data: ${JSON.stringify(latestQuatData)}\n\n`));

			// Robust cleanup with Node adapter if available
			const node = (event as any)?.platform?.node as { req?: any; res?: any } | undefined;
			if (node) {
				const cleanup = () => {
					if (controllerForThisClient) {
						activeControllers.delete(controllerForThisClient);
						stopPollerWhenIdle();
						try {
							controllerForThisClient.close();
						} catch {}
						node.req.off?.('close', cleanup);
						node.res.off?.('close', cleanup);
						node.req.off?.('aborted', cleanup);
						console.log(
							'SSE client disconnected (socket close). Total clients:',
							activeControllers.size
						);
					}
				};
				node.req.on?.('close', cleanup);
				node.res.on?.('close', cleanup);
				node.req.on?.('aborted', cleanup);
			}

			// Fallback: abort signal
			event.request.signal.addEventListener('abort', () => {
				if (controllerForThisClient) {
					activeControllers.delete(controllerForThisClient);
					stopPollerWhenIdle();
					try {
						controllerForThisClient.close();
					} catch {}
					console.log('SSE client disconnected (abort). Total clients:', activeControllers.size);
				}
			});
		},
		cancel() {
			if (controllerForThisClient) {
				activeControllers.delete(controllerForThisClient);
				stopPollerWhenIdle();
				console.log('SSE client disconnected (cancel). Total clients:', activeControllers.size);
			}
		}
	});

	return new Response(stream, {
		headers: {
			'Content-Type': 'text/event-stream',
			'Cache-Control': 'no-cache, no-transform',
			Connection: 'keep-alive'
		}
	});
};
