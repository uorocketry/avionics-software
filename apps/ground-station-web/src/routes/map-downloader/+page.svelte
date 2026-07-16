<script lang="ts">
	import { onMount, onDestroy } from 'svelte';
	import 'leaflet/dist/leaflet.css';
	import {
		Tile,
		NumberInput,
		Slider,
		Button,
		InlineNotification,
		Select,
		SelectItem
	} from 'carbon-components-svelte';

	// Lazy-loaded Leaflet module
	let Leaflet: any = null;

	// UI + map state
	let mapContainer: HTMLDivElement | null = null;
	let map: any = null;
	let selectionCircle = $state<any>(null);

	let minZoom = $state(1);
	let maxZoom = $state(19);
	let radiusKm = $state(5);

	let currentLayer = $state('satellite');
	let tileLayerInstance: any = null;

	$effect(() => {
		if (tileLayerInstance && currentLayer) {
			tileLayerInstance.setUrl(`${TILE_BASE}/tiles/${currentLayer}/{z}/{x}/{y}`);
		}
	});

	// Download/job state
	type DownloadState = {
		id: string;
		lat: number;
		lon: number;
		minZoom: number;
		maxZoom: number;
		radius: number; // km
		downloaded: number;
		total: number;
		status: 'pending' | 'processing' | 'completed' | 'cancelled' | 'failed';
		error?: string;
	} | null;

	let job: DownloadState = $state(null);
	let pollHandle: number | null = null;

	const TILE_BASE = import.meta.env.DEV ? 'http://localhost:6565' : '//tiles.uorocketry.ca';

	function clampConfig() {
		if (!Number.isInteger(minZoom)) minZoom = 1;
		if (!Number.isInteger(maxZoom)) maxZoom = 19;
		minZoom = Math.max(0, Math.min(19, minZoom));
		maxZoom = Math.max(0, Math.min(19, maxZoom));
		if (minZoom > maxZoom) {
			minZoom = 1;
			maxZoom = 19;
		}
		radiusKm = Math.max(0.1, Math.min(25, +radiusKm || 5));
	}

	function saveState() {
		if (!map) return;
		const center = map.getCenter?.();
		const state = {
			center: center ? { lat: center.lat, lng: center.lng } : null,
			zoom: map.getZoom?.(),
			minZoom,
			maxZoom,
			radiusKm,
			job
		};
		localStorage.setItem('mapDownloaderState', JSON.stringify(state));
	}

	function validateCoordinates(lat: number, lon: number): boolean {
		return lat >= -90 && lat <= 90 && lon >= -180 && lon <= 180;
	}

	function restoreState() {
		try {
			const raw = localStorage.getItem('mapDownloaderState');
			if (!raw) return;
			const state = JSON.parse(raw);

			minZoom = Number.isInteger(state?.minZoom) ? Math.max(0, Math.min(19, state?.minZoom)) : 1;
			maxZoom = Number.isInteger(state?.maxZoom) ? Math.max(0, Math.min(19, state?.maxZoom)) : 19;
			if (minZoom > maxZoom) {
				minZoom = 1;
				maxZoom = 19;
			}
			radiusKm = Number.isFinite(state?.radiusKm)
				? Math.max(0.1, Math.min(25, state?.radiusKm))
				: 5;

			if (
				state?.center &&
				typeof state.center.lat === 'number' &&
				typeof state.center.lng === 'number' &&
				validateCoordinates(state.center.lat, state.center.lng)
			) {
				map?.setView([state.center.lat, state.center.lng], state.zoom || 2);
			}

			if (
				state?.job &&
				typeof state.job.lat === 'number' &&
				typeof state.job.lon === 'number' &&
				validateCoordinates(state.job.lat, state.job.lon)
			) {
				job = {
					...state.job,
					radius: Math.max(0.1, Math.min(25, state.job.radius ?? radiusKm))
				};
				if (job) {
					const center = Leaflet?.latLng(job.lat, job.lon);
					if (center) {
						selectionCircle = Leaflet.circle(center, {
							radius: job.radius * 1000,
							color: '#3388ff',
							fillColor: '#3388ff',
							fillOpacity: 0.2,
							weight: 2,
							dashArray: '5, 10'
						}).addTo(map);
					}
					if (job.status === 'processing' && job.id) startPolling(job.id);
				}
			}
		} catch (e) {
			console.error('Failed to restore state', e);
			localStorage.removeItem('mapDownloaderState');
		}
	}

	function clearJob() {
		if (pollHandle) {
			clearInterval(pollHandle);
			pollHandle = null;
		}
		job = null;
		saveState();
	}

	function startPolling(jobId: string) {
		if (pollHandle) clearInterval(pollHandle);
		pollHandle = window.setInterval(async () => {
			try {
				const res = await fetch(`${TILE_BASE}/api/download/${jobId}/status`);
				if (!res.ok) throw new Error('Failed to fetch job status');
				const j = await res.json();
				if (job) {
					job = {
						...job,
						...j,
						downloaded: j.downloaded || 0,
						total: j.total || 0,
						status: j.status || 'failed',
						error: j.error
					};
					saveState();
					if (j.status !== 'processing') {
						clearInterval(pollHandle!);
						pollHandle = null;
					}
				}
			} catch (err: unknown) {
				console.error('Polling error', err);
				if (job) {
					job.status = 'failed';
					job.error = err instanceof Error ? err.message : 'Failed to fetch job status';
					saveState();
				}
				if (pollHandle) {
					clearInterval(pollHandle);
					pollHandle = null;
				}
			}
		}, 1000);
	}

	async function startDownload() {
		if (!map || !selectionCircle) return;
		const center = selectionCircle.getLatLng();
		const radiusMeters = selectionCircle.getRadius();

		// Validate coordinates before proceeding
		if (!validateCoordinates(center.lat, center.lng)) {
			console.error('Invalid coordinates:', center.lat, center.lng);
			alert('Invalid coordinates detected. Please click on the map again to set a valid location.');
			return;
		}

		job = {
			id: '',
			lat: center.lat,
			lon: center.lng,
			minZoom,
			maxZoom,
			radius: radiusMeters / 1000,
			downloaded: 0,
			total: 0,
			status: 'pending'
		};
		saveState();

		try {
			const res = await fetch(`${TILE_BASE}/api/download`, {
				method: 'POST',
				headers: { 'Content-Type': 'application/json' },
				body: JSON.stringify({
					lat: center.lat,
					lon: center.lng,
					radiusKm: radiusMeters / 1000,
					minZoom,
					maxZoom
				})
			});
			if (!res.ok) throw new Error('Download request failed');
			const result = await res.json();
			if (!result.success) throw new Error(result.error || 'Download failed');
			if (result.jobId && job) {
				job.id = result.jobId;
				job.status = 'processing';
				saveState();
				startPolling(result.jobId);
			}
		} catch (err: unknown) {
			console.error('Error starting download', err);
			if (job) {
				job.status = 'failed';
				job.error = err instanceof Error ? err.message : 'Unknown error';
				saveState();
			}
		}
	}

	async function cancelDownload() {
		if (!job?.id) return;
		try {
			const res = await fetch(`${TILE_BASE}/api/download/${job.id}/cancel`, { method: 'POST' });
			if (!res.ok) throw new Error('Failed to cancel download');
			const result = await res.json();
			if (result.success) {
				if (pollHandle) {
					clearInterval(pollHandle);
					pollHandle = null;
				}
				job.status = 'cancelled';
				saveState();
			}
		} catch (err) {
			console.error('Cancel error', err);
		}
	}

	function handleMapClick(e: any) {
		if (!map) return;
		if (job?.status === 'processing') return;
		if (selectionCircle) map.removeLayer(selectionCircle);
		selectionCircle = Leaflet.circle(e.latlng, {
			radius: radiusKm * 1000,
			color: '#3388ff',
			fillColor: '#3388ff',
			fillOpacity: 0.2,
			weight: 2,
			dashArray: '5, 10'
		}).addTo(map);
		saveState();
	}

	$effect(() => {
		if (selectionCircle) {
			selectionCircle.setRadius(radiusKm * 1000);
			saveState();
		}
	});

	onMount(async () => {
		// Load Leaflet from local dependency
		const mod = await import('leaflet');
		Leaflet = (mod as any).default ?? mod;

		// Init map
		map = Leaflet.map(mapContainer!, { minZoom, maxZoom }).setView([0, 0], 2);
		tileLayerInstance = Leaflet.tileLayer(`${TILE_BASE}/tiles/${currentLayer}/{z}/{x}/{y}`, {
			maxZoom,
			tileSize: 256
		}).addTo(map);

		map.on('click', handleMapClick);
		map.on('moveend', saveState);

		restoreState();
	});

	onDestroy(() => {
		if (pollHandle) clearInterval(pollHandle);
		pollHandle = null;
		if (map) {
			map.off('click', handleMapClick);
			map.off('moveend', saveState);
		}
	});
