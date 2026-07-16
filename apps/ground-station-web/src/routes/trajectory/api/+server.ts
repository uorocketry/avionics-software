import { getDbClient } from '$lib/server/db';
import { json, error } from '@sveltejs/kit';
import type { RequestHandler } from './$types';

const DEFAULT_CENTER = { lat: 47.986877, lon: -81.848765 };
const DEFAULT_RADIUS_KM = 50;

function computeBoundingBox(lat: number, lon: number, radiusKm: number) {
    const latDelta = radiusKm / 111.32; // degrees
    const cosLat = Math.cos((lat * Math.PI) / 180);
    const lonDelta = radiusKm / (111.32 * Math.max(0.0001, cosLat));
    return {
        minLat: lat - latDelta,
        maxLat: lat + latDelta,
        minLon: lon - lonDelta,
        maxLon: lon + lonDelta
    };
}

async function getLatest(lat: number, lon: number, radiusKm: number) {
    const db = getDbClient();
    const { minLat, maxLat, minLon, maxLon } = computeBoundingBox(lat, lon, radiusKm);
    const res = await db.execute({
        sql:
            "SELECT rf.timestamp_epoch AS ts, p.latitude AS lat, p.longitude AS lon, p.latitude_accuracy AS lat_acc, p.longitude_accuracy AS lon_acc, p.num_sv_used AS num_sv_used FROM RadioFrame rf JOIN SbgGpsPos p ON rf.data_type = 'SbgGpsPos' AND rf.data_id = p.id WHERE p.latitude IS NOT NULL AND p.longitude IS NOT NULL AND p.latitude != 0 AND p.longitude != 0 AND p.latitude BETWEEN ? AND ? AND p.longitude BETWEEN ? AND ? ORDER BY rf.timestamp_epoch DESC LIMIT 1",
        args: [minLat, maxLat, minLon, maxLon]
    });
    const row: any = res.rows[0] ?? null;
    if (!row) return null;
    return {
        ts: Number(row.ts),
        lat: Number(row.lat),
        lon: Number(row.lon),
        lat_acc: row.lat_acc != null ? Number(row.lat_acc) : null,
        lon_acc: row.lon_acc != null ? Number(row.lon_acc) : null,
        num_sv_used: row.num_sv_used != null ? Number(row.num_sv_used) : null
    };
}

async function getSeries(minutes: number, maxPoints: number, lat: number, lon: number, radiusKm: number) {
    const db = getDbClient();
    const seconds = Math.max(1, Math.floor(minutes * 60));
    const targetPoints = Math.max(50, Math.floor(maxPoints));
    const bucketSizeSec = Math.max(1, Math.ceil(seconds / targetPoints));
    const { minLat, maxLat, minLon, maxLon } = computeBoundingBox(lat, lon, radiusKm);

    // Less distracting decimation: choose the last point within each time bucket
    const res = await db.execute({
        sql:
            "WITH cte AS (" +
            " SELECT CAST(rf.timestamp_epoch / ? AS INTEGER) * ? AS bucket_ts, rf.timestamp_epoch AS ts, p.latitude AS lat, p.longitude AS lon" +
            " FROM RadioFrame rf JOIN SbgGpsPos p ON rf.data_type = 'SbgGpsPos' AND rf.data_id = p.id" +
            " WHERE rf.timestamp_epoch >= strftime('%s','now') - ?" +
            " AND p.latitude IS NOT NULL AND p.longitude IS NOT NULL AND p.latitude != 0 AND p.longitude != 0" +
            " AND p.latitude BETWEEN ? AND ? AND p.longitude BETWEEN ? AND ?" +
            ") SELECT c1.bucket_ts AS bucket_ts, c1.lat AS lat, c1.lon AS lon" +
            " FROM cte c1 JOIN (SELECT bucket_ts, MAX(ts) AS mts FROM cte GROUP BY bucket_ts) c2" +
            " ON c1.bucket_ts = c2.bucket_ts AND c1.ts = c2.mts" +
            " ORDER BY c1.bucket_ts ASC",
        args: [bucketSizeSec, bucketSizeSec, seconds, minLat, maxLat, minLon, maxLon]
    });

    const points = (res.rows as any[]).map((r) => ({
        ts: Number((r as any).bucket_ts),
        lat: Number((r as any).lat),
        lon: Number((r as any).lon)
    }));
    return { points };
}

export const GET: RequestHandler = async ({ url, request }) => {
    const minutes = Number(url.searchParams.get('minutes') || '240');
    const maxPoints = Number(url.searchParams.get('max_points') || '3000');
    const lat = Number(url.searchParams.get('lat') || String(DEFAULT_CENTER.lat));
    const lon = Number(url.searchParams.get('lon') || String(DEFAULT_CENTER.lon));
    const radiusKm = Number(url.searchParams.get('radius_km') || String(DEFAULT_RADIUS_KM));
    const useSSE = url.searchParams.get('sse') === '1';

    if (!Number.isFinite(lat) || !Number.isFinite(lon) || !Number.isFinite(radiusKm)) {
        throw error(400, 'Invalid lat/lon/radius');
    }

    if (useSSE) {
        let cancelled = false;
        let intervalId: any;
        const stream = new ReadableStream({
            start(controller) {
                const encoder = new TextEncoder();
                function sendEvent(event: string, payload: unknown) {
                    if (cancelled) return;
                    const data = typeof payload === 'string' ? payload : JSON.stringify(payload);
                    const chunk = `event: ${event}\n` + `data: ${data}\n\n`;
                    controller.enqueue(encoder.encode(chunk));
                }
                const push = async () => {
                    if (cancelled) return;
                    try {
                        const [latest, series] = await Promise.all([
                            getLatest(lat, lon, radiusKm),
                            getSeries(minutes, maxPoints, lat, lon, radiusKm)
                        ]);
                        sendEvent('trajectory', { latest, series, center: { lat, lon }, radius_km: radiusKm });
                    } catch (e) {
                        console.error('Trajectory SSE push error:', e);
                        sendEvent('error', { message: 'Failed to fetch trajectory data' });
                    }
                };
                push();
                intervalId = setInterval(push, 5000);
                request.signal.addEventListener('abort', () => {
                    cancelled = true;
                    clearInterval(intervalId);
                });
            },
            cancel() {
                cancelled = true;
                if (intervalId) clearInterval(intervalId);
            }
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
        const [latest, series] = await Promise.all([
            getLatest(lat, lon, radiusKm),
            getSeries(minutes, maxPoints, lat, lon, radiusKm)
        ]);
        return json({ latest, series, center: { lat, lon }, radius_km: radiusKm });
    } catch (e: any) {
        console.error('Trajectory API GET error:', e);
        throw error(500, `Failed to fetch trajectory data: ${e.message}`);
    }
};
