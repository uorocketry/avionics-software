import { getDbClient } from '$lib/server/db';
import { listServices } from '$lib/server/services';
import type { FlightTransition, OverviewSnapshot } from '$lib/realtime/types';
import { json } from '@sveltejs/kit';
import type { RequestHandler } from './$types';

const QNH_KPA = 102.5;

function altitudeFromPressure(pressureKpa: number): number {
	return 44330 * (1 - Math.pow(pressureKpa / QNH_KPA, 0.190263));
}

function numberOrNull(value: unknown): number | null {
	if (value === null || value === undefined) return null;
	const number = Number(value);
	return Number.isFinite(number) ? number : null;
}

async function getSnapshot(): Promise<OverviewSnapshot> {
	const db = getDbClient();
	const [
		stateRows,
		eventRows,
		baroRows,
		gpsRows,
		velocityRows,
		imuRows,
		radioRows,
		frameRows,
		lossRows,
		transitionRows,
		pathRows,
		serviceResult
	] = await Promise.all([
		db.execute({
			sql: `SELECT ps.id, ps.state AS value, rf.timestamp_epoch AS ts FROM PhoenixState ps JOIN RadioFrame rf ON rf.data_type = 'PhoenixState' AND rf.data_id = ps.id ORDER BY rf.timestamp_epoch DESC LIMIT 1`,
			args: []
		}),
		db.execute({
			sql: `SELECT pe.id, pe.event AS value, rf.timestamp_epoch AS ts FROM PhoenixEvent pe JOIN RadioFrame rf ON rf.data_type = 'PhoenixEvent' AND rf.data_id = pe.id ORDER BY rf.timestamp_epoch DESC LIMIT 1`,
			args: []
		}),
		db.execute({
			sql: `SELECT b.pressure_kpa, rf.timestamp_epoch AS ts FROM Barometer b JOIN RadioFrame rf ON rf.data_type = 'Barometer' AND rf.data_id = b.id ORDER BY rf.timestamp_epoch DESC LIMIT 1`,
			args: []
		}),
		db.execute({
			sql: `SELECT p.altitude, p.latitude, p.longitude, p.num_sv_used, rf.timestamp_epoch AS ts FROM SbgGpsPos p JOIN RadioFrame rf ON rf.data_type = 'SbgGpsPos' AND rf.data_id = p.id WHERE p.latitude IS NOT NULL AND p.longitude IS NOT NULL ORDER BY rf.timestamp_epoch DESC LIMIT 1`,
			args: []
		}),
		db.execute({
			sql: `SELECT v.velocity_down, rf.timestamp_epoch AS ts FROM SbgGpsVel v JOIN RadioFrame rf ON rf.data_type = 'SbgGpsVel' AND rf.data_id = v.id ORDER BY rf.timestamp_epoch DESC LIMIT 1`,
			args: []
		}),
		db.execute({
			sql: `SELECT i.accelerometer_x, i.accelerometer_y, i.accelerometer_z, rf.timestamp_epoch AS ts FROM SbgImu i JOIN RadioFrame rf ON rf.data_type = 'SbgImu' AND rf.data_id = i.id ORDER BY rf.timestamp_epoch DESC LIMIT 1`,
			args: []
		}),
		db.execute({
			sql: `SELECT timestamp, rssi FROM RadioMetrics ORDER BY timestamp DESC LIMIT 1`,
			args: []
		}),
		db.execute({
			sql: `SELECT COUNT(*) AS total, SUM(CASE WHEN timestamp_epoch >= strftime('%s','now') - 10 THEN 1 ELSE 0 END) AS recent FROM RadioFrame`,
			args: []
		}),
		db.execute({
			sql: `SELECT COALESCE(SUM(packets_lost), 0) AS lost FROM RadioMetrics`,
			args: []
		}),
		db.execute({
			sql: `SELECT id, kind, value, ts FROM (SELECT ps.id, 'State' AS kind, ps.state AS value, rf.timestamp_epoch AS ts FROM PhoenixState ps JOIN RadioFrame rf ON rf.data_type = 'PhoenixState' AND rf.data_id = ps.id UNION ALL SELECT pe.id, 'Event' AS kind, pe.event AS value, rf.timestamp_epoch AS ts FROM PhoenixEvent pe JOIN RadioFrame rf ON rf.data_type = 'PhoenixEvent' AND rf.data_id = pe.id) ORDER BY ts DESC LIMIT 12`,
			args: []
		}),
		db.execute({
			sql: `SELECT rf.timestamp_epoch AS ts, p.latitude AS lat, p.longitude AS lon FROM SbgGpsPos p JOIN RadioFrame rf ON rf.data_type = 'SbgGpsPos' AND rf.data_id = p.id WHERE p.latitude IS NOT NULL AND p.longitude IS NOT NULL AND p.latitude != 0 AND p.longitude != 0 ORDER BY rf.timestamp_epoch DESC LIMIT 120`,
			args: []
		}),
		listServices()
			.then((services) => ({ services, available: true }))
			.catch(() => ({ services: [], available: false }))
	]);

	const state: any = stateRows.rows[0];
	const event: any = eventRows.rows[0];
	const baro: any = baroRows.rows[0];
	const gps: any = gpsRows.rows[0];
	const velocity: any = velocityRows.rows[0];
	const imu: any = imuRows.rows[0];
	const radio: any = radioRows.rows[0];
	const frames: any = frameRows.rows[0] ?? {};
	const losses: any = lossRows.rows[0] ?? {};
	const totalFrames = Number(frames.total ?? 0);
	const totalLost = Number(losses.lost ?? 0);
	const gpsTimestamp = numberOrNull(gps?.ts);
	const imuValues = [imu?.accelerometer_x, imu?.accelerometer_y, imu?.accelerometer_z].map(
		numberOrNull
	);
	const acceleration = imuValues.every((value) => value !== null)
		? Math.hypot(...(imuValues as number[]))
		: null;

	const transition = (row: any): FlightTransition => ({
		id: Number(row.id),
		kind: row.kind as 'State' | 'Event',
		value: String(row.value),
		timestamp: numberOrNull(row.ts)
	});

	return {
		generatedAt: Date.now(),
		flight: {
			state: state
				? {
						id: Number(state.id),
						kind: 'State',
						value: String(state.value),
						timestamp: numberOrNull(state.ts)
					}
				: null,
			event: event
				? {
						id: Number(event.id),
						kind: 'Event',
						value: String(event.value),
						timestamp: numberOrNull(event.ts)
					}
				: null,
			altitudeM: {
				value: baro ? altitudeFromPressure(Number(baro.pressure_kpa)) : null,
				updatedAt: numberOrNull(baro?.ts)
			},
			gnssAltitudeM: { value: numberOrNull(gps?.altitude), updatedAt: gpsTimestamp },
			verticalVelocityMps: {
				value: velocity?.velocity_down == null ? null : -Number(velocity.velocity_down),
				updatedAt: numberOrNull(velocity?.ts)
			},
			accelerationMps2: { value: acceleration, updatedAt: numberOrNull(imu?.ts) },
			position: {
				value: gps ? { lat: Number(gps.latitude), lon: Number(gps.longitude) } : null,
				updatedAt: gpsTimestamp
			},
			satellites: { value: numberOrNull(gps?.num_sv_used), updatedAt: gpsTimestamp }
		},
		radio: {
			updatedAt: numberOrNull(radio?.timestamp),
			rssiDbm: numberOrNull(radio?.rssi),
			framesPerSecond: Number(frames.recent ?? 0) / 10,
			packetLossPercent: totalFrames > 0 ? (totalLost / totalFrames) * 100 : 0,
			totalFrames
		},
		services: {
			updatedAt: Date.now(),
			running: serviceResult.services.filter((service) => service.status === 'running').length,
			total: serviceResult.services.length,
			degraded: serviceResult.services
				.filter((service) => service.status !== 'running')
				.map((service) => service.name),
			available: serviceResult.available
		},
		recentTransitions: transitionRows.rows.map(transition),
		trajectory: [...pathRows.rows]
			.reverse()
			.map((row: any) => ({
				timestamp: Number(row.ts),
				lat: Number(row.lat),
				lon: Number(row.lon)
			}))
	};
}

export const GET: RequestHandler = async ({ url, request }) => {
	if (url.searchParams.get('sse') !== '1') return json(await getSnapshot());

	let interval: ReturnType<typeof setInterval>;
	let closed = false;
	const encoder = new TextEncoder();
	const stream = new ReadableStream({
		start(controller) {
			const push = async () => {
				if (closed) return;
				try {
					const snapshot = await getSnapshot();
					controller.enqueue(
						encoder.encode(`event: overview\ndata: ${JSON.stringify(snapshot)}\n\n`)
					);
				} catch (error) {
					console.error('Overview update failed', error);
					controller.enqueue(
						encoder.encode(`event: error\ndata: {"message":"Overview unavailable"}\n\n`)
					);
				}
			};
			void push();
			interval = setInterval(push, 1_000);
			request.signal.addEventListener('abort', () => {
				closed = true;
				clearInterval(interval);
				try {
					controller.close();
				} catch {
					/* already closed */
				}
			});
		},
		cancel() {
			closed = true;
			clearInterval(interval);
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
