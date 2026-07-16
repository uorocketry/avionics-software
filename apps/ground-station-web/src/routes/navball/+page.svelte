<script lang="ts">
	import { Grid, Row, Column, Tile, InlineNotification, Tag } from 'carbon-components-svelte';
	import { Quaternion, Vector3, MathUtils, Euler } from 'three';
	import NavBall from '$lib/components/Navball/NavBall.svelte';
	import HeadingCompass from '$lib/components/HeadingCompass/HeadingCompass.svelte';
	import { onMount } from 'svelte';

	// State for IMU data
	let imuData = $state<any>(null);
	let error = $state<string | null>(null);

	// Persistent orientation quaternion for the navball
	let targetRotation = $state(new Quaternion());
	let lastTimestamp: number | null = $state(null);
	// Get the pitch and roll from the quaternion
	let upTransformed = $derived(new Vector3(0, 1, 0).applyQuaternion(targetRotation));
	let pitch = $derived(MathUtils.radToDeg(Math.asin(upTransformed.y)));
	let heading = $derived(
		(MathUtils.radToDeg(Math.atan2(upTransformed.x, upTransformed.z)) + 360 + 90) % 360
	);
	let transformed = $derived(new Vector3(0, 0, -1).applyQuaternion(targetRotation));
	let roll = $derived(MathUtils.radToDeg(Math.atan2(transformed.x, transformed.z) + Math.PI));

	// Function to update rotation from EKF quaternion (SbgEkfQuat)
	function updateRotationFromImu(data: any) {
		if (!data) return;
		const qw = typeof data.quaternion_w === 'number' ? data.quaternion_w : 1;
		const qx = typeof data.quaternion_x === 'number' ? data.quaternion_x : 0;
		const qy = typeof data.quaternion_y === 'number' ? data.quaternion_y : 0;
		const qz = typeof data.quaternion_z === 'number' ? data.quaternion_z : 0;
		// three.js expects (x, y, z, w)
		targetRotation = new Quaternion(qx, qy, qz, qw).normalize();
	}

	// Set up SSE
	let eventSource: EventSource;
	onMount(() => {
		eventSource = new EventSource('/navball/api');

		eventSource.onmessage = (event) => {
			try {
				imuData = JSON.parse(event.data);
				updateRotationFromImu(imuData);
				error = null; // Clear previous errors on successful message
			} catch (e: any) {
				error = `Error parsing IMU data: ${e.message}`;
				console.error('Error parsing IMU data:', e);
			}
		};

		eventSource.onerror = (err) => {
			console.error('EventSource failed:', err);
			error = 'Connection to server lost. Attempting to reconnect...';
			// EventSource attempts to reconnect automatically by default
			// You could close it here if you want to stop retrying: eventSource.close();
		};

		return () => {
			if (eventSource) {
				eventSource.close();
			}
		};
	});
</script>

<Grid padding={true} class="rgs-route">
	<Row>
		<Column sm={16} md={12} lg={12}>
			<Tile>
				<div style="position: relative; height: 60vh;">
					{#if error}
						<InlineNotification
							kind="error"
							title="Navball Error"
							subtitle={error}
							hideCloseButton
						/>
					{/if}
					<div style="position:absolute; inset:0;">
						<NavBall {targetRotation} />
					</div>
					<div style="position:absolute; inset:0; pointer-events:none;">
						<HeadingCompass heading={(heading * Math.PI) / 180} />
					</div>
				</div>
			</Tile>
		</Column>
		<Column sm={16} md={4} lg={4}>
			<Tile>
				<h2>IMU Readout</h2>
				<p><strong>Roll:</strong> <Tag>{roll.toFixed(2)}°</Tag></p>
				<p><strong>Pitch:</strong> <Tag>{pitch.toFixed(2)}°</Tag></p>
				<p>
					<strong>Pointing:</strong>
					<Tag type={pitch > 0 ? 'green' : 'red'}>{pitch > 0 ? 'Up' : 'Down'}</Tag>
				</p>
				{#if imuData}
					<p><strong>Status:</strong> <Tag type="blue">{imuData.status}</Tag></p>
				{/if}
			</Tile>
		</Column>
	</Row>
</Grid>
