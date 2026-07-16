// ground_altitude = ($air?.altitude ?? 0) - $flightDirector.relativeAltitude;
// export const ground_altitude: Readable<number> = derived(
// 	[flightDirector, air],
// 	([$flightDirector, $air]) => {
// 		return ($air?.altitude ?? 0) - ($flightDirector?.relativeAltitude ?? 0);
// 	}
// );

// $: total_traveled_distance = haversineDistance($rocketPosition, $launchPoint);
// export const total_traveled_distance: Readable<number> = derived(
// 	[rocketPos, flightDirector],
// 	([$rocketPos, $flightDirector]) => {
// 		return haversineDistance(
// 			{ lat: $rocketPos?.latitude ?? 0, lng: $rocketPos?.longitude ?? 0 },
// 			{ lat: $flightDirector?.latitude ?? 0, lng: $flightDirector?.longitude ?? 0 }
// 		);
// 	}
// );

// For reference:
// function calcGForce(vf: number, t: number) {
// 	return vf / (t * 9.81);
// }

// FIXME: The G-Force calculation is wrong since it's not given a proper time delta
// Should also be calculated on database view (?)

// export const g_force: Readable<number> = derived(nav, ($nav) => {
// 	if ($nav.velocity) {
// 		return calcGForce($nav.velocity[1], 0.1);
// 	}
// 	return 0;
// });

// const _g_force = 0;
// export const max_g_force: Readable<number> = derived(g_force, ($g_force) => {
// 	return max($g_force, _g_force);
// });
