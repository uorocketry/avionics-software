<script lang="ts">
	import { onMount, onDestroy } from 'svelte';
	import {
		Grid,
		Row,
		Column,
		Tile,
		Button,
		InlineNotification,
		InlineLoading,
		Tag
	} from 'carbon-components-svelte';
	import { LineChart } from '@carbon/charts-svelte';
	import type { ChartTabularData, LineChartOptions } from '@carbon/charts-svelte';
	import { ScaleTypes } from '@carbon/charts';
	import { findSetting } from '$lib/common/settings';
	import { get } from 'svelte/store';

	type Stats = {
		peak_accel_mps2: number;
		peak_accel_from_dvdt_mps2: number;
		peak_gyro_rads: number;
		peak_rate_from_dadt_rads: number;
		cumulative_delta_v_mps: number;
		cumulative_rotation_turns: number;
		latest_temperature_c: number | null;
	};

	let accelData: ChartTabularData = $state([]);
	let gyroData: ChartTabularData = $state([]);
	let stats: Stats | null = $state(null);
	let connected = $state(false);
	let loading = $state(false);
	let error: string | null = $state(null);
	let eventSource: EventSource | null = null;

	const imuMinutesSetting = findSetting('flight.imuMinutes');
	let minutesBack = $state(
		imuMinutesSetting && imuMinutesSetting.valueDescription === 'number'
			? Number(get(imuMinutesSetting.value))
			: 10
	);

	const accelOptions: LineChartOptions = {
		title: 'Acceleration Magnitudes',
		axes: {
			left: { title: 'm/s²', mapsTo: 'value', scaleType: ScaleTypes.LINEAR },
			bottom: { title: 'Time', mapsTo: 'date', scaleType: ScaleTypes.TIME }
		},
		curve: 'curveMonotoneX',
		legend: { enabled: true },
		grid: { x: { enabled: true }, y: { enabled: true } },
		height: '350px',
		toolbar: { enabled: false }
	};

	const gyroOptions: LineChartOptions = {
		title: 'Angular Rate Magnitudes',
		axes: {
			left: { title: 'rad/s', mapsTo: 'value', scaleType: ScaleTypes.LINEAR },
			bottom: { title: 'Time', mapsTo: 'date', scaleType: ScaleTypes.TIME }
		},
		curve: 'curveMonotoneX',
		legend: { enabled: true },
		grid: { x: { enabled: true }, y: { enabled: true } },
		height: '350px',
		toolbar: { enabled: false }
	};

	async function refresh() {
		loading = true;
		try {
			const res = await fetch(`/imu/api?minutes=${encodeURIComponent(minutesBack)}`);
			if (!res.ok) throw new Error(await res.text());
			const body = await res.json();
			accelData = body.series?.accel ?? [];
			gyroData = body.series?.gyro ?? [];
			stats = body.stats ?? null;
			error = null;
		} catch (e: any) {
			error = e.message || 'Failed to fetch IMU metrics';
		} finally {
			loading = false;
		}
	}

	function startSSE() {
		if (eventSource) eventSource.close();
		eventSource = new EventSource(`/imu/api?sse=1&minutes=${encodeURIComponent(minutesBack)}`);
		connected = false;
		loading = true;
		eventSource.onopen = () => {
			connected = true;
			loading = false;
			error = null;
		};
		eventSource.addEventListener('imu', (event) => {
			try {
				const data = JSON.parse(event.data);
				accelData = data.series?.accel ?? [];
				gyroData = data.series?.gyro ?? [];
				stats = data.stats ?? null;
			} catch {}
		});
		eventSource.onerror = () => {
			connected = false;
			loading = false;
		};
	}

	function stopSSE() {
		if (eventSource) {
			eventSource.close();
			eventSource = null;
		}
		connected = false;
	}

	onMount(() => {
		startSSE();
	});
	onDestroy(() => {
		stopSSE();
	});
</script>

<Grid padding={true} class="rgs-route telemetry-route">
	<Row>
		<Column>
			<Tile>
				<div style="display: flex; justify-content: space-between; align-items: center;">
					<h2>IMU Metrics</h2>
					<div>
						<Button
							kind="secondary"
							size="default"
							on:click={refresh}
							disabled={loading}
							title="Refresh"
						>
							{#if loading}
								<InlineLoading description="Refreshing..." />
							{:else}
								Refresh
							{/if}
						</Button>
						{#if connected}
							<Button
								kind="danger"
								size="default"
								on:click={stopSSE}
								style="margin-left: 0.5rem;"
								title="Stop Live">Stop Live</Button
							>
						{:else}
							<Button
								kind="primary"
								size="default"
								on:click={startSSE}
								style="margin-left: 0.5rem;"
								title="Start Live">Start Live</Button
							>
						{/if}
					</div>
				</div>

				{#if error}
					<InlineNotification
						kind="error"
						title="Error"
						subtitle={error}
						hideCloseButton
						style="margin: 1rem 0;"
					/>
				{/if}

				<!-- Summary tiles -->
				<Row style="margin: 1rem 0;">
					<Column>
						<Tile>
							<h3>Peak Accel</h3>
							<div style="font-size: 2rem; font-weight: bold;">
								{stats ? stats.peak_accel_mps2.toFixed(2) : '—'} m/s²
							</div>
							<Tag type="blue" size="sm">from sensor</Tag>
						</Tile>
					</Column>
					<Column>
						<Tile>
							<h3>Peak Accel (Δv/Δt)</h3>
							<div style="font-size: 2rem; font-weight: bold;">
								{stats ? stats.peak_accel_from_dvdt_mps2.toFixed(2) : '—'} m/s²
							</div>
							<Tag type="teal" size="sm">from integration</Tag>
						</Tile>
					</Column>
					<Column>
						<Tile>
							<h3>Peak |ω|</h3>
							<div style="font-size: 2rem; font-weight: bold;">
								{stats ? stats.peak_gyro_rads.toFixed(2) : '—'} rad/s
							</div>
							<Tag type="green" size="sm">gyro</Tag>
						</Tile>
					</Column>
					<Column>
						<Tile>
							<h3>Peak Rate (Δθ/Δt)</h3>
							<div style="font-size: 2rem; font-weight: bold;">
								{stats ? stats.peak_rate_from_dadt_rads.toFixed(2) : '—'} rad/s
							</div>
							<Tag type="gray" size="sm">from integration</Tag>
						</Tile>
					</Column>
				</Row>

				<Row style="margin: 1rem 0;">
					<Column>
						<Tile>
							<h3>Cumulative Δv</h3>
							<div style="font-size: 2rem; font-weight: bold;">
								{stats ? stats.cumulative_delta_v_mps.toFixed(2) : '—'} m/s
							</div>
						</Tile>
					</Column>
					<Column>
						<Tile>
							<h3>Rotation</h3>
							<div style="font-size: 2rem; font-weight: bold;">
								{stats ? stats.cumulative_rotation_turns.toFixed(2) : '—'} turns
							</div>
						</Tile>
					</Column>
					<Column>
						<Tile>
							<h3>IMU Temperature</h3>
							<div style="font-size: 2rem; font-weight: bold;">
								{stats && stats.latest_temperature_c != null
									? stats.latest_temperature_c.toFixed(1)
									: '—'} °C
							</div>
						</Tile>
					</Column>
				</Row>

				<Tile>
					<LineChart data={accelData} options={accelOptions} />
				</Tile>
				<Tile>
					<LineChart data={gyroData} options={gyroOptions} />
				</Tile>
			</Tile>
		</Column>
	</Row>
</Grid>
