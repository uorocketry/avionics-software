import { getDbClient } from '$lib/server/db';
import { json, error } from '@sveltejs/kit';
import type { RequestHandler } from './$types';

const DEFAULT_QNH_KPA = 102.5; // Estimated Timmin's ON average for August 2025 at CYTS

function altitudeFromPressureKPa(pressureKpa: number, qnhKpa: number = DEFAULT_QNH_KPA): number {
    if (!isFinite(pressureKpa) || pressureKpa <= 0) return NaN;
    const ratio = pressureKpa / qnhKpa;
    return 44330 * (1 - Math.pow(ratio, 0.190263));
}

function altitudeFromPressurePa(pressurePa: number, qnhKpa: number = DEFAULT_QNH_KPA): number {
    return altitudeFromPressureKPa(pressurePa / 1000, qnhKpa);
}

async function getLatestSnapshot(qnhKpa: number) {
    const db = getDbClient();

    const [baroLatest, airLatest, gnssLatest] = await Promise.all([
        // Barometer latest (RadioFrame timestamp)
        db.execute({
            sql:
                "SELECT rf.timestamp_epoch AS ts, b.id AS id, b.pressure_kpa AS pressure_kpa, b.temperature_celsius AS temperature_celsius FROM RadioFrame rf JOIN Barometer b ON rf.data_type = 'Barometer' AND rf.data_id = b.id ORDER BY rf.timestamp_epoch DESC LIMIT 1",
            args: []
        }),
        // SbgAir latest (RadioFrame timestamp)
        db.execute({
            sql:
                "SELECT rf.timestamp_epoch AS ts, a.id AS id, a.pressure_abs AS pressure_pa FROM RadioFrame rf JOIN SbgAir a ON rf.data_type = 'SbgAir' AND rf.data_id = a.id ORDER BY rf.timestamp_epoch DESC LIMIT 1",
            args: []
        }),
        // SbgGpsPos latest valid altitude > 0 (RadioFrame timestamp)
        db.execute({
            sql:
                "SELECT rf.timestamp_epoch AS ts, p.id AS id, p.altitude AS altitude_m FROM RadioFrame rf JOIN SbgGpsPos p ON rf.data_type = 'SbgGpsPos' AND rf.data_id = p.id WHERE p.altitude IS NOT NULL AND p.altitude > 0 ORDER BY rf.timestamp_epoch DESC LIMIT 1",
            args: []
        })
    ]);

    const latestBaroRow: any = baroLatest.rows[0] ?? null;
    const latestAirRow: any = airLatest.rows[0] ?? null;
    const latestGnssRow: any = gnssLatest.rows[0] ?? null;

    const latest = {
        barometer: latestBaroRow
            ? {
                ts: Number(latestBaroRow.ts),
                pressure_kpa: Number(latestBaroRow.pressure_kpa),
                temperature_celsius: latestBaroRow.temperature_celsius !== null ? Number(latestBaroRow.temperature_celsius) : null,
                altitude_m: altitudeFromPressureKPa(Number(latestBaroRow.pressure_kpa), qnhKpa)
            }
            : null,
        sbgAir: latestAirRow
            ? {
                ts: Number(latestAirRow.ts),
                pressure_pa: Number(latestAirRow.pressure_pa),
                pressure_kpa: Number(latestAirRow.pressure_pa) / 1000,
                altitude_m: altitudeFromPressurePa(Number(latestAirRow.pressure_pa), qnhKpa)
            }
            : null,
        gnss: latestGnssRow
            ? {
                ts: Number(latestGnssRow.ts),
                altitude_m: Number(latestGnssRow.altitude_m)
            }
            : null
    };

    return latest;
}

