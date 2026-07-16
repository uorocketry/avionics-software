import { getDbClient } from '$lib/server/db';
import { json } from '@sveltejs/kit';
import type { RequestHandler } from './$types';

const encoder = new TextEncoder();
const activeControllers = new Set<ReadableStreamDefaultController>();
let pollIntervalId: NodeJS.Timeout | null = null;

async function pollAndBroadcast() {
    const db = getDbClient();
    try {
        const metricsResult = await db.execute({
            sql: `SELECT timestamp, rssi, packets_lost FROM RadioMetrics ORDER BY timestamp DESC LIMIT 20`,
            args: []
        });
        // Throughput: frames per second over last 10 seconds
        const fpsRes = await db.execute({
            sql: `SELECT COUNT(*) AS cnt FROM RadioFrame WHERE timestamp_epoch >= strftime('%s','now') - 10`,
            args: []
        });
        const framesIn10s = Number((fpsRes.rows[0] as any)?.cnt ?? 0);
        const frames_per_second = framesIn10s / 10.0;

        let total_packets_lost = 0;
        let total_radio_frames = 0;
        try {
            const summary = await db.execute({
                sql: `SELECT total_packets_lost, total_radio_frames FROM MetricsSummary LIMIT 1`,
                args: []
            });
            if (summary.rows.length > 0) {
                const row: any = summary.rows[0];
                total_packets_lost = Number(row.total_packets_lost ?? 0);
                total_radio_frames = Number(row.total_radio_frames ?? 0);
            }
        } catch (_) {
            const [lostAgg, frameAgg] = await Promise.all([
                db.execute({ sql: `SELECT COALESCE(SUM(packets_lost), 0) AS total FROM RadioMetrics`, args: [] }),
                db.execute({ sql: `SELECT COUNT(*) AS total FROM RadioFrame`, args: [] })
            ]);
            total_packets_lost = Number((lostAgg.rows[0] as any).total ?? 0);
            total_radio_frames = Number((frameAgg.rows[0] as any).total ?? 0);
        }

        const packet_loss_ratio = total_radio_frames > 0 ? total_packets_lost / total_radio_frames : 0;
        const packet_loss_percentage = packet_loss_ratio * 100;

        const payload = {
            metrics: metricsResult.rows,
            totals: { total_packets_lost, total_radio_frames, packet_loss_ratio, packet_loss_percentage, frames_per_second }
        };
        const chunk = encoder.encode(`event: metrics\n` + `data: ${JSON.stringify(payload)}\n\n`);
        for (const c of Array.from(activeControllers)) {
            try { c.enqueue(chunk); } catch { try { c.close(); } catch { } activeControllers.delete(c); }
        }
    } catch (e) {
        const errChunk = encoder.encode(`event: error\n` + `data: {"message":"Failed to fetch radio metrics"}\n\n`);
        for (const c of Array.from(activeControllers)) {
            try { c.enqueue(errChunk); } catch { try { c.close(); } catch { } activeControllers.delete(c); }
        }
        console.error('Radio central poll error:', e);
    }
}

function ensurePoller() {
    if (!pollIntervalId) {
        pollIntervalId = setInterval(pollAndBroadcast, 5000);
        // kick immediately
        pollAndBroadcast();
    }
}

export const GET: RequestHandler = async ({ url, request }) => {
    const useSSE = url.searchParams.get('sse') === '1';

    if (useSSE) {
        const stream = new ReadableStream({
            start(controller) {
                activeControllers.add(controller);
                ensurePoller();
                request.signal.addEventListener('abort', () => {
                    activeControllers.delete(controller);
                    try { controller.close(); } catch { }
                });
            },
            cancel() { }
        });

        return new Response(stream, {
            headers: {
                'Content-Type': 'text/event-stream',
                'Cache-Control': 'no-cache, no-transform',
                'Connection': 'keep-alive'
            }
        });
    }

    // Regular GET request
    const db = getDbClient();
    const metricsResult = await db.execute({
        sql: `SELECT timestamp, rssi, packets_lost FROM RadioMetrics ORDER BY timestamp DESC LIMIT 20`,
        args: []
    });

    // Summary (prefer summary table, fallback to aggregates)
    let total_packets_lost = 0;
    let total_radio_frames = 0;
    try {
        const summary = await db.execute({
            sql: `SELECT total_packets_lost, total_radio_frames FROM MetricsSummary LIMIT 1`,
            args: []
        });
        if (summary.rows.length > 0) {
            const row: any = summary.rows[0];
            total_packets_lost = Number(row.total_packets_lost ?? 0);
            total_radio_frames = Number(row.total_radio_frames ?? 0);
        }
    } catch (_) {
        const [lostAgg, frameAgg] = await Promise.all([
            db.execute({ sql: `SELECT COALESCE(SUM(packets_lost), 0) AS total FROM RadioMetrics`, args: [] }),
            db.execute({ sql: `SELECT COUNT(*) AS total FROM RadioFrame`, args: [] })
        ]);
        total_packets_lost = Number((lostAgg.rows[0] as any).total ?? 0);
        total_radio_frames = Number((frameAgg.rows[0] as any).total ?? 0);
    }

    const packet_loss_ratio = total_radio_frames > 0 ? total_packets_lost / total_radio_frames : 0;
    const packet_loss_percentage = packet_loss_ratio * 100;

    return json({
        metrics: metricsResult.rows,
        totals: {
            total_packets_lost,
            total_radio_frames,
            packet_loss_ratio,
            packet_loss_percentage
        }
    });
};
