import { getDbClient } from '$lib/server/db';
import type { RequestHandler } from './$types';

type Row = { id: number; kind: 'State' | 'Event'; value: string; ts: number | null; duration_s: number | null };

// Central SSE state
const encoder = new TextEncoder();
const activeControllers: Set<ReadableStreamDefaultController> = new Set();
let pollIntervalId: NodeJS.Timeout | null = null;
let lastBroadcastStateId: number | null = null;
let lastBroadcastEventId: number | null = null;

async function getLatest(kind: 'State' | 'Event'): Promise<Row | null> {
    const db = getDbClient();
    // Use Phoenix* tables
    const table = kind === 'State' ? 'PhoenixState' : 'PhoenixEvent';
    const column = kind === 'State' ? 'state' : 'event';
    const dataType = kind === 'State' ? 'PhoenixState' : 'PhoenixEvent';
    try {
        const res = await db.execute({
            sql: `SELECT t.id AS id, t.${column} AS value, rf.timestamp_epoch AS ts
                  FROM ${table} t
                  JOIN RadioFrame rf ON rf.data_id = t.id AND rf.data_type = '${dataType}'
                  ORDER BY rf.timestamp_epoch DESC
                  LIMIT 1`,
            args: []
        });
        if (res.rows.length) {
            const r = res.rows[0];
            const tsNum = r.ts ? Number(r.ts) : null;
            const now = Math.floor(Date.now() / 1000);
            const duration = tsNum != null ? Math.max(0, now - tsNum) : null;
            return { id: Number(r.id), kind, value: String(r.value), ts: tsNum, duration_s: duration };
        }
    } catch (e) {
        console.error(`${kind} API: join query failed, falling back`, e);
    }
    try {
        const res2 = await db.execute({
            sql: `SELECT id, ${column} AS value FROM ${table} ORDER BY id DESC LIMIT 1`,
            args: []
        });
        if (res2.rows.length) {
            const r = res2.rows[0];
            return { id: Number(r.id), kind, value: String(r.value), ts: null, duration_s: null };
        }
    } catch (e) {
        console.error(`${kind} API: fallback latest query failed`, e);
    }
    return null;
}

async function getSnapshot(limit = 50): Promise<Row[]> {
    const db = getDbClient();
    try {
        // Only transitions (changes) for state and event using window LAG over time
        const stateRes = await db.execute({
            sql: `WITH ordered AS (
                      SELECT ps.id AS id,
                             ps.state AS value,
                             rf.timestamp_epoch AS ts,
                             LAG(ps.state) OVER (ORDER BY rf.timestamp_epoch ASC) AS prev_value,
                             LEAD(rf.timestamp_epoch) OVER (ORDER BY rf.timestamp_epoch ASC) AS next_ts
                      FROM PhoenixState ps
                      JOIN RadioFrame rf ON rf.data_id = ps.id AND rf.data_type = 'PhoenixState'
                  )
                  SELECT id, value, ts,
                         COALESCE(next_ts, strftime('%s','now')) - ts AS duration_s,
                         'State' AS kind
                  FROM ordered
                  WHERE prev_value IS NULL OR value <> prev_value
                  ORDER BY ts DESC
                  LIMIT ?`,
            args: [limit]
        });
        const eventRes = await db.execute({
            sql: `WITH ordered AS (
                      SELECT pe.id AS id,
                             pe.event AS value,
                             rf.timestamp_epoch AS ts,
                             LAG(pe.event) OVER (ORDER BY rf.timestamp_epoch ASC) AS prev_value,
                             LEAD(rf.timestamp_epoch) OVER (ORDER BY rf.timestamp_epoch ASC) AS next_ts
                      FROM PhoenixEvent pe
                      JOIN RadioFrame rf ON rf.data_id = pe.id AND rf.data_type = 'PhoenixEvent'
                  )
                  SELECT id, value, ts,
                         COALESCE(next_ts, strftime('%s','now')) - ts AS duration_s,
                         'Event' AS kind
                  FROM ordered
                  WHERE prev_value IS NULL OR value <> prev_value
                  ORDER BY ts DESC
                  LIMIT ?`,
            args: [limit]
        });
        const toRow = (r: any) => ({ id: Number(r.id), kind: r.kind as 'State' | 'Event', value: String(r.value), ts: r.ts ? Number(r.ts) : null, duration_s: r.duration_s != null ? Number(r.duration_s) : null });
        return [...stateRes.rows.map(toRow), ...eventRes.rows.map(toRow)]
            .sort((a, b) => (b.ts ?? 0) - (a.ts ?? 0))
            .slice(0, limit);
    } catch (e) {
        console.error('State API snapshot failed', e);
        return [];
    }
}

function ensurePoller() {
    if (pollIntervalId) return;
    pollIntervalId = setInterval(async () => {
        try {
            const [s, e] = await Promise.all([getLatest('State'), getLatest('Event')]);
            const newRows: Row[] = [];
            if (s && (lastBroadcastStateId === null || s.id !== lastBroadcastStateId)) {
                lastBroadcastStateId = s.id;
                newRows.push(s);
            }
            if (e && (lastBroadcastEventId === null || e.id !== lastBroadcastEventId)) {
                lastBroadcastEventId = e.id;
                newRows.push(e);
            }
            if (newRows.length) {
                newRows.sort((a, b) => (b.ts ?? 0) - (a.ts ?? 0));
                const payload = JSON.stringify(newRows);
                const chunk = encoder.encode(`event: append\n` + `data: ${payload}\n\n`);
                for (const c of Array.from(activeControllers)) {
                    try { c.enqueue(chunk); } catch { try { c.close(); } catch { } activeControllers.delete(c); }
                }
            }
        } catch (e) {
            // ignore single tick failures
        }
    }, 1000);
}

export const GET: RequestHandler = async ({ request }) => {
    const stream = new ReadableStream({
        start(controller) {
            activeControllers.add(controller);
            ensurePoller();
            // Initial snapshot for quick render
            getSnapshot(50)
                .then((rows) => controller.enqueue(encoder.encode(`event: snapshot\n` + `data: ${JSON.stringify(rows)}\n\n`)))
                .catch(() => { });
            request.signal.addEventListener('abort', () => {
                activeControllers.delete(controller);
                try { controller.close(); } catch { }
                if (activeControllers.size === 0 && pollIntervalId) { clearInterval(pollIntervalId); pollIntervalId = null; }
            });
        },
        cancel() {
            // best-effort cleanup
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
