<script lang="ts">
	import { onMount, onDestroy } from 'svelte';
	import 'leaflet/dist/leaflet.css';
	import {
		Tile,
		Button,
		InlineNotification,
		Tag,
		Select,
		SelectItem
	} from 'carbon-components-svelte';
	import { findSetting } from '$lib/common/settings';
	import { get } from 'svelte/store';

	let Leaflet: any = null;
	let mapContainer: HTMLDivElement | null = null;
	let map: any = null;
	let path: any = null;
	let centerCircle: any = null;
	let latestMarker: any = null;
	let accuracyCircle: any = null;

	const CENTER = { lat: 47.986877, lon: -81.848765 };
	const TILE_BASE = import.meta.env.DEV ? 'http://localhost:6565' : '//tiles.uorocketry.ca';

	let currentLayer = $state('satellite');
	let tileLayerInstance = $state<any>(null);

	$effect(() => {
		console.log(
			'Trajectory Map Layer Triggered:',
			currentLayer,
			'Instance Ready:',
			!!tileLayerInstance
		);
		if (tileLayerInstance && currentLayer) {
			const url = `${TILE_BASE}/tiles/${currentLayer}/{z}/{x}/{y}`;
			console.log('Setting Leaflet Tile URL:', url);
			tileLayerInstance.setUrl(url);
		}
	});
	const trajMinutesSetting = findSetting('flight.trajMinutes');
	const trajMaxPointsSetting = findSetting('flight.trajMaxPoints');
	const trajRadiusSetting = findSetting('flight.trajRadiusKm');
	let RADIUS_KM = $state(
		trajRadiusSetting && trajRadiusSetting.valueDescription === 'number'
			? Number(get(trajRadiusSetting.value))
			: 50
	);
	// Centralized query params for API requests
	let WINDOW_MINUTES = $state(
		trajMinutesSetting && trajMinutesSetting.valueDescription === 'number'
			? Number(get(trajMinutesSetting.value as any))
			: 240
	);
	let MAX_POINTS = $state(
		trajMaxPointsSetting && trajMaxPointsSetting.valueDescription === 'number'
			? Number(get(trajMaxPointsSetting.value as any))
			: 3000
	);
	function buildQueryParams() {
		return `minutes=${WINDOW_MINUTES}&max_points=${MAX_POINTS}&lat=${CENTER.lat}&lon=${CENTER.lon}&radius_km=${RADIUS_KM}`;
	}

	let error: string | null = $state(null);
	let connected = $state(false);
	let loading = $state(false);
	let eventSource: EventSource | null = null;

	type Latest = {
		ts: number;
		lat: number;
		lon: number;
		lat_acc: number | null;
		lon_acc: number | null;
		num_sv_used: number | null;
	} | null;
	type Series = { points: Array<{ ts: number; lat: number; lon: number }> };

	let latest: Latest = $state(null);
	let series: Series | null = $state(null);

	function metersForAccuracy(latAcc: number | null, lonAcc: number | null): number | null {
		if (latAcc == null && lonAcc == null) return null;
		// Roughly, 1 deg lat ~ 111.32 km; for lon scale by cos(lat)
		const latMeters = latAcc != null ? latAcc * 111320 : 0;
		const lonMeters =
			lonAcc != null
				? lonAcc * 111320 * Math.cos(((latest?.lat ?? CENTER.lat) * Math.PI) / 180)
				: 0;
		const r = Math.max(latMeters, lonMeters);
		return isFinite(r) && r > 0 ? r : null;
	}

	function draw() {
		if (!map) return;
		// Center ring for 50km
		if (!centerCircle) {
			centerCircle = Leaflet.circle([CENTER.lat, CENTER.lon], {
				radius: Number(RADIUS_KM) * 1000,
				color: '#0f62fe',
				fillColor: '#0f62fe',
				fillOpacity: 0.05,
				weight: 1
			}).addTo(map);
		}

		// Path polyline
		const latlngs = (series?.points ?? []).map((p) => [p.lat, p.lon]);
		if (!path) {
			path = Leaflet.polyline(latlngs, { color: '#da1e28', weight: 3, smoothFactor: 0.2 }).addTo(
				map
			);
		} else {
			path.setLatLngs(latlngs);
		}

		// Latest marker + accuracy circle
		if (latest && Number.isFinite(latlngs?.length) && latlngs.length > 0) {
			const p = Leaflet.latLng(latest.lat, latest.lon);
			if (!latestMarker) {
				latestMarker = Leaflet.circleMarker(p, {
					radius: 6,
					color: '#24a148',
					weight: 2,
					fillColor: '#24a148',
					fillOpacity: 0.9
				}).addTo(map);
			} else {
				latestMarker.setLatLng(p);
			}
			const accM = metersForAccuracy(latest.lat_acc, latest.lon_acc);
			if (accM && accM > 0) {
				if (!accuracyCircle) {
					accuracyCircle = Leaflet.circle(p, {
						radius: accM,
						color: '#0f62fe',
						fillColor: '#0f62fe',
						fillOpacity: 0.15,
						weight: 1
					}).addTo(map);
				} else {
					accuracyCircle.setLatLng(p);
					accuracyCircle.setRadius(accM);
				}
			} else if (accuracyCircle) {
				map.removeLayer(accuracyCircle);
				accuracyCircle = null;
			}
		}

		// Fit bounds once if nothing set
		if (latlngs.length > 1) {
			const bounds = Leaflet.latLngBounds(latlngs);
			map.fitBounds(bounds, { padding: [20, 20] });
		} else if (latlngs.length === 1) {
			map.setView(latlngs[0], 12);
		} else {
			map.setView([CENTER.lat, CENTER.lon], 9);
		}
	}

	async function refresh() {
		loading = true;
		try {
			const res = await fetch(`/trajectory/api?${buildQueryParams()}`);
			if (!res.ok) throw new Error(await res.text());
			const body = await res.json();
			latest = body.latest;
			series = body.series;
			error = null;
			draw();
		} catch (e: any) {
			error = e.message || 'Failed to fetch';
		} finally {
			loading = false;
		}
	}

	function startSSE() {
		if (eventSource) eventSource.close();
		eventSource = new EventSource(`/trajectory/api?sse=1&${buildQueryParams()}`);
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
				series = data.series;
				draw();
			} catch {}
		};
		eventSource.addEventListener('trajectory', (event) => {
			try {
				const data = JSON.parse(event.data);
				latest = data.latest;
				series = data.series;
				draw();
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

	onMount(async () => {
		const mod = await import('leaflet');
		Leaflet = (mod as any).default ?? mod;
		map = Leaflet.map(mapContainer!, { zoomControl: true, preferCanvas: true }).setView(
			[CENTER.lat, CENTER.lon],
			9
		);
		tileLayerInstance = Leaflet.tileLayer(`${TILE_BASE}/tiles/${currentLayer}/{z}/{x}/{y}`, {
			maxZoom: 19,
			tileSize: 256
		}).addTo(map);
		startSSE();
	});
	onDestroy(() => {
		stopSSE();
	});
</script>

<div class="trajectory-page">
	<Tile class="trajectory-toolbar">
		<div class="toolbar-content">
			<div>
				<h2>Trajectory</h2>
				<div class="position-tags">
					<!-- <Tag type="blue" size="sm" style="white-space: nowrap;"
						>Center: {CENTER.lat.toFixed(6)}, {CENTER.lon.toFixed(6)}</Tag
					> -->
					<!-- <Tag type="teal" size="sm" style="white-space: nowrap;">Radius: {RADIUS_KM} km</Tag> -->
					{#if latest}
						<Tag type="green" size="sm" style="white-space: nowrap;"
							>Lat: {latest.lat.toFixed(6)}</Tag
						>
						<Tag type="green" size="sm" style="white-space: nowrap;"
							>Lon: {latest.lon.toFixed(6)}</Tag
						>
						{#if latest.lat_acc != null}<Tag type="gray" size="sm" style="white-space: nowrap;"
								>Lat σ: {latest.lat_acc.toFixed(2)}°</Tag
							>{/if}
						{#if latest.lon_acc != null}<Tag type="gray" size="sm" style="white-space: nowrap;"
								>Lon σ: {latest.lon_acc.toFixed(2)}°</Tag
							>{/if}
						{#if latest.num_sv_used != null}
							<Tag type="purple" size="sm" style="white-space: nowrap;"
								>SV used: {latest.num_sv_used}</Tag
							>
						{/if}
					{/if}
				</div>
			</div>
			<div class="layer-select">
				<Select noLabel inline bind:selected={currentLayer}>
					<SelectItem value="satellite" text="Satellite Hybrid" />
					<SelectItem value="terrain" text="Terrain with Roads" />
				</Select>
			</div>
			<div class="toolbar-actions">
				<Button kind="secondary" disabled={loading} on:click={refresh}>
					{#if loading}Loading...{:else}Refresh{/if}
				</Button>
				{#if connected}
					<Button kind="danger" style="margin-left: 8px;" on:click={stopSSE}>Stop Live</Button>
				{:else}
					<Button kind="primary" style="margin-left: 8px;" on:click={startSSE}>Start Live</Button>
				{/if}
			</div>
		</div>
		{#if error}
			<InlineNotification
				kind="error"
				title="Error"
				subtitle={error}
				hideCloseButton
				style="margin-top: 0.5rem;"
			/>
		{/if}
	</Tile>

	<div bind:this={mapContainer} class="map-canvas"></div>
</div>

<style>
	.trajectory-page {
		display: grid;
		height: calc(100vh - var(--rgs-header-h));
		grid-template-rows: auto 1fr;
		gap: 8px;
		padding: 8px;
	}
	.toolbar-content {
		display: grid;
		grid-template-columns: minmax(0, 1fr) 200px auto;
		align-items: center;
		gap: 8px;
	}
	.toolbar-content h2 {
		margin: 0 0 4px;
		text-transform: uppercase;
	}
	.position-tags,
	.toolbar-actions {
		display: flex;
		flex-wrap: wrap;
		align-items: center;
		gap: 6px;
	}
	.map-canvas {
		position: relative;
		width: 100%;
		height: 100%;
		min-height: 320px;
		border: 1px solid var(--rgs-border);
	}
	@media (max-width: 900px) {
		.toolbar-content {
			grid-template-columns: 1fr auto;
		}
		.layer-select {
			grid-column: 1;
			grid-row: 2;
			width: 200px;
		}
		.toolbar-actions {
			grid-column: 2;
			grid-row: 1 / 3;
		}
	}
	@media (max-width: 767px) {
		.trajectory-page {
			height: calc(100vh - var(--rgs-header-h));
			padding: 0;
			gap: 0;
			grid-template-rows: auto 1fr;
		}
		.toolbar-content {
			grid-template-columns: 1fr;
		}
		.layer-select,
		.toolbar-actions {
			grid-column: 1;
			grid-row: auto;
			width: 100%;
		}
		.toolbar-actions :global(.cds--btn) {
			flex: 1;
		}
		.position-tags {
			max-height: 52px;
			overflow-y: auto;
		}
		.map-canvas {
			min-height: 260px;
			border-inline: 0;
		}
	}
</style>