</script>

<div class="mapdl-container">
	<div class="map" bind:this={mapContainer}></div>

	<div class="control-wrap">
		<Tile>
			<h3>Download Settings</h3>

			<Select labelText="Active Map Layer" bind:selected={currentLayer}>
				<SelectItem value="satellite" text="Satellite Hybrid" />
				<SelectItem value="terrain" text="Terrain with Roads" />
			</Select>
			<div style="margin-bottom: 12px;"></div>

			<NumberInput
				id="min-zoom"
				label="Minimum Zoom Level"
				min={0}
				max={19}
				bind:value={minZoom}
				onchange={clampConfig}
			/>
			<NumberInput
				id="max-zoom"
				label="Maximum Zoom Level"
				min={0}
				max={19}
				bind:value={maxZoom}
				onchange={clampConfig}
			/>

			<Slider
				id="radius"
				min={0.1}
				max={25}
				step={0.1}
				bind:value={radiusKm}
				labelText={`Radius (${radiusKm.toFixed(1)} km)`}
				oninput={clampConfig}
			/>

			<div class="btn-row">
				<Button
					kind="primary"
					onclick={startDownload}
					disabled={!selectionCircle || job?.status === 'processing'}>Download</Button
				>
				{#if job?.status === 'processing'}
					<Button kind="danger" onclick={cancelDownload}>Cancel</Button>
				{/if}
			</div>

			{#if job}
				<div class="progress">
					<div class="bar">
						<div
							class="fill"
							style={`width:${job.total > 0 ? (job.downloaded / job.total) * 100 : 0}%`}
						></div>
					</div>
					<div class="text">{job.downloaded}/{job.total} tiles</div>
					<div class="cfg">
						<div>Lat: {job.lat.toFixed(6)}</div>
						<div>Lon: {job.lon.toFixed(6)}</div>
						<div>Zoom: {job.minZoom}-{job.maxZoom}</div>
						<div>Radius: {job.radius.toFixed(1)}km</div>
					</div>
					{#if job.status === 'failed'}
						<div class="btn-center">
							<Button kind="secondary" onclick={startDownload}>Retry Download</Button>
						</div>
					{/if}
					{#if job.status !== 'processing'}
						<div class="btn-center">
							<Button kind="tertiary" onclick={clearJob}>Clear</Button>
						</div>
					{/if}
				</div>
			{/if}

			{#if job?.status}
				<InlineNotification
					kind={job.status === 'failed' ? 'error' : job.status === 'completed' ? 'success' : 'info'}
					title={job.status === 'failed'
						? 'Download failed'
						: job.status === 'completed'
							? 'Download complete'
							: 'Download status'}
					subtitle={job.error ? job.error : job.status}
					lowContrast
				/>
			{/if}
		</Tile>
	</div>
</div>

<style>
	.mapdl-container {
		position: relative;
		height: calc(100vh - 3rem);
		width: 100%;
	}
	.map {
		position: absolute;
		top: 0;
		left: 0;
		right: 0;
		bottom: 0;
	}
	.control-wrap {
		position: absolute;
		top: 16px;
		right: 16px;
		z-index: 1000;
	}
	.btn-row {
		display: flex;
		gap: 8px;
		margin-top: 8px;
	}

	.progress {
		margin-top: 12px;
		padding: 10px;
		background: var(--rgs-surface);
		border-radius: 4px;
	}
	.bar {
		height: 20px;
		background: var(--rgs-surface-alt);
		border-radius: 10px;
		overflow: hidden;
		margin-bottom: 6px;
	}
	@media (max-width: 767px) {
		.mapdl-container {
			height: calc(100vh - var(--rgs-header-h));
		}
		.control-wrap {
			inset: auto 8px 8px 8px;
			max-height: 52vh;
			overflow-y: auto;
		}
		.control-wrap :global(.cds--tile) {
			width: 100%;
		}
	}
	.fill {
		height: 100%;
		background: #28a745;
		transition: width 0.3s ease;
	}
	.text {
		text-align: center;
		font-weight: 600;
		color: #666;
		margin-bottom: 6px;
	}
	.cfg {
		font-size: 0.9em;
		color: #666;
		display: grid;
		grid-template-columns: 1fr 1fr;
		gap: 6px;
	}
	.btn-center {
		margin-top: 10px;
		text-align: center;
	}

	/* Notification handled by Carbon InlineNotification */
</style>
