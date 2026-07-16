<script lang="ts">
	import type { PageData } from './$types';
	import { onMount, untrack } from 'svelte';
	import {
		Grid,
		Row,
		Column,
		Tile,
		Loading,
		Tag,
		Dropdown,
		InlineLoading,
		StructuredList,
		StructuredListBody,
		StructuredListCell,
		StructuredListHead,
		StructuredListRow,
		Header
	} from 'carbon-components-svelte';
	import { LineChart, ScaleTypes, type LineChartOptions } from '@carbon/charts-svelte';
	import { carbonTheme } from '$lib/common/theme';
	import { get } from 'svelte/store';
	import '@carbon/charts-svelte/styles.min.css';
	import { Heading } from 'carbon-icons-svelte';
	import { toastStore } from '$lib/stores/toastStore';

	let { data } = $props<{ data: PageData }>();

	// Snapshot polling state
	let sbgData: any = $state(untrack(() => data.sbgData));
	let lastUpdated = $state(new Date());
	let isLoading = $state(false);
	let error = $state<string | null>(null);

	// Metric chart state
	const METRIC_OPTIONS = [
		{ id: 'air_altitude', text: 'Air Altitude' },
		{ id: 'air_pressure', text: 'Air Pressure (abs/diff)' },
		{ id: 'air_true_airspeed', text: 'True Airspeed' },
		{ id: 'ekf_altitude', text: 'EKF Altitude' },
		{ id: 'gpsvel_velocity', text: 'GPS Velocity (N/E/D)' },
		{ id: 'gpsvel_course', text: 'GPS Course' },
		{ id: 'imu_temperature', text: 'IMU Temperature' },
		{ id: 'imu_accel', text: 'IMU Acceleration (X/Y/Z)' },
		{ id: 'imu_gyro', text: 'IMU Gyroscope (X/Y/Z)' }
	];

	let selectedMetric = $state(METRIC_OPTIONS[0]);
	let chartData = $state<Array<{ group: string; date: Date; value: number }>>([]);
	let chartOptions = $derived<LineChartOptions>({
		title: selectedMetric.text,
		axes: {
			left: { mapsTo: 'value', scaleType: ScaleTypes.LINEAR },
			bottom: { mapsTo: 'date', scaleType: ScaleTypes.TIME }
		},
		height: '480px',
		legend: { position: 'top' },
		theme: get(carbonTheme)
	});

	let eventSource: EventSource | null = null;

	function connectSSE() {
		if (eventSource) {
			eventSource.close();
			eventSource = null;
		}
		const params = new URLSearchParams({ metric: selectedMetric.id, minutes: '15', sse: '1' });
		const es = new EventSource(`/sbg/api?${params.toString()}`);
		eventSource = es;

		isLoading = true;
		es.addEventListener('metric', (e: MessageEvent) => {
			try {
				const body = JSON.parse(e.data);
				chartData = body.data ?? [];
			} catch {}
		});
		es.addEventListener('snapshot', (e: MessageEvent) => {
			try {
				const latestData = JSON.parse(e.data);
				if (latestData?.ekfNav?.time_stamp !== sbgData?.ekfNav?.time_stamp) {
					sbgData = latestData;
					lastUpdated = new Date();
				}
				isLoading = false;
			} catch {}
		});
		es.addEventListener('error', (e) => {
			console.error('SSE error:', e);
		});
	}

	onMount(() => {
		connectSSE();
		return () => {
			if (eventSource) eventSource.close();
		};
	});

	function formatNum(value: number | null | undefined, decimals = 3): string {
		if (value === null || typeof value === 'undefined') return 'N/A';
		return value.toFixed(decimals);
	}
	function formatTimestamp(ts: number | null | undefined): string {
		if (ts === null || typeof ts === 'undefined') return 'N/A';
		try {
			return new Date(ts * 1000).toLocaleString();
		} catch {
			return 'Invalid Date';
		}
	}
	function getStatusKind(status: string | null | undefined): 'red' | 'green' | 'cool-gray' {
		if (!status) return 'cool-gray';
		const s = status.toLowerCase();
		if (s.includes('ok') || s.includes('valid') || s.includes('good')) return 'green';
		if (s.includes('error') || s.includes('invalid') || s.includes('bad')) return 'red';
		return 'cool-gray';
	}

	// Toast any error changes
	let lastErrorMessage = $state<string | null>(null);
	$effect(() => {
		if (error && error !== lastErrorMessage) {
			toastStore.error(`SBG: ${error}`, 0);
		}
		lastErrorMessage = error;
	});
</script>

<svelte:head>
	<title>SBG Sensor Status</title>
	<meta name="description" content="Real-time SBG sensor data summary" />
	<style>
		.bx--structured-list {
			margin-bottom: 1rem;
		}
	</style>
</svelte:head>

