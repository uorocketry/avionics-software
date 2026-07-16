import { getDbClient } from '$lib/server/db';
import { json, error } from '@sveltejs/kit';
import type { RequestHandler } from './$types';

const encoder = new TextEncoder();
const activeControllers = new Set<ReadableStreamDefaultController>();
let pollIntervalId: NodeJS.Timeout | null = null;
let lastPayload: any = null;

type ImuRow = {
    ts: number;
    accelerometer_x: number | null;
    accelerometer_y: number | null;
    accelerometer_z: number | null;
    gyroscope_x: number | null;
    gyroscope_y: number | null;
    gyroscope_z: number | null;
    delta_velocity_x: number | null;
    delta_velocity_y: number | null;
    delta_velocity_z: number | null;
    delta_angle_x: number | null;
    delta_angle_y: number | null;
    delta_angle_z: number | null;
    temperature: number | null;
};

function magnitude(x: number, y: number, z: number): number {
    return Math.sqrt(x * x + y * y + z * z);
}

async function fetchImuSeries(minutes: number): Promise<ImuRow[]> {
    const db = getDbClient();
    const seconds = Number.isFinite(minutes) && minutes > 0 ? Math.floor(minutes * 60) : 600;
    const res = await db.execute({
        sql: `SELECT rf.timestamp_epoch AS ts,
                     i.accelerometer_x, i.accelerometer_y, i.accelerometer_z,
                     i.gyroscope_x, i.gyroscope_y, i.gyroscope_z,
                     i.delta_velocity_x, i.delta_velocity_y, i.delta_velocity_z,
                     i.delta_angle_x, i.delta_angle_y, i.delta_angle_z,
                     i.temperature AS temperature
              FROM RadioFrame rf
              JOIN SbgImu i ON rf.data_type = 'SbgImu' AND rf.data_id = i.id
              WHERE rf.timestamp_epoch >= strftime('%s','now') - ?
              ORDER BY rf.timestamp_epoch ASC`,
        args: [seconds]
    });
    return res.rows as unknown as ImuRow[];
}

function buildMetrics(rows: ImuRow[]) {
    const accelChart: Array<{ group: string; date: Date; value: number }> = [];
    const gyroChart: Array<{ group: string; date: Date; value: number }> = [];

    let prev: ImuRow | null = null;
    let prevAccelDvdt = 0;
    let cumDeltaV = 0;
    let cumRotation = 0;

    let peakAccel = 0; // m/s^2 from accelerometer
    let peakAccelDvdt = 0; // m/s^2 from delta_v/dt
    let peakGyro = 0; // rad/s from gyro
    let peakOmegaDadt = 0; // rad/s from delta_angle/dt
    let latestTemp: number | null = null;

    for (const r of rows) {
        const date = new Date(r.ts * 1000);
        const ax = Number(r.accelerometer_x ?? 0);
        const ay = Number(r.accelerometer_y ?? 0);
        const az = Number(r.accelerometer_z ?? 0);
        const gx = Number(r.gyroscope_x ?? 0);
        const gy = Number(r.gyroscope_y ?? 0);
        const gz = Number(r.gyroscope_z ?? 0);
        const dvx = Number(r.delta_velocity_x ?? 0);
        const dvy = Number(r.delta_velocity_y ?? 0);
        const dvz = Number(r.delta_velocity_z ?? 0);
        const dax = Number(r.delta_angle_x ?? 0);
        const day = Number(r.delta_angle_y ?? 0);
        const daz = Number(r.delta_angle_z ?? 0);

        latestTemp = r.temperature ?? latestTemp;

        const accelMag = magnitude(ax, ay, az);
        peakAccel = Math.max(peakAccel, accelMag);
        accelChart.push({ group: 'Accel |a| (sensor)', date, value: accelMag });

        const gyroMag = magnitude(gx, gy, gz);
        peakGyro = Math.max(peakGyro, gyroMag);
        gyroChart.push({ group: 'Gyro |ω| (sensor)', date, value: gyroMag });

        if (prev && r.ts > prev.ts) {
            const dt = Math.max(1e-6, r.ts - prev.ts);
            const dvMag = magnitude(dvx, dvy, dvz);
            const daMag = magnitude(dax, day, daz);

            const accelDvdt = dvMag / dt;
            const omegaDadt = daMag / dt;

            peakAccelDvdt = Math.max(peakAccelDvdt, accelDvdt);
            peakOmegaDadt = Math.max(peakOmegaDadt, omegaDadt);
            accelChart.push({ group: 'Accel |Δv/Δt|', date, value: accelDvdt });
            gyroChart.push({ group: 'Rate |Δθ/Δt|', date, value: omegaDadt });

            cumDeltaV += dvMag;
            cumRotation += daMag;

            // Jerk could be added later if needed using prevAccelDvdt
            prevAccelDvdt = accelDvdt;
        }

        prev = r;
    }

    const stats = {
        peak_accel_mps2: peakAccel,
        peak_accel_from_dvdt_mps2: peakAccelDvdt,
        peak_gyro_rads: peakGyro,
        peak_rate_from_dadt_rads: peakOmegaDadt,
        cumulative_delta_v_mps: cumDeltaV,
        cumulative_rotation_turns: cumRotation / (2 * Math.PI),
        latest_temperature_c: latestTemp
    };

    return { accelChart, gyroChart, stats };
}

async function getPayload(minutes: number) {
    const series = await fetchImuSeries(minutes);
    const { accelChart, gyroChart, stats } = buildMetrics(series);
    return { series: { accel: accelChart, gyro: gyroChart }, stats };
}

export const GET: RequestHandler = async ({ url, request }) => {
    const minutes = Number(url.searchParams.get('minutes') || '999');
    const useSSE = url.searchParams.get('sse') === '1';

    if (useSSE) {
        const stream = new ReadableStream({
            start(controller) {
                activeControllers.add(controller);
                if (!pollIntervalId) {
                    const poll = async () => {
                        try {
                            const payload = await getPayload(minutes);
                            lastPayload = payload;
                            const chunk = encoder.encode(`event: imu\n` + `data: ${JSON.stringify(payload)}\n\n`);
                            for (const c of Array.from(activeControllers)) {
                                try { c.enqueue(chunk); } catch { try { c.close(); } catch { } activeControllers.delete(c); }
                            }
                        } catch (e) {
                            const err = encoder.encode(`event: error\n` + `data: {"message":"Failed to compute IMU metrics"}\n\n`);
                            for (const c of Array.from(activeControllers)) {
                                try { c.enqueue(err); } catch { try { c.close(); } catch { } activeControllers.delete(c); }
                            }
                            console.error('IMU central poll error', e);
                        }
                    };
                    pollIntervalId = setInterval(poll, 2000);
                    poll();
                } else if (lastPayload) {
                    controller.enqueue(encoder.encode(`event: imu\n` + `data: ${JSON.stringify(lastPayload)}\n\n`));
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

    try {
        const payload = await getPayload(minutes);
        return json(payload);
    } catch (e: any) {
        console.error('IMU API error', e);
        throw error(500, `Failed to fetch IMU metrics: ${e.message}`);
    }
};
