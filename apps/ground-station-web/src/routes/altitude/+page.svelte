<script lang="ts">
	import { onMount, onDestroy } from 'svelte';
	import { Button, InlineNotification } from 'carbon-components-svelte';

	import { LineChart } from '@carbon/charts-svelte';
	import type { ChartTabularData, LineChartOptions } from '@carbon/charts-svelte';
	import { ScaleTypes } from '@carbon/charts';

	type LatestSnapshot = {
		barometer: {
			ts: number;
			pressure_kpa: number;
			temperature_celsius: number | null;
			altitude_m: number;
		} | null;
		sbgAir: { ts: number; pressure_pa: number; pressure_kpa: number; altitude_m: number } | null;
		gnss: { ts: number; altitude_m: number } | null;
	};

	let error: string | null = $state(null);
	let loading = $state(false);
	let connected = $state(false);
	let eventSource: EventSource | null = null;
	let latest: LatestSnapshot | null = $state(null);
	import { findSetting } from '$lib/common/settings';
	const altitudeMinutesSetting = findSetting('flight.altitudeMinutes');
	import { get } from 'svelte/store';
	// QNH is not user-configurable in this page; hardcode to 102.5 kPa
	let qnhKpa = $state(102.5);
	let minutesBack = $state(
		altitudeMinutesSetting && altitudeMinutesSetting.valueDescription === 'number'
			? Number(get(altitudeMinutesSetting.value))
			: 15
	);

	let mToFeet = (meters: number) => meters / 0.3048;

	let chartData: ChartTabularData = $state([]);
	const chartOptions: LineChartOptions = {
		title: 'Altitude Over Time',
		axes: {
			left: { title: 'Altitude (m)', mapsTo: 'value', scaleType: ScaleTypes.LINEAR },
			bottom: { title: 'Time', mapsTo: 'date', scaleType: ScaleTypes.TIME }
		},
		curve: 'curveMonotoneX',
		legend: { enabled: true },
		grid: { x: { enabled: true }, y: { enabled: true } },
		height: '400px',
		toolbar: { enabled: false }
	};

	function formatTs(ts: number | null | undefined): string {
		if (!ts || !Number.isFinite(ts)) return 'N/A';
		try {
			return new Date(ts * 1000).toLocaleString();
		} catch {
			return 'N/A';
		}
	}

	async function refresh() {
		loading = true;
		try {
			const res = await fetch(`/altitude/api?minutes=${encodeURIComponent(minutesBack)}`);
			if (!res.ok) throw new Error(await res.text());
			const body = await res.json();
			latest = body.latest;
			chartData = body.series?.data ?? [];
			error = null;
		} catch (e: any) {
			error = e.message || 'Failed to fetch data';
		} finally {
			loading = false;
		}
	}

	function startSSE() {
		if (eventSource) eventSource.close();
		eventSource = new EventSource(`/altitude/api?sse=1&minutes=${encodeURIComponent(minutesBack)}`);
		connected = false;
		loading = true;
		eventSource.onopen = () => {
			connected = true;
			loading = false;
			error = null;
		};
		eventSource.onmessage = (event) => {
			try {
				const data = JSON.parse(event.data);
				latest = data.latest;
				chartData = data.series?.data ?? [];
				console.log('chartData sample', chartData?.[0]);
			} catch (e) {
				console.error('Error parsing event data', e);
				/* ignore */
			}
		};
		eventSource.addEventListener('altitude', (event) => {
			try {
				const data = JSON.parse(event.data);
				latest = data.latest;
				chartData = data.series?.data ?? [];
			} catch {
				/* ignore */
			}
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

<div class="rgs-page telemetry-page">
	<header class="rgs-page-header">
		<div>
			<h1 class="rgs-page-title">Altitude</h1>
			<div class="rgs-muted subtitle">BAROMETRIC AND GNSS ALTITUDE SOURCES</div>
		</div>
		<div class="rgs-page-actions">
			<span
				class="rgs-status {connected
					? 'rgs-status--good'
					: error
						? 'rgs-status--danger'
						: 'rgs-status--warn'}">{connected ? 'LIVE' : error ? 'ERROR' : 'PAUSED'}</span
			>
			<Button kind="secondary" size="small" on:click={refresh} disabled={loading}
				>{loading ? 'REFRESHING' : 'REFRESH'}</Button
			>
			<Button
				kind={connected ? 'danger-tertiary' : 'primary'}
				size="small"
				on:click={connected ? stopSSE : startSSE}>{connected ? 'PAUSE LIVE' : 'START LIVE'}</Button
			>
		</div>
	</header>
	{#if error}<InlineNotification
			kind="error"
			title="Altitude unavailable"
			subtitle={error}
			hideCloseButton
		/>{/if}
	<section class="source-grid">
		<div class="source-card">
			<span class="source-label">BAROMETER</span><span class="rgs-value"
				>{latest?.barometer ? latest.barometer.altitude_m.toFixed(1) : '---'}<span class="rgs-unit"
					>M</span
				></span
			><span
				>{latest?.barometer
					? `${mToFeet(latest.barometer.altitude_m).toFixed(1)} FT · ${latest.barometer.pressure_kpa.toFixed(3)} KPA`
					: 'NO DATA'}</span
			><time>{formatTs(latest?.barometer?.ts)}</time>
		</div>
		<div class="source-card">
			<span class="source-label">SBG AIR</span><span class="rgs-value"
				>{latest?.sbgAir ? latest.sbgAir.altitude_m.toFixed(1) : '---'}<span class="rgs-unit"
					>M</span
				></span
			><span
				>{latest?.sbgAir
					? `${mToFeet(latest.sbgAir.altitude_m).toFixed(1)} FT · ${latest.sbgAir.pressure_kpa.toFixed(3)} KPA`
					: 'NO DATA'}</span
			><time>{formatTs(latest?.sbgAir?.ts)}</time>
		</div>
		<div class="source-card">
			<span class="source-label">GNSS</span><span class="rgs-value"
				>{latest?.gnss ? latest.gnss.altitude_m.toFixed(1) : '---'}<span class="rgs-unit">M</span
				></span
			><span
				>{latest?.gnss ? `${mToFeet(latest.gnss.altitude_m).toFixed(1)} FT · GPS` : 'NO DATA'}</span
			><time>{formatTs(latest?.gnss?.ts)}</time>
		</div>
	</section>
	<section class="rgs-panel chart-panel">
		<div class="rgs-panel-header">
			<h2>Altitude History</h2>
			<span>QNH {qnhKpa.toFixed(1)} KPA</span>
		</div>
		<div class="chart-body"><LineChart data={chartData} options={chartOptions} /></div>
	</section>
</div>

<style>
	.subtitle,
	.source-label,
	time {
		color: var(--rgs-text-muted);
		font-size: 0.68rem;
		letter-spacing: 0.06em;
	}
	.source-grid {
		display: grid;
		grid-template-columns: repeat(3, 1fr);
		gap: 1px;
		border: 1px solid var(--rgs-border);
		background: var(--rgs-border);
	}
	.source-card {
		display: flex;
		min-height: 112px;
		flex-direction: column;
		gap: 5px;
		padding: 10px 12px;
		background: var(--rgs-surface);
	}
	.source-card time {
		margin-top: auto;
	}
	.chart-panel {
		min-height: 0;
	}
	.chart-body {
		min-height: 420px;
	}
	.rgs-panel-header span {
		color: var(--rgs-text-muted);
		font-size: 0.7rem;
	}
	@media (max-width: 767px) {
		.source-grid {
			grid-template-columns: 1fr;
		}
		.source-card {
			min-height: 96px;
		}
		.chart-body {
			min-height: 320px;
			overflow-x: auto;
		}
	}
</style>
