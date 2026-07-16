<script lang="ts">
	import { onMount, onDestroy } from 'svelte';
	import 'leaflet/dist/leaflet.css';
	import { Tile, Slider, Button, InlineNotification } from 'carbon-components-svelte';

	let Leaflet: any = null;

	let mapContainer: HTMLDivElement | null = null;
	let map: any = null;
	let marker: any = null;
	let centerCircle: any = null;

	let selectedLat = $state<number | null>(null);
	let selectedLon = $state<number | null>(null);
	let selectedAlt = $state<number>(0);

	let submitting = $state(false);
	let notice: { kind: 'success' | 'error' | 'info'; title: string; subtitle?: string } | null =
		$state(null);

	const TILE_BASE = import.meta.env.DEV ? 'http://localhost:6565' : '//tiles.uorocketry.ca';
	const CENTER = { lat: 47.986877, lon: -81.848765 };
	const RADIUS_KM = 50;

	function handleMapClick(e: any) {
		if (!map) return;
		const { lat, lng } = e.latlng;
		selectedLat = lat;
		selectedLon = lng;
		if (marker) marker.remove();
		marker = Leaflet.marker([lat, lng]).addTo(map);
	}

	onMount(async () => {
		const mod = await import('leaflet');
		Leaflet = (mod as any).default ?? mod;

		map = Leaflet.map(mapContainer!, { minZoom: 1, maxZoom: 19 }).setView([0, 0], 2);
		Leaflet.tileLayer(`${TILE_BASE}/tiles/{z}/{x}/{y}`, { maxZoom: 19, tileSize: 256 }).addTo(map);
		map.on('click', handleMapClick);

		// Add reference circle for the trajectory center
		centerCircle = Leaflet.circle([CENTER.lat, CENTER.lon], {
			radius: RADIUS_KM * 1000,
			color: '#0f62fe',
			fillColor: '#0f62fe',
			fillOpacity: 0.05,
			weight: 1
		}).addTo(map);
	});

	onDestroy(() => {
		if (map) {
			map.off('click', handleMapClick);
		}
	});

	async function insertMock() {
		notice = null;
		if (selectedLat == null || selectedLon == null) {
			notice = {
				kind: 'error',
				title: 'No location selected',
				subtitle: 'Click on the map to pick a point.'
			};
			return;
		}
		submitting = true;
		try {
			const res = await fetch('/debug/coord-mock/api', {
				method: 'POST',
				headers: { 'Content-Type': 'application/json' },
				body: JSON.stringify({ lat: selectedLat, lon: selectedLon, alt: selectedAlt })
			});
			if (!res.ok) throw new Error(await res.text());
			const result = await res.json();
			if (!result.success) throw new Error(result.error || 'Unknown error');
			notice = {
				kind: 'success',
				title: 'Inserted mock telemetry',
				subtitle: `lat=${selectedLat.toFixed(6)}, lon=${selectedLon.toFixed(6)}, alt=${selectedAlt}m`
			};
		} catch (e: any) {
			notice = { kind: 'error', title: 'Insert failed', subtitle: e?.message ?? String(e) };
		} finally {
			submitting = false;
		}
	}
</script>

<div class="coord-mock-container">
	<div class="map" bind:this={mapContainer}></div>

	<div class="control-wrap">
		<Tile>
			<h3>Coordinate Mock</h3>
			<div class="row">
				<div><strong>Lat:</strong> {selectedLat != null ? selectedLat.toFixed(6) : '—'}</div>
				<div><strong>Lon:</strong> {selectedLon != null ? selectedLon.toFixed(6) : '—'}</div>
			</div>

			<Slider
				id="alt"
				min={0}
				max={5000}
				step={1}
				bind:value={selectedAlt}
				labelText={`Altitude (${selectedAlt} m)`}
			/>

			<div class="btn-row">
				<Button
					kind="primary"
					onclick={insertMock}
					disabled={submitting || selectedLat == null || selectedLon == null}
				>
					{submitting ? 'Inserting…' : 'Insert Telemetry'}
				</Button>
			</div>

			{#if notice}
				<InlineNotification
					lowContrast
					kind={notice.kind}
					title={notice.title}
					subtitle={notice.subtitle}
				/>
			{/if}
		</Tile>
	</div>
</div>

<style>
	.coord-mock-container {
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
	.row {
		display: grid;
		grid-template-columns: 1fr 1fr;
		gap: 8px;
		margin-bottom: 8px;
	}
	.btn-row {
		display: flex;
		gap: 8px;
		margin-top: 8px;
	}
	@media (max-width: 767px) {
		.coord-mock-container {
			height: calc(100vh - var(--rgs-header-h));
		}
		.control-wrap {
			inset: auto 8px 8px 8px;
		}
		.control-wrap :global(.cds--tile) {
			width: 100%;
		}
	}
</style>