<Grid padding={true} class="rgs-route">
	<Row>
		<Column>
			<Tile>
				<Tag>
					Last updated: {lastUpdated.toLocaleString()}
					{#if isLoading}
						<InlineLoading description="Updating..." />
					{/if}
				</Tag>

				<h2>
					EKF Navigation
					<Tag type={getStatusKind(sbgData.ekfNav?.status)}>{sbgData.ekfNav?.status ?? 'N/A'}</Tag>
				</h2>
				<StructuredList condensed>
					<StructuredListHead>
						<StructuredListRow head>
							<StructuredListCell head>Field</StructuredListCell>
							<StructuredListCell head>Value</StructuredListCell>
						</StructuredListRow>
					</StructuredListHead>
					<StructuredListBody>
						<StructuredListRow>
							<StructuredListCell>Timestamp</StructuredListCell>
							<StructuredListCell>{formatTimestamp(sbgData.ekfNav?.time_stamp)}</StructuredListCell>
						</StructuredListRow>
						<StructuredListRow>
							<StructuredListCell>Latitude</StructuredListCell>
							<StructuredListCell
								>{formatNum(sbgData.ekfNav?.position_latitude, 7)}° (±{formatNum(
									sbgData.ekfNav?.position_std_dev_latitude,
									2
								)}m)</StructuredListCell
							>
						</StructuredListRow>
						<StructuredListRow>
							<StructuredListCell>Longitude</StructuredListCell>
							<StructuredListCell
								>{formatNum(sbgData.ekfNav?.position_longitude, 7)}° (±{formatNum(
									sbgData.ekfNav?.position_std_dev_longitude,
									2
								)}m)</StructuredListCell
							>
						</StructuredListRow>
						<StructuredListRow>
							<StructuredListCell>Altitude</StructuredListCell>
							<StructuredListCell
								>{formatNum(sbgData.ekfNav?.position_altitude, 2)} m (±{formatNum(
									sbgData.ekfNav?.position_std_dev_altitude,
									2
								)}m)</StructuredListCell
							>
						</StructuredListRow>
						<StructuredListRow>
							<StructuredListCell>Vel N/E/D</StructuredListCell>
							<StructuredListCell
								>{formatNum(sbgData.ekfNav?.velocity_north)} / {formatNum(
									sbgData.ekfNav?.velocity_east
								)} / {formatNum(sbgData.ekfNav?.velocity_down)} m/s</StructuredListCell
							>
						</StructuredListRow>
					</StructuredListBody>
				</StructuredList>

				<h2>
					Air Data
					<Tag type={getStatusKind(sbgData.air?.status)}>{sbgData.air?.status ?? 'N/A'}</Tag>
				</h2>
				<StructuredList condensed>
					<StructuredListHead>
						<StructuredListRow head>
							<StructuredListCell head>Field</StructuredListCell>
							<StructuredListCell head>Value</StructuredListCell>
						</StructuredListRow>
					</StructuredListHead>
					<StructuredListBody>
						<StructuredListRow>
							<StructuredListCell>Timestamp</StructuredListCell>
							<StructuredListCell>{formatTimestamp(sbgData.air?.time_stamp)}</StructuredListCell>
						</StructuredListRow>
						<StructuredListRow>
							<StructuredListCell>Abs Pressure</StructuredListCell>
							<StructuredListCell>{formatNum(sbgData.air?.pressure_abs, 2)} Pa</StructuredListCell>
						</StructuredListRow>
						<StructuredListRow>
							<StructuredListCell>Altitude</StructuredListCell>
							<StructuredListCell>{formatNum(sbgData.air?.altitude, 2)} m</StructuredListCell>
						</StructuredListRow>
						<StructuredListRow>
							<StructuredListCell>Diff Pressure</StructuredListCell>
							<StructuredListCell>{formatNum(sbgData.air?.pressure_diff, 2)} Pa</StructuredListCell>
						</StructuredListRow>
						<StructuredListRow>
							<StructuredListCell>True Airspeed</StructuredListCell>
							<StructuredListCell>{formatNum(sbgData.air?.true_airspeed, 2)} m/s</StructuredListCell
							>
						</StructuredListRow>
						<StructuredListRow>
							<StructuredListCell>Temperature</StructuredListCell>
							<StructuredListCell
								>{formatNum(sbgData.air?.air_temperature, 1)} °C</StructuredListCell
							>
						</StructuredListRow>
					</StructuredListBody>
				</StructuredList>

				<h2>
					IMU Data
					<Tag type={getStatusKind(sbgData.imu?.status)}>{sbgData.imu?.status ?? 'N/A'}</Tag>
				</h2>
				<StructuredList condensed>
					<StructuredListHead>
						<StructuredListRow head>
							<StructuredListCell head>Field</StructuredListCell>
							<StructuredListCell head>Value</StructuredListCell>
						</StructuredListRow>
					</StructuredListHead>
					<StructuredListBody>
						<StructuredListRow>
							<StructuredListCell>Timestamp</StructuredListCell>
							<StructuredListCell>{formatTimestamp(sbgData.imu?.time_stamp)}</StructuredListCell>
						</StructuredListRow>
						<StructuredListRow>
							<StructuredListCell>Accel X/Y/Z</StructuredListCell>
							<StructuredListCell
								>{formatNum(sbgData.imu?.accelerometer_x)} / {formatNum(
									sbgData.imu?.accelerometer_y
								)} / {formatNum(sbgData.imu?.accelerometer_z)} m/s²</StructuredListCell
							>
						</StructuredListRow>
						<StructuredListRow>
							<StructuredListCell>Gyro X/Y/Z</StructuredListCell>
							<StructuredListCell
								>{formatNum(sbgData.imu?.gyroscope_x)} / {formatNum(sbgData.imu?.gyroscope_y)} / {formatNum(
									sbgData.imu?.gyroscope_z
								)} rad/s</StructuredListCell
							>
						</StructuredListRow>
						<StructuredListRow>
							<StructuredListCell>Temperature</StructuredListCell>
							<StructuredListCell>{formatNum(sbgData.imu?.temperature, 1)} °C</StructuredListCell>
						</StructuredListRow>
					</StructuredListBody>
				</StructuredList>
			</Tile>
		</Column>

		<Column>
			<Tile>
				<Dropdown
					items={METRIC_OPTIONS}
					selectedId={selectedMetric.id}
					itemToString={(i) => i?.text ?? ''}
					on:select={(e) => {
						selectedMetric = e.detail.selectedItem;
						connectSSE();
					}}
					size="sm"
					label="Metric"
				/>
				<LineChart data={chartData} options={chartOptions} />
			</Tile>
		</Column>
	</Row>
</Grid>
