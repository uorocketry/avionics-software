import { getDbClient } from '$lib/server/db'; // Assuming this path is correct
import { json, error as svelteKitError } from '@sveltejs/kit';
import type { RequestHandler } from './$types';

// Define the expected payload structure from the client
interface MockSbgImuPayload {
	time_stamp: number;
	status: string;
	accelerometer_x?: number | null;
	accelerometer_y?: number | null;
	accelerometer_z?: number | null;
	gyroscope_x?: number | null;
	gyroscope_y?: number | null;
	gyroscope_z?: number | null;
	delta_velocity_x?: number | null;
	delta_velocity_y?: number | null;
	delta_velocity_z?: number | null;
	delta_angle_x?: number | null;
	delta_angle_y?: number | null;
	delta_angle_z?: number | null;
	temperature?: number | null;
}

export const POST: RequestHandler = async ({ request }) => {
	try {
		const payload = await request.json() as MockSbgImuPayload;

		// Basic validation (can be more extensive)
		if (!payload.time_stamp || !payload.status) {
			return svelteKitError(400, 'Missing required fields: time_stamp and status');
		}

		const db = getDbClient();

		const result = await db.execute({
			sql: `INSERT INTO SbgImu (
                    time_stamp, status,
                    accelerometer_x, accelerometer_y, accelerometer_z,
                    gyroscope_x, gyroscope_y, gyroscope_z,
                    delta_velocity_x, delta_velocity_y, delta_velocity_z,
                    delta_angle_x, delta_angle_y, delta_angle_z,
                    temperature
                ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?);`,
			args: [
				payload.time_stamp,
				payload.status,
				payload.accelerometer_x ?? null,
				payload.accelerometer_y ?? null,
				payload.accelerometer_z ?? null,
				payload.gyroscope_x ?? null,
				payload.gyroscope_y ?? null,
				payload.gyroscope_z ?? null,
				payload.delta_velocity_x ?? null,
				payload.delta_velocity_y ?? null,
				payload.delta_velocity_z ?? null,
				payload.delta_angle_x ?? null,
				payload.delta_angle_y ?? null,
				payload.delta_angle_z ?? null,
				payload.temperature ?? null,
			],
		});

		return json({
			message: 'Mock IMU data inserted successfully',
			rowId: result.lastInsertRowid ? String(result.lastInsertRowid) : null
		}, { status: 201 });

	} catch (e: any) {
		console.error("Error inserting mock IMU data:", e);
		// Check for specific error types if needed, e.g., JSON parsing errors
		if (e instanceof SyntaxError) {
			return svelteKitError(400, 'Invalid JSON payload');
		}
		return svelteKitError(500, e.message || 'Failed to insert mock IMU data');
	}
};