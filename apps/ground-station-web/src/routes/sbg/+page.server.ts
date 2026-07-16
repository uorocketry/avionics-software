import { getDbClient } from '$lib/server/db';
import type { Row } from '@libsql/client';
import type { PageServerLoad } from './$types';

// Helper function to get the latest row from a table
async function getLatestRow(tableName: string): Promise<Row | null> {
	const db = getDbClient();
	try {
		const result = await db.execute({
			sql: `SELECT * FROM ${tableName} ORDER BY time_stamp DESC LIMIT 1`,
			args: []
		});
		return result.rows.length > 0 ? result.rows[0] : null;
	} catch (e: any) {
		console.error(`Error fetching latest row from ${tableName}:`, e);
		// Depending on requirements, you might want to throw, return null, or a specific error object
		return null; // Return null on error for graceful handling
	}
}

export const load: PageServerLoad = async () => {
	console.log("Loading initial SBG data for /sbg page...");

	// Fetch latest data from relevant tables in parallel
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

	console.log("Finished loading initial SBG data.");

	// Return the data in a structured way
	// Handle potential nulls if a table is empty or an error occurred
	return {
		sbgData: {
			utcTime: latestUtcTime ?? {},
			air: latestAir ?? {},
			ekfQuat: latestEkfQuat ?? {},
			ekfNav: latestEkfNav ?? {},
			imu: latestImu ?? {},
			gpsVel: latestGpsVel ?? {},
			gpsPos: latestGpsPos ?? {},
		}
	};
};