import { getDbClient } from '$lib/server/db';
import { json, error } from '@sveltejs/kit';
import type { Row } from '@libsql/client';
import type { RequestHandler } from './$types';

// Re-use the helper function (could be moved to a shared lib if used elsewhere too)
async function getLatestRow(tableName: string): Promise<Row | null> {
    const db = getDbClient();
    try {
        const result = await db.execute({
            sql: `SELECT * FROM ${tableName} ORDER BY time_stamp DESC LIMIT 1`,
            args: []
        });
        return result.rows.length > 0 ? result.rows[0] : null;
    } catch (e: any) {
        console.error(`Error fetching latest row from ${tableName} for API:`, e);
        return null;
    }
}

// Map of supported metrics to their safe table/column definitions
// This prevents SQL injection since table/columns are not taken directly from user input
const METRICS: Record<string, { table: string; columns: { key: string; label: string }[] }> = {
    air_altitude: { table: 'SbgAir', columns: [{ key: 'altitude', label: 'Altitude (m)' }] },
    air_pressure: {
        table: 'SbgAir',
        columns: [
            { key: 'pressure_abs', label: 'Abs Pressure (Pa)' },
            { key: 'pressure_diff', label: 'Diff Pressure (Pa)' }
        ]
    },
    air_true_airspeed: { table: 'SbgAir', columns: [{ key: 'true_airspeed', label: 'True Airspeed (m/s)' }] },
    ekf_altitude: { table: 'SbgEkfNav', columns: [{ key: 'position_altitude', label: 'Altitude (m)' }] },
    gpsvel_velocity: {
        table: 'SbgGpsVel',
        columns: [
            { key: 'velocity_north', label: 'Vel North (m/s)' },
            { key: 'velocity_east', label: 'Vel East (m/s)' },
            { key: 'velocity_down', label: 'Vel Down (m/s)' }
        ]
    },
    gpsvel_course: { table: 'SbgGpsVel', columns: [{ key: 'course', label: 'Course (deg)' }] },
    imu_temperature: { table: 'SbgImu', columns: [{ key: 'temperature', label: 'IMU Temp (°C)' }] },
    imu_accel: {
        table: 'SbgImu',
        columns: [
            { key: 'accelerometer_x', label: 'Accel X (m/s²)' },
            { key: 'accelerometer_y', label: 'Accel Y (m/s²)' },
            { key: 'accelerometer_z', label: 'Accel Z (m/s²)' }
        ]
    },
    imu_gyro: {
        table: 'SbgImu',
        columns: [
            { key: 'gyroscope_x', label: 'Gyro X (rad/s)' },
            { key: 'gyroscope_y', label: 'Gyro Y (rad/s)' },
            { key: 'gyroscope_z', label: 'Gyro Z (rad/s)' }
        ]
    }
};

async function getMetricSeries(metric: string, minutes: number) {
    const def = METRICS[metric];
    if (!def) {
        throw error(400, `Unsupported metric: ${metric}`);
    }
    const db = getDbClient();
    const seconds = Math.max(1, Math.floor(minutes * 60));

    // Build a SELECT that returns ts and one column per requested series
    const selectCols = ['time_stamp AS ts', ...def.columns.map((c) => `${c.key} AS ${c.key}`)].join(', ');
    const sql = `SELECT ${selectCols} FROM ${def.table} WHERE time_stamp >= strftime('%s','now') - ? ORDER BY time_stamp ASC`;

    const result = await db.execute({ sql, args: [seconds] });

    // Convert rows into Carbon Charts data format: { group, date, value }
    const data: Array<{ group: string; date: Date; value: number }> = [];
    for (const row of result.rows) {
        const ts = Number(row.ts);
        for (const col of def.columns) {
            const val = row[col.key] as number | null | undefined;
            if (typeof val === 'number') {
                data.push({ group: col.label, date: new Date(ts * 1000), value: val });
            }
        }
    }

    return {
        metric,
        table: def.table,
        seriesLabels: def.columns.map((c) => c.label),
        data
    };
}

const encoder = new TextEncoder();
const activeControllers = new Set<ReadableStreamDefaultController>();
let pollIntervalId: NodeJS.Timeout | null = null;
let lastMetricPayload: any = null;
let lastSnapshotPayload: any = null;

