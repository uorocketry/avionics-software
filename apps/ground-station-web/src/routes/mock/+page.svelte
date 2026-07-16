<script lang="ts">
	import { toastStore } from '$lib/stores/toastStore';
	import { onMount } from 'svelte';
	import {
		Grid,
		Row,
		Column,
		Tile,
		Button,
		NumberInput,
		InlineNotification,
		InlineLoading
	} from 'carbon-components-svelte';
	// onMount is fine for effects that need cleanup

	// --- State for the form inputs ---
	let time_stamp = $state(Math.floor(Date.now() / 1000));
	let accelerometer_x = $state(0.1);
	let accelerometer_y = $state(0.2);
	let accelerometer_z = $state(9.81);
	let gyroscope_alpha = $state(0); // Alpha for device orientation
	let gyroscope_beta = $state(0); // Beta
	let gyroscope_gamma = $state(0); // Gamma
	let temperature = $state(25.5);
	// Optional fields, can be left empty or have defaults
	let delta_velocity_x = $state(0);
	let delta_velocity_y = $state(0);
	let delta_velocity_z = $state(0);
	let delta_angle_x = $state(0);
	let delta_angle_y = $state(0);
	let delta_angle_z = $state(0);

	let isLoading = $state(false);
	let isListeningToSensors = $state(false);
	let sensorError = $state<string | null>(null);

	// --- Sensor Handling ---
	function roundToTwo(num: number | null | undefined): number {
		if (num === null || num === undefined) return 0; // Or handle as per your preference, e.g., return num
		return parseFloat(num.toFixed(2));
	}

	function handleDeviceMotion(event: DeviceMotionEvent) {
		time_stamp = Math.floor(Date.now() / 1000);
		if (event.accelerationIncludingGravity) {
			accelerometer_x = roundToTwo(event.accelerationIncludingGravity.x);
			accelerometer_y = roundToTwo(event.accelerationIncludingGravity.y);
			accelerometer_z = roundToTwo(event.accelerationIncludingGravity.z);
		}
		// event.acceleration for acceleration without gravity if needed
		// event.rotationRate for gyroscope if preferred over deviceorientation
		// event.interval for sample rate
	}

	function handleDeviceOrientation(event: DeviceOrientationEvent) {
		time_stamp = Math.floor(Date.now() / 1000);
		gyroscope_alpha = roundToTwo(event.alpha);
		gyroscope_beta = roundToTwo(event.beta);
		gyroscope_gamma = roundToTwo(event.gamma);
	}

	async function startSensorListeners() {
		sensorError = null;
		if (
			typeof DeviceMotionEvent !== 'undefined' &&
			typeof (DeviceMotionEvent as any).requestPermission === 'function'
		) {
			try {
				const motionPermission = await (DeviceMotionEvent as any).requestPermission();
				if (motionPermission !== 'granted') {
					sensorError = 'Motion sensor permission denied.';
					toastStore.error(sensorError);
					return;
				}
			} catch (err: any) {
				sensorError = `Error requesting motion permission: ${err.message}`;
				toastStore.error(sensorError);
				console.error(sensorError, err);
				return;
			}
		}
		if (
			typeof DeviceOrientationEvent !== 'undefined' &&
			typeof (DeviceOrientationEvent as any).requestPermission === 'function'
		) {
			try {
				const orientationPermission = await (DeviceOrientationEvent as any).requestPermission();
				if (orientationPermission !== 'granted') {
					sensorError = 'Orientation sensor permission denied.';
					toastStore.error(sensorError);
					return;
				}
			} catch (err: any) {
				sensorError = `Error requesting orientation permission: ${err.message}`;
				toastStore.error(sensorError);
				console.error(sensorError, err);
				return;
			}
		}

		if (window.DeviceMotionEvent) {
			window.addEventListener('devicemotion', handleDeviceMotion, true);
		} else {
			sensorError = 'Device Motion API not available.';
		}
		if (window.DeviceOrientationEvent) {
			window.addEventListener('deviceorientation', handleDeviceOrientation, true);
		} else {
			const currentError = sensorError ? sensorError + ' ' : '';
			sensorError = currentError + 'Device Orientation API not available.';
		}

		if (!sensorError) {
			isListeningToSensors = true;
			toastStore.info('Started listening to device sensors.');
		} else {
			toastStore.error(sensorError);
		}
	}

	function stopSensorListeners() {
		window.removeEventListener('devicemotion', handleDeviceMotion, true);
		window.removeEventListener('deviceorientation', handleDeviceOrientation, true);
		isListeningToSensors = false;
		toastStore.info('Stopped listening to device sensors.');
	}

	onMount(() => {
		// Cleanup listeners when component is unmounted
		return () => {
			if (isListeningToSensors) {
				stopSensorListeners();
			}
		};
	});

	async function handleSubmit() {
		isLoading = true;
		try {
			const payload = {
				time_stamp,
				status: 'VALID',
				// Using device orientation alpha, beta, gamma as direct substitutes for gyro x,y,z
				// This is a simplification; true gyroscope data (rotationRate) might be preferred
				// or a more complex conversion from orientation to angular velocity might be needed for strict accuracy.
				accelerometer_x: accelerometer_x || null,
				accelerometer_y: accelerometer_y || null,
				accelerometer_z: accelerometer_z || null,
				gyroscope_x: gyroscope_alpha || null, // Mapped alpha to x
				gyroscope_y: gyroscope_beta || null, // Mapped beta to y
				gyroscope_z: gyroscope_gamma || null, // Mapped gamma to z
				delta_velocity_x: delta_velocity_x || null,
				delta_velocity_y: delta_velocity_y || null,
				delta_velocity_z: delta_velocity_z || null,
				delta_angle_x: delta_angle_x || null,
				delta_angle_y: delta_angle_y || null,
				delta_angle_z: delta_angle_z || null,
				temperature: temperature || null
			};

			const response = await fetch('/mock/api', {
				method: 'POST',
				headers: {
					'Content-Type': 'application/json'
				},
				body: JSON.stringify(payload)
			});

			const result = await response.json();

			if (!response.ok) {
				throw new Error(result.message || `Failed to submit mock data: ${response.status}`);
			}

			toastStore.success(`Mock data inserted! Row ID: ${result.rowId}`);
			// Optionally reset form fields here if not listening to sensors
			if (!isListeningToSensors) {
				time_stamp = Math.floor(Date.now() / 1000);
			}
		} catch (err: any) {
			toastStore.error(err.message || 'An unknown error occurred.');
			console.error('Submission error:', err);
		} finally {
			isLoading = false;
		}
	}

	// Function to add unique IDs for accessibility
	function id(name: string) {
		return `mock-${name}`;
	}