async function getSeries(minutes: number, qnhKpa: number, maxPoints: number) {
    const db = getDbClient();
    const seconds = Number.isFinite(minutes) && minutes > 0 ? Math.floor(minutes * 60) : 900;
    const targetPoints = Number.isFinite(maxPoints) && maxPoints > 0 ? Math.floor(maxPoints) : 1500;
    const bucketSizeSec = Math.max(1, Math.ceil(seconds / targetPoints));

    const [baroSeriesRes, airSeriesRes, gnssSeriesRes] = await Promise.all([
        // Barometer series joined with RadioFrame for timestamps, bucket-averaged
        db.execute({
            sql:
                "SELECT CAST(rf.timestamp_epoch / ? AS INTEGER) * ? AS bucket_ts, AVG(b.pressure_kpa) AS pressure_kpa FROM RadioFrame rf JOIN Barometer b ON rf.data_type = 'Barometer' AND rf.data_id = b.id WHERE rf.timestamp_epoch >= strftime('%s','now') - ? GROUP BY bucket_ts ORDER BY bucket_ts ASC",
            args: [bucketSizeSec, bucketSizeSec, seconds]
        }),
        // SbgAir series bucket-averaged
        db.execute({
            sql:
                "SELECT CAST(rf.timestamp_epoch / ? AS INTEGER) * ? AS bucket_ts, AVG(a.pressure_abs) AS pressure_pa FROM RadioFrame rf JOIN SbgAir a ON rf.data_type = 'SbgAir' AND rf.data_id = a.id WHERE rf.timestamp_epoch >= strftime('%s','now') - ? GROUP BY bucket_ts ORDER BY bucket_ts ASC",
            args: [bucketSizeSec, bucketSizeSec, seconds]
        }),
        // GNSS series (ignore <= 0), bucket-averaged
        db.execute({
            sql:
                "SELECT CAST(rf.timestamp_epoch / ? AS INTEGER) * ? AS bucket_ts, AVG(p.altitude) AS altitude_m FROM RadioFrame rf JOIN SbgGpsPos p ON rf.data_type = 'SbgGpsPos' AND rf.data_id = p.id WHERE rf.timestamp_epoch >= strftime('%s','now') - ? AND p.altitude IS NOT NULL AND p.altitude > 0 GROUP BY bucket_ts ORDER BY bucket_ts ASC",
            args: [bucketSizeSec, bucketSizeSec, seconds]
        })
    ]);

    // Convert to Carbon line chart format
    const data: Array<{ group: string; date: Date; value: number }> = [];

    for (const row of baroSeriesRes.rows as any[]) {
        const ts = Number((row as any).bucket_ts);
        const pressureKpa = Number((row as any).pressure_kpa);
        const alt = altitudeFromPressureKPa(pressureKpa, qnhKpa);
        if (isFinite(alt)) data.push({ group: 'Altitude (Barometer)', date: new Date(ts * 1000), value: alt });
    }
    for (const row of airSeriesRes.rows as any[]) {
        const ts = Number((row as any).bucket_ts);
        const pressurePa = Number((row as any).pressure_pa);
        const alt = altitudeFromPressurePa(pressurePa, qnhKpa);
        if (isFinite(alt)) data.push({ group: 'Altitude (SBG Air)', date: new Date(ts * 1000), value: alt });
    }
    for (const row of gnssSeriesRes.rows as any[]) {
        const ts = Number((row as any).bucket_ts);
        const alt = Number((row as any).altitude_m);
        if (isFinite(alt) && alt > 0) data.push({ group: 'Altitude (GNSS)', date: new Date(ts * 1000), value: alt });
    }

    return { data };
}

export const GET: RequestHandler = async ({ url, request }) => {
    const minutes = Number(url.searchParams.get('minutes') || '999');
    const maxPoints = Number(url.searchParams.get('max_points') || '1500');
    // Ignore any client-provided qnh_kpa. Use DEFAULT_QNH_KPA.
    const qnhKpa = DEFAULT_QNH_KPA;
    const useSSE = url.searchParams.get('sse') === '1';

    if (useSSE) {
        const encoder = new TextEncoder();
        const activeControllers: Set<ReadableStreamDefaultController> = (globalThis as any).__altitudeControllers ?? new Set<ReadableStreamDefaultController>();
        (globalThis as any).__altitudeControllers = activeControllers;
        let poller: NodeJS.Timeout | null = (globalThis as any).__altitudePoller ?? null;
        const stream = new ReadableStream({
            start(controller) {
                activeControllers.add(controller);
                const poll = async () => {
                    try {
                        const [latest, series] = await Promise.all([getLatestSnapshot(qnhKpa), getSeries(minutes, qnhKpa, maxPoints)]);
                        const chunk = encoder.encode(`event: altitude\n` + `data: ${JSON.stringify({ latest, series, qnh_kpa: qnhKpa })}\n\n`);
                        for (const c of Array.from(activeControllers)) {
                            try { c.enqueue(chunk); } catch { try { c.close(); } catch { } activeControllers.delete(c); }
                        }
                    } catch (e) {
                        const err = encoder.encode(`event: error\n` + `data: {"message":"Failed to fetch altitude data"}\n\n`);
                        for (const c of Array.from(activeControllers)) {
                            try { c.enqueue(err); } catch { try { c.close(); } catch { } activeControllers.delete(c); }
                        }
                        console.error('Altitude central poll error:', e);
                    }
                };
                if (!poller) {
                    poller = setInterval(poll, 1000);
                    (globalThis as any).__altitudePoller = poller;
                    poll();
                }
                request.signal.addEventListener('abort', () => {
                    activeControllers.delete(controller);
                    try { controller.close(); } catch { }
                    if (activeControllers.size === 0 && poller) {
                        clearInterval(poller);
                        (globalThis as any).__altitudePoller = null;
                    }
                });
            },
            cancel() { }
        });

        return new Response(stream, {
            headers: {
                'Content-Type': 'text/event-stream',
                'Cache-Control': 'no-cache, no-transform',
                Connection: 'keep-alive'
            }
        });
    }

    try {
        const [latest, series] = await Promise.all([getLatestSnapshot(qnhKpa), getSeries(minutes, qnhKpa, maxPoints)]);
        return json({ latest, series, qnh_kpa: qnhKpa });
    } catch (e: any) {
        console.error('Altitude API GET error:', e);
        throw error(500, `Failed to fetch altitude data: ${e.message}`);
    }
};
