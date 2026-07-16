import { getDbClient } from '$lib/server/db';
import { json, error } from '@sveltejs/kit';
import type { RequestHandler } from './$types';

function nowEpoch(): number {
    return Math.floor(Date.now() / 1000);
}

function clampAltitudeMeters(v: unknown): number {
    const n = Number(v);
    if (!Number.isFinite(n)) return 0;
    return Math.max(0, Math.min(5000, Math.round(n)));
}

function pressureKpaFromAltitude(altitudeMeters: number, qnhKpa: number = 102.5): number {
    // Invert standard atmosphere: p = p0 * (1 - h/44330)^(1/0.190263)
    const ratio = 1 - Math.max(0, altitudeMeters) / 44330;
    const p = qnhKpa * Math.pow(Math.max(1e-6, ratio), 1 / 0.190263);
    return p;
}

function normalizeLatLon(v: unknown, min: number, max: number): number {
    const n = Number(v);
    if (!Number.isFinite(n)) throw error(400, 'Invalid coordinate');
    if (n < min || n > max) throw error(400, 'Coordinate out of range');
    return n;
}

export const POST: RequestHandler = async ({ request }) => {
    const db = getDbClient();
    const body = await request.json().catch(() => null);
    if (!body) throw error(400, 'Invalid JSON');

    const latitude = normalizeLatLon(body.lat, -90, 90);
    const longitude = normalizeLatLon(body.lon, -180, 180);
    const altitude_m = clampAltitudeMeters(body.alt);

    const ts = nowEpoch();

    const transaction = await db.transaction("write");
    try {
        // Insert SbgGpsPos
        const gpsRes = await transaction.execute({
            sql: `INSERT INTO SbgGpsPos (latitude, longitude, time_of_week, undulation, altitude, time_stamp, status, latitude_accuracy, longitude_accuracy, altitude_accuracy, num_sv_used, base_station_id, differential_age)
			VALUES (?, ?, 0, 0.0, ?, ?, 'OK', 0.0, 0.0, 0.0, 0, 0, 0)`,
            args: [latitude, longitude, altitude_m, ts]
        });
        const gpsId = Number(gpsRes.lastInsertRowid ?? 0);

        // Insert SbgEkfNav
        const ekfRes = await transaction.execute({
            sql: `INSERT INTO SbgEkfNav (status, velocity_north, velocity_east, velocity_down, velocity_std_dev_north, velocity_std_dev_east, velocity_std_dev_down, position_latitude, position_longitude, position_altitude, position_std_dev_latitude, position_std_dev_longitude, position_std_dev_altitude, time_stamp, undulation)
			VALUES ('OK', 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, ?, ?, ?, 0.0, 0.0, 0.0, ?, 0.0)`,
            args: [latitude, longitude, altitude_m, ts]
        });

        // Insert SbgAir
        const airRes = await transaction.execute({
            sql: `INSERT INTO SbgAir (time_stamp, status, pressure_abs, altitude, pressure_diff, true_airspeed, air_temperature)
			VALUES (?, 'OK', ?, ?, NULL, NULL, NULL)`,
            args: [ts, Math.round(pressureKpaFromAltitude(altitude_m) * 1000), altitude_m]
        });
        const airId = Number(airRes.lastInsertRowid ?? 0);

        // Insert Barometer
        const baroRes = await transaction.execute({
            sql: `INSERT INTO Barometer (pressure_kpa, temperature_celsius)
			VALUES (?, NULL)`,
            args: [pressureKpaFromAltitude(altitude_m)]
        });
        const baroId = Number(baroRes.lastInsertRowid ?? 0);

        // Create RadioFrame rows for each insert so downstream queries can join on timestamps
        const isoTs = new Date(ts * 1000).toISOString();
        await transaction.execute({
            sql: `INSERT INTO RadioFrame (timestamp, timestamp_epoch, node, data_type, data_id, millis_since_start)
                  VALUES (?, ?, 'Gps', 'SbgGpsPos', ?, NULL)`,
            args: [isoTs, ts, gpsId]
        });
        await transaction.execute({
            sql: `INSERT INTO RadioFrame (timestamp, timestamp_epoch, node, data_type, data_id, millis_since_start)
                  VALUES (?, ?, 'Sbg', 'SbgAir', ?, NULL)`,
            args: [isoTs, ts, airId]
        });
        await transaction.execute({
            sql: `INSERT INTO RadioFrame (timestamp, timestamp_epoch, node, data_type, data_id, millis_since_start)
                  VALUES (?, ?, 'Barometer', 'Barometer', ?, NULL)`,
            args: [isoTs, ts, baroId]
        });

        // Commit the transaction
        await transaction.commit();

        return json({
            success: true,
            inserted: {
                gps: gpsRes.rowsAffected,
                ekf: ekfRes.rowsAffected,
                air: airRes.rowsAffected,
                baro: baroRes.rowsAffected
            },
            lat: latitude,
            lon: longitude,
            alt: altitude_m
        });
    } catch (e: any) {
        // Rollback on error
        try {
            await transaction.rollback();
        } catch (rollbackError) {
            console.error('Rollback failed:', rollbackError);
        }
        console.error('Coord-mock insert failed', e);
        throw error(500, e?.message ?? 'Insert failed');
    } finally {
        // Always close the transaction
        transaction.close();
    }
};