</script>

<Grid padding={true} class="rgs-route">
	<Row>
		<Column>
			<h1>Mock SBG IMU Data Inserter</h1>
		</Column>
	</Row>

	<Row>
		<Column>
			<Tile>
				<h2 class="cds--type-productive-heading-03">Device Sensors Control</h2>
				{#if sensorError}
					<InlineNotification kind="warning" title="Sensor Error" subtitle={sensorError} />
				{/if}
				{#if !isListeningToSensors}
					<Button kind="tertiary" on:click={startSensorListeners}>Use Device Sensors</Button>
				{:else}
					<Button kind="danger" on:click={stopSensorListeners}>Stop Device Sensors</Button>
				{/if}
				<p class="cds--label-01" style="margin-top: .5rem;">
					Uses device orientation (alpha, beta, gamma) for gyroscope fields; may require
					permissions.
				</p>
			</Tile>
		</Column>
	</Row>

	<Row>
		<Column>
			<Tile>
				<form
					onsubmit={(e) => {
						e.preventDefault();
						handleSubmit();
					}}
				>
					<Row>
						<Column sm={16} md={8} lg={8}>
							<NumberInput
								label="Timestamp (Unix seconds)"
								bind:value={time_stamp}
								readonly={isListeningToSensors}
							/>
						</Column>
					</Row>

					<h3 class="cds--type-productive-heading-03" style="margin-top: .75rem;">
						Accelerometers (m/s²)
					</h3>
					<Row>
						<Column sm={16} md={5} lg={5}
							><NumberInput
								label="Accel X"
								bind:value={accelerometer_x}
								step={0.01}
								readonly={isListeningToSensors}
							/></Column
						>
						<Column sm={16} md={5} lg={5}
							><NumberInput
								label="Accel Y"
								bind:value={accelerometer_y}
								step={0.01}
								readonly={isListeningToSensors}
							/></Column
						>
						<Column sm={16} md={6} lg={6}
							><NumberInput
								label="Accel Z"
								bind:value={accelerometer_z}
								step={0.01}
								readonly={isListeningToSensors}
							/></Column
						>
					</Row>

					<h3 class="cds--type-productive-heading-03" style="margin-top: .75rem;">
						Gyroscopes (α, β, γ)
					</h3>
					<Row>
						<Column sm={16} md={5} lg={5}
							><NumberInput
								label="Gyro X (Alpha °)"
								bind:value={gyroscope_alpha}
								step={0.01}
								readonly={isListeningToSensors}
							/></Column
						>
						<Column sm={16} md={5} lg={5}
							><NumberInput
								label="Gyro Y (Beta °)"
								bind:value={gyroscope_beta}
								step={0.01}
								readonly={isListeningToSensors}
							/></Column
						>
						<Column sm={16} md={6} lg={6}
							><NumberInput
								label="Gyro Z (Gamma °)"
								bind:value={gyroscope_gamma}
								step={0.01}
								readonly={isListeningToSensors}
							/></Column
						>
					</Row>

					<h3 class="cds--type-productive-heading-03" style="margin-top: .75rem;">
						Delta Velocity (m/s)
					</h3>
					<Row>
						<Column sm={16} md={5} lg={5}
							><NumberInput label="ΔV X" bind:value={delta_velocity_x} step={0.01} /></Column
						>
						<Column sm={16} md={5} lg={5}
							><NumberInput label="ΔV Y" bind:value={delta_velocity_y} step={0.01} /></Column
						>
						<Column sm={16} md={6} lg={6}
							><NumberInput label="ΔV Z" bind:value={delta_velocity_z} step={0.01} /></Column
						>
					</Row>

					<h3 class="cds--type-productive-heading-03" style="margin-top: .75rem;">
						Delta Angle (rad)
					</h3>
					<Row>
						<Column sm={16} md={5} lg={5}
							><NumberInput label="Δθ X" bind:value={delta_angle_x} step={0.01} /></Column
						>
						<Column sm={16} md={5} lg={5}
							><NumberInput label="Δθ Y" bind:value={delta_angle_y} step={0.01} /></Column
						>
						<Column sm={16} md={6} lg={6}
							><NumberInput label="Δθ Z" bind:value={delta_angle_z} step={0.01} /></Column
						>
					</Row>

					<Row>
						<Column sm={16} md={6} lg={6}
							><NumberInput
								label="Temperature (°C)"
								bind:value={temperature}
								step={0.1}
								readonly={isListeningToSensors}
							/></Column
						>
					</Row>

					<Row>
						<Column>
							<Button type="submit" kind="primary" disabled={isLoading}>
								{#if isLoading}
									<InlineLoading description="Submitting..." />
								{:else}
									Insert Mock IMU Data
								{/if}
							</Button>
						</Column>
					</Row>
				</form>
			</Tile>
		</Column>
	</Row>
</Grid>