async function centralPoll(metric: string | null, minutes: number) {
    // If metric is provided, poll metric + snapshot; otherwise only snapshot
    try {
        const series = metric ? await getMetricSeries(metric, minutes) : null;
        const snapshot = await Promise.all([
            getLatestRow('SbgUtcTime'),
            getLatestRow('SbgAir'),
            getLatestRow('SbgEkfQuat'),
            getLatestRow('SbgEkfNav'),
            getLatestRow('SbgImu'),
            getLatestRow('SbgGpsVel'),
            getLatestRow('SbgGpsPos')
        ]).then(([utc, air, quat, nav, imu, vel, pos]) => ({
            utcTime: utc ?? {},
            air: air ?? {},
            ekfQuat: quat ?? {},
            ekfNav: nav ?? {},
            imu: imu ?? {},
            gpsVel: vel ?? {},
            gpsPos: pos ?? {}
        }));
        if (series) lastMetricPayload = series;
        lastSnapshotPayload = snapshot;
        const chunks: Uint8Array[] = [];
        if (series) chunks.push(encoder.encode(`event: metric\n` + `data: ${JSON.stringify(series)}\n\n`));
        chunks.push(encoder.encode(`event: snapshot\n` + `data: ${JSON.stringify(snapshot)}\n\n`));
        for (const c of Array.from(activeControllers)) {
            try { for (const ch of chunks) c.enqueue(ch); } catch { try { c.close(); } catch { } activeControllers.delete(c); }
        }
    } catch (e) {
        const err = encoder.encode(`event: error\n` + `data: {"message":"Failed to fetch data"}\n\n`);
        for (const c of Array.from(activeControllers)) {
            try { c.enqueue(err); } catch { try { c.close(); } catch { } activeControllers.delete(c); }
        }
        console.error('SBG central poll error:', e);
    }
}

export const GET: RequestHandler = async ({ url, request }) => {
    const metric = url.searchParams.get('metric');
    const minutes = Number(url.searchParams.get('minutes') || '10');
    const useSSE = url.searchParams.get('sse') === '1';

    if (metric && useSSE) {
        const stream = new ReadableStream({
            start(controller) {
                activeControllers.add(controller);
                if (!pollIntervalId) {
                    pollIntervalId = setInterval(() => centralPoll(metric, minutes), 5000);
                    centralPoll(metric, minutes);
                } else {
                    if (lastMetricPayload) controller.enqueue(encoder.encode(`event: metric\n` + `data: ${JSON.stringify(lastMetricPayload)}\n\n`));
                    if (lastSnapshotPayload) controller.enqueue(encoder.encode(`event: snapshot\n` + `data: ${JSON.stringify(lastSnapshotPayload)}\n\n`));
                }
                request.signal.addEventListener('abort', () => {
                    activeControllers.delete(controller);
                    try { controller.close(); } catch { }
                    if (activeControllers.size === 0 && pollIntervalId) { clearInterval(pollIntervalId); pollIntervalId = null; }
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

    if (metric) {
        try {
            const series = await getMetricSeries(metric, minutes);
            return json(series);
        } catch (e: any) {
            if (e?.status && e?.body) throw e; // forward handled errors
            console.error('Error fetching metric series:', e);
            throw error(500, `Failed to fetch metric series: ${e.message}`);
        }
    }

    // Fallback: latest snapshot, preserving existing behavior
    try {
        const [
            latestUtcTime,
            latestAir,
            latestEkfQuat,
            latestEkfNav,
            latestImu,
            latestGpsVel,
            latestGpsPos
        ] = await Promise.all([
            getLatestRow('SbgUtcTime'),
            getLatestRow('SbgAir'),
            getLatestRow('SbgEkfQuat'),
            getLatestRow('SbgEkfNav'),
            getLatestRow('SbgImu'),
            getLatestRow('SbgGpsVel'),
            getLatestRow('SbgGpsPos')
        ]);

        const responseData = {
            utcTime: latestUtcTime ?? {},
            air: latestAir ?? {},
            ekfQuat: latestEkfQuat ?? {},
            ekfNav: latestEkfNav ?? {},
            imu: latestImu ?? {},
            gpsVel: latestGpsVel ?? {},
            gpsPos: latestGpsPos ?? {}
        };

        return json(responseData);
    } catch (e: any) {
        console.error('Error in SBG API GET handler:', e);
        throw error(500, `Failed to fetch SBG data: ${e.message}`);
    }
};