import L from 'leaflet';
import 'leaflet/dist/leaflet.css';
import { html, render } from 'lit-html';

// Create and append the map container
const rootContainer = document.createElement('div');
rootContainer.id = 'map-container';
document.body.appendChild(rootContainer);


interface DownloadState {
	id: string;
	layer: string;
	lat: number;
	lon: number;
	minZoom: number;
	maxZoom: number;
	radius: number;
	downloaded: number;
	total: number;
	status: 'pending' | 'processing' | 'completed' | 'cancelled' | 'failed';
	error?: string;
}

let container: HTMLElement | null = null;
let map: L.Map | null = null;
let tileLayerInstance: L.TileLayer | null = null;
let currentLayer = 'satellite';
let minZoom = 1;
let maxZoom = 19;
let zIndexMin = 2;
let zIndexMax = 3;
let radius = 5;
let downloadCircle: L.Circle | null = null;
let currentDownloadState: DownloadState | null = null;
let statusPollInterval: number | null = null;

// State persistence
function saveState() {
	if (!map) return;
	const center = map.getCenter();
	const state = {
		center: {
			lat: center.lat,
			lng: center.lng
		},
		zoom: map.getZoom(),
		minZoom,
		maxZoom,
		radius,
		currentLayer,
		downloadState: currentDownloadState
	};
	localStorage.setItem('mapState', JSON.stringify(state));
}

function restoreState() {
	const savedState = localStorage.getItem('mapState');
	if (!savedState) return;

	try {
		const state = JSON.parse(savedState);

		// Validate and restore active layer
		currentLayer = typeof state.currentLayer === 'string' ? state.currentLayer : 'satellite';
		if (tileLayerInstance) {
			tileLayerInstance.setUrl(`/tiles/${currentLayer}/{z}/{x}/{y}`);
		}

		// Validate and set zoom levels with defaults
		minZoom = Number.isInteger(state.minZoom) ? Math.max(0, Math.min(19, state.minZoom)) : 1;
		maxZoom = Number.isInteger(state.maxZoom) ? Math.max(0, Math.min(19, state.maxZoom)) : 19;
		if (minZoom > maxZoom) {
			minZoom = 1;
			maxZoom = 19;
		}

		// Validate and set radius with default
		radius = Number.isFinite(state.radius) ? Math.max(0.1, Math.min(25, state.radius)) : 5;

		// Validate and set center coordinates
		if (state.center &&
			typeof state.center.lat === 'number' &&
			typeof state.center.lng === 'number' &&
			!isNaN(state.center.lat) &&
			!isNaN(state.center.lng)) {
			if (map) {
				map.setView([state.center.lat, state.center.lng], state.zoom || 2);
			}
		}

		// Validate and restore download state
		if (state.downloadState &&
			typeof state.downloadState.lat === 'number' &&
			typeof state.downloadState.lon === 'number' &&
			!isNaN(state.downloadState.lat) &&
			!isNaN(state.downloadState.lon)) {
			const validatedState: DownloadState = {
				...state.downloadState,
				lat: state.downloadState.lat,
				lon: state.downloadState.lon,
				radius: Number.isFinite(state.downloadState.radius) ?
					Math.max(0.1, Math.min(25, state.downloadState.radius)) : 5
			};
			currentDownloadState = validatedState;

			if (map) {
				const center = L.latLng(validatedState.lat, validatedState.lon);
				downloadCircle = L.circle(center, {
					radius: validatedState.radius * 1000,
					color: '#3388ff',
					fillColor: '#3388ff',
					fillOpacity: 0.2,
					weight: 2,
					dashArray: '5, 10',
				}).addTo(map);

				if (validatedState.status === 'processing') {
					startStatusPolling(validatedState.id);
				}
			}
		}
	} catch (error) {
		console.error('Error restoring state:', error);
		// Clear invalid state
		localStorage.removeItem('mapState');
	}
}

function clearDownloadState() {
	if (statusPollInterval) {
		clearInterval(statusPollInterval);
		statusPollInterval = null;
	}
	currentDownloadState = null;
	saveState();
	renderControlContent();
}

function startStatusPolling(jobId: string) {
	if (statusPollInterval) {
		clearInterval(statusPollInterval);
	}

	statusPollInterval = window.setInterval(async () => {
		try {
			const response = await fetch(`/api/download/${jobId}/status`);
			if (!response.ok) {
				throw new Error('Failed to fetch job status');
			}

			const job = await response.json();
			if (!job) {
				throw new Error('No job data received');
			}

			// Update the current download state with the job status
			if (currentDownloadState) {
				currentDownloadState = {
					...currentDownloadState,
					...job,
					downloaded: job.downloaded || 0,
					total: job.total || 0,
					status: job.status || 'failed',
					error: job.error
				};
				saveState();
				renderControlContent();

				// Stop polling if the job is no longer processing
				if (job.status !== 'processing') {
					if (statusPollInterval) {
						clearInterval(statusPollInterval);
						statusPollInterval = null;
					}
				}
			}
		} catch (error) {
			console.error('Error polling job status:', error);
			if (currentDownloadState) {
				currentDownloadState.status = 'failed';
				currentDownloadState.error = error instanceof Error ? error.message : 'Failed to fetch job status';
				saveState();
				renderControlContent();
			}
			if (statusPollInterval) {
				clearInterval(statusPollInterval);
				statusPollInterval = null;
			}
		}
	}, 1000);
}

function renderControlContent() {
	if (!container) return;

	const content = html`
    <h3>Layer Settings</h3>
    <label for="layer-select">Active Map Layer:</label>
	    <select
	      id="layer-select"
      @change=${handleLayerChange}
      style="width: 100%; padding: 8px; margin-bottom: 15px; border: 1px solid #ddd; border-radius: 4px; box-sizing: border-box;"
    >
      <option value="satellite" ?selected=${currentLayer === 'satellite'}>Satellite Hybrid</option>
      <option value="terrain" ?selected=${currentLayer === 'terrain'}>Terrain with Roads</option>
    </select>

    <h3>Download Settings</h3>
    <label>Zoom Range: <span class="zoom-value" style="color: #818cf8; font-weight: 700; font-size: 0.95rem; float: right;">${minZoom} - ${maxZoom}</span></label>
    <div class="dual-range-container" @pointermove=${handleSliderPointerMove} @pointerdown=${handleSliderPointerDown}>
      <div class="slider-track" style="background: linear-gradient(to right, rgba(255,255,255,0.1) ${(minZoom/19)*100}%, #6366f1 ${(minZoom/19)*100}%, #6366f1 ${(maxZoom/19)*100}%, rgba(255,255,255,0.1) ${(maxZoom/19)*100}%);"></div>
	      <input
	        type="range"
	        id="min-zoom-slider"
	        min="0"
	        max="19"
	        .value=${minZoom}
        @input=${handleMinSliderInput}
        class="range-slider"
        style="z-index: ${zIndexMin};"
      >
	      <input
	        type="range"
	        id="max-zoom-slider"
	        min="0"
	        max="19"
	        .value=${maxZoom}
        @input=${handleMaxSliderInput}
        class="range-slider"
        style="z-index: ${zIndexMax};"
      >
    </div>
    <label for="radius">Radius:</label>
    <div class="radius-control">
      <input
        type="range"
        id="radius"
        value=${radius}
        min="0.1"
        max="25"
        step="0.1"
        @input=${handleRadiusChange}
        class="radius-slider"
      >
      <span class="radius-value">${radius.toFixed(1)}km</span>
    </div>
    <div class="button-group">
      <button
        @click=${handleDownload}
        ?disabled=${!downloadCircle || currentDownloadState?.status === 'processing'}
        class="download-button"
      >
        Download
      </button>
      ${currentDownloadState?.status === 'processing' ? html`
        <button
          @click=${handleCancelDownload}
          class="cancel-button"
        >
          Cancel
        </button>
      ` : ''}
    </div>
    ${currentDownloadState ? html`
      <div class="download-progress">
        <div class="progress-bar">
          <div class="progress-fill" style="width: ${currentDownloadState.total > 0 ? (currentDownloadState.downloaded / currentDownloadState.total) * 100 : 0}%"></div>
        </div>
        <div class="progress-text">
          ${currentDownloadState.downloaded}/${currentDownloadState.total} tiles
        </div>
        <div class="download-config">
          <div>Lat: ${currentDownloadState.lat.toFixed(6)}</div>
          <div>Lon: ${currentDownloadState.lon.toFixed(6)}</div>
          <div>Zoom: ${currentDownloadState.minZoom}-${currentDownloadState.maxZoom}</div>
          <div>Radius: ${currentDownloadState.radius.toFixed(1)}km</div>
          <div style="grid-column: span 2;">Layer: ${currentDownloadState.layer}</div>
        </div>
        ${currentDownloadState.status === 'failed' ? html`
          <div class="retry-section">
            <button
              @click=${handleRetryDownload}
              class="retry-button"
            >
              🔄 Retry Download
            </button>
          </div>
        ` : ''}
        ${currentDownloadState.status !== 'processing' ? html`
          <div class="clear-section">
            <button
              @click=${clearDownloadState}
              class="clear-button"
            >
              🗑️ Clear
            </button>
          </div>
        ` : ''}
      </div>
    ` : ''}
    ${currentDownloadState?.status ? html`
      <div class="download-status ${currentDownloadState.status === 'failed' ? 'error' : currentDownloadState.status === 'completed' ? 'success' : ''}">
        ${currentDownloadState.status}
      </div>
    ` : ''}
  `;

	render(content, container);
}

function updateTemplate() {
	const template = html`
    <style>
      .map-container {
        height: 100vh;
        width: 100%;
        position: relative;
      }
      #map {
        height: 100%;
        width: 100%;
        position: absolute;
        top: 0;
        left: 0;
        background-color: #0b0f19;
      }
      /* Premium Dark Glassmorphic Panel */
      .download-control {
        background: rgba(13, 17, 23, 0.85) !important;
        backdrop-filter: blur(12px);
        -webkit-backdrop-filter: blur(12px);
        border: 1px solid rgba(255, 255, 255, 0.08) !important;
        color: #f8fafc;
        padding: 16px 20px;
        width: 320px;
        border-radius: 16px !important;
        box-shadow: 0 10px 30px rgba(0, 0, 0, 0.5) !important;
        font-family: 'Plus Jakarta Sans', 'Inter', sans-serif;
      }
      .download-control h3 {
        margin-top: 0;
        margin-bottom: 16px;
        color: #f8fafc;
        font-size: 1.1rem;
        font-weight: 700;
        letter-spacing: -0.02em;
        border-bottom: 1px solid rgba(255, 255, 255, 0.08);
        padding-bottom: 8px;
      }
      .download-control label {
        display: block;
        margin-top: 10px;
        margin-bottom: 6px;
        font-weight: 600;
        font-size: 0.75rem;
        text-transform: uppercase;
        letter-spacing: 0.05em;
        color: #94a3b8;
      }
      .download-control input, .download-control select {
        width: 100%;
        padding: 10px 12px;
        margin-bottom: 12px;
        background: rgba(30, 41, 59, 0.5);
        border: 1px solid rgba(255, 255, 255, 0.12);
        color: #f8fafc;
        border-radius: 8px;
        box-sizing: border-box;
        font-family: inherit;
        font-size: 0.9rem;
        transition: border-color 0.2s, box-shadow 0.2s;
      }
      .download-control input:focus, .download-control select:focus {
        outline: none;
        border-color: #6366f1;
        box-shadow: 0 0 0 3px rgba(99, 102, 241, 0.25);
      }
      .radius-control {
        display: flex;
        align-items: center;
        gap: 12px;
        margin-bottom: 12px;
      }
      .radius-slider {
        flex: 1;
        -webkit-appearance: none;
        width: 100%;
        height: 6px;
        background: rgba(255, 255, 255, 0.1);
        border-radius: 9999px;
        outline: none;
        margin: 0 !important;
        padding: 0 !important;
        border: none !important;
      }
      .radius-slider::-webkit-slider-runnable-track {
        background: none;
        border: none;
      }
      .radius-slider::-moz-range-track {
        background: none;
        border: none;
      }
      .radius-slider::-webkit-slider-thumb {
        -webkit-appearance: none;
        appearance: none;
        width: 18px;
        height: 18px;
        background: #6366f1;
        border-radius: 50%;
        cursor: pointer;
        transition: background-color 0.2s, transform 0.1s;
        box-shadow: 0 2px 6px rgba(0, 0, 0, 0.4);
        margin: 0;
      }
      .radius-slider::-moz-range-thumb {
        width: 18px;
        height: 18px;
        background: #6366f1;
        border-radius: 50%;
        cursor: pointer;
        transition: background-color 0.2s, transform 0.1s;
        box-shadow: 0 2px 6px rgba(0, 0, 0, 0.4);
        border: none;
      }
      .radius-slider::-webkit-slider-thumb:hover {
        background: #818cf8;
        transform: scale(1.1);
      }
      .radius-slider::-moz-range-thumb:hover {
        background: #818cf8;
        transform: scale(1.1);
      }
      .radius-value {
        min-width: 60px;
        text-align: right;
        font-weight: 700;
        color: #818cf8;
        font-size: 0.9rem;
      }
      .button-group {
        display: flex;
        gap: 10px;
        margin-top: 16px;
      }
      .download-button, .cancel-button {
        flex: 1;
        padding: 12px;
        border: none;
        border-radius: 8px;
        cursor: pointer;
        font-weight: 600;
        font-size: 0.9rem;
        font-family: inherit;
        transition: all 0.2s ease;
      }
      .download-button {
        background: linear-gradient(135deg, #6366f1 0%, #4f46e5 100%);
        color: white;
        box-shadow: 0 4px 12px rgba(99, 102, 241, 0.25);
      }
      .download-button:hover:not(:disabled) {
        opacity: 0.95;
        transform: translateY(-1px);
        box-shadow: 0 6px 16px rgba(99, 102, 241, 0.35);
      }
      .download-button:disabled {
        background: #334155;
        color: #64748b;
        cursor: not-allowed;
        opacity: 0.6;
        transform: none;
        box-shadow: none;
      }
      .cancel-button {
        background: linear-gradient(135deg, #ef4444 0%, #dc2626 100%);
        color: white;
        box-shadow: 0 4px 12px rgba(239, 68, 68, 0.2);
      }
      .cancel-button:hover {
        opacity: 0.95;
        transform: translateY(-1px);
        box-shadow: 0 6px 16px rgba(239, 68, 68, 0.3);
      }
      .download-progress {
        margin-top: 18px;
        padding: 14px;
        background: rgba(255, 255, 255, 0.03);
        border: 1px solid rgba(255, 255, 255, 0.05);
        border-radius: 12px;
      }
      .progress-bar {
        height: 8px;
        background: rgba(255, 255, 255, 0.06);
        border-radius: 9999px;
        overflow: hidden;
        margin-bottom: 8px;
      }
      .progress-fill {
        height: 100%;
        background: linear-gradient(90deg, #10b981 0%, #059669 100%);
        border-radius: 9999px;
        transition: width 0.3s ease;
      }
      .progress-text {
        text-align: center;
        font-weight: 700;
        font-size: 0.85rem;
        color: #10b981;
        margin-bottom: 10px;
      }
      .download-config {
        font-size: 0.75rem;
        color: #94a3b8;
        display: grid;
        grid-template-columns: 1fr 1fr;
        gap: 6px;
        background: rgba(0, 0, 0, 0.2);
        padding: 10px;
        border-radius: 8px;
        border: 1px solid rgba(255, 255, 255, 0.03);
      }
      .download-status {
        margin-top: 12px;
        padding: 10px;
        background: rgba(255, 255, 255, 0.03);
        border-radius: 8px;
        font-size: 0.8rem;
        font-weight: 600;
        text-transform: uppercase;
        letter-spacing: 0.05em;
        text-align: center;
        color: #94a3b8;
      }
      .download-status.error {
        background: rgba(239, 68, 68, 0.1);
        color: #f87171;
        border: 1px solid rgba(239, 68, 68, 0.2);
      }
      .download-status.success {
        background: rgba(16, 185, 129, 0.1);
        color: #34d399;
        border: 1px solid rgba(16, 185, 129, 0.2);
      }
      .retry-section, .clear-section {
        margin-top: 12px;
        text-align: center;
      }
      .retry-button, .clear-button {
        width: 100%;
        padding: 10px;
        border: none;
        border-radius: 8px;
        cursor: pointer;
        font-weight: 600;
        font-size: 0.85rem;
        transition: all 0.2s ease;
      }
      .retry-button {
        background: linear-gradient(135deg, #10b981 0%, #059669 100%);
        color: white;
        box-shadow: 0 4px 12px rgba(16, 185, 129, 0.2);
      }
      .retry-button:hover {
        opacity: 0.95;
        transform: translateY(-1px);
      }
      .clear-button {
        background: rgba(255, 255, 255, 0.06);
        color: #cbd5e1;
        border: 1px solid rgba(255, 255, 255, 0.08);
      }
      .clear-button:hover {
        background: rgba(255, 255, 255, 0.1);
        color: white;
      }
      .dual-range-container {
        position: relative;
        width: 100%;
        height: 30px;
        margin-top: 6px;
        margin-bottom: 12px;
      }
      .slider-track {
        position: absolute;
        top: 12px;
        left: 0;
        width: 100%;
        height: 6px;
        border-radius: 9999px;
        z-index: 1;
      }
      .range-slider {
        position: absolute;
        top: 5px;
        left: 0;
        width: 100%;
        background: none;
        pointer-events: none;
        -webkit-appearance: none;
        appearance: none;
        margin: 0 !important;
        padding: 0 !important;
        height: 20px;
        border: none !important;
      }
      .range-slider::-webkit-slider-runnable-track {
        background: none;
        border: none;
      }
      .range-slider::-moz-range-track {
        background: none;
        border: none;
      }
      .range-slider::-webkit-slider-thumb {
        -webkit-appearance: none;
        appearance: none;
        width: 18px;
        height: 18px;
        background: #6366f1;
        border-radius: 50%;
        cursor: pointer;
        pointer-events: auto;
        transition: background-color 0.2s, transform 0.1s;
        box-shadow: 0 2px 6px rgba(0, 0, 0, 0.4);
        margin: 0;
      }
      .range-slider::-moz-range-thumb {
        width: 18px;
        height: 18px;
        background: #6366f1;
        border-radius: 50%;
        cursor: pointer;
        pointer-events: auto;
        transition: background-color 0.2s, transform 0.1s;
        box-shadow: 0 2px 6px rgba(0, 0, 0, 0.4);
        border: none;
      }
      .range-slider::-webkit-slider-thumb:hover {
        background: #818cf8;
        transform: scale(1.1);
      }
      .range-slider::-moz-range-thumb:hover {
        background: #818cf8;
        transform: scale(1.1);
      }
    </style>
    <div class="map-container">
      <div id="map"></div>
    </div>
  `;
	return template;
}

async function handleRetryDownload() {
	if (!currentDownloadState || !map) return;

	// Restore the previous download configuration
	const center = L.latLng(currentDownloadState.lat, currentDownloadState.lon);
	if (downloadCircle) {
		map.removeLayer(downloadCircle);
	}
	downloadCircle = L.circle(center, {
		radius: currentDownloadState.radius * 1000,
		color: '#3388ff',
		fillColor: '#3388ff',
		fillOpacity: 0.2,
		weight: 2,
		dashArray: '5, 10',
	}).addTo(map);

	// Start the download
	await handleDownload();
}

async function handleDownload() {
	if (!map || !downloadCircle) return;

	const center = downloadCircle.getLatLng();
	const radius = downloadCircle.getRadius();

	// Initialize download state immediately
	currentDownloadState = {
		id: '', // Will be set when we get the response
		layer: currentLayer,
		lat: center.lat,
		lon: center.lng,
		minZoom,
		maxZoom,
		radius: radius / 1000,
		downloaded: 0,
		total: 0,
		status: 'pending'
	};
	saveState();
	renderControlContent();

	try {
		const response = await fetch('/api/download', {
			method: 'POST',
			headers: {
				'Content-Type': 'application/json',
			},
			body: JSON.stringify({
				layer: currentLayer,
				lat: center.lat,
				lon: center.lng,
				radiusKm: radius / 1000,
				minZoom,
				maxZoom,
			}),
		});

		if (!response.ok) {
			throw new Error('Download request failed');
		}

		const result = await response.json();
		if (!result.success) {
			throw new Error(result.error || 'Download failed');
		}

		if (result.jobId && currentDownloadState) {
			currentDownloadState.id = result.jobId;
			currentDownloadState.status = 'processing';
			saveState();
			renderControlContent();
			startStatusPolling(result.jobId);
		}
	} catch (error) {
		console.error('Error starting download:', error);
		if (currentDownloadState) {
			currentDownloadState.status = 'failed';
			currentDownloadState.error = error instanceof Error ? error.message : 'Unknown error';
			saveState();
			renderControlContent();
		}
	}
}

async function handleCancelDownload() {
	if (!currentDownloadState) return;

	try {
		const response = await fetch(`/api/download/${currentDownloadState.id}/cancel`, {
			method: 'POST'
		});

		if (!response.ok) {
			throw new Error('Failed to cancel download');
		}

		const result = await response.json();
		if (result.success) {
			if (statusPollInterval) {
				clearInterval(statusPollInterval);
				statusPollInterval = null;
			}
			currentDownloadState.status = 'cancelled';
			saveState();
			renderControlContent();
		}
	} catch (error) {
		console.error('Error cancelling download:', error);
	}
}

function handleSliderPointerMove(e: PointerEvent) {
	// If the user is currently dragging/pressing, do not swap z-index to avoid interrupting active gesture
	if (e.buttons > 0) return;

	const containerEl = e.currentTarget as HTMLElement;
	const rect = containerEl.getBoundingClientRect();
	const clickX = e.clientX - rect.left;
	const pct = Math.max(0, Math.min(1, clickX / rect.width));
	const clickVal = pct * 19;
	const mid = (minZoom + maxZoom) / 2;
	let newMinZ = 2;
	let newMaxZ = 2;

	if (clickVal < mid) {
		newMinZ = 3;
		newMaxZ = 2;
	} else {
		newMinZ = 2;
		newMaxZ = 3;
	}

	if (zIndexMin !== newMinZ || zIndexMax !== newMaxZ) {
		zIndexMin = newMinZ;
		zIndexMax = newMaxZ;
		renderControlContent();
	}
}

function handleSliderPointerDown(e: PointerEvent) {
	const containerEl = e.currentTarget as HTMLElement;
	const rect = containerEl.getBoundingClientRect();
	const clickX = e.clientX - rect.left;
	const pct = Math.max(0, Math.min(1, clickX / rect.width));
	const clickVal = pct * 19;
	const mid = (minZoom + maxZoom) / 2;
	let newMinZ = 2;
	let newMaxZ = 2;

	if (clickVal < mid) {
		newMinZ = 3;
		newMaxZ = 2;
	} else {
		newMinZ = 2;
		newMaxZ = 3;
	}

	if (zIndexMin !== newMinZ || zIndexMax !== newMaxZ) {
		zIndexMin = newMinZ;
		zIndexMax = newMaxZ;
		renderControlContent();
	}
}

function handleMinSliderInput(e: Event) {
	const input = e.target as HTMLInputElement;
	const val = parseInt(input.value) || 0;
	if (val > maxZoom) {
		input.value = maxZoom.toString();
		minZoom = maxZoom;
	} else {
		minZoom = val;
	}
	saveState();
	renderControlContent();
}

function handleMaxSliderInput(e: Event) {
	const input = e.target as HTMLInputElement;
	const val = parseInt(input.value) || 0;
	if (val < minZoom) {
		input.value = minZoom.toString();
		maxZoom = minZoom;
	} else {
		maxZoom = val;
	}
	saveState();
	renderControlContent();
}

function handleRadiusChange(e: Event) {
	const input = e.target as HTMLInputElement;
	radius = parseFloat(input.value);
	if (downloadCircle) {
		downloadCircle.setRadius(radius * 1000);
		renderControlContent();
	}
}

function handleMapClick(e: L.LeafletMouseEvent) {
	if (!map) return;

	// Don't create a new circle if we're currently downloading
	if (currentDownloadState?.status === 'processing') {
		return;
	}

	if (downloadCircle) {
		map.removeLayer(downloadCircle);
	}

	downloadCircle = L.circle(e.latlng, {
		radius: radius * 1000,
		color: '#3388ff',
		fillColor: '#3388ff',
		fillOpacity: 0.2,
		weight: 2,
		dashArray: '5, 10',
	}).addTo(map);

	renderControlContent();
}

const DownloadControl = L.Control.extend({
	options: {
		position: 'topright'
	},

	onAdd: function (map: L.Map) {
		container = L.DomUtil.create('div', 'leaflet-bar leaflet-control download-control');

		L.DomEvent.disableClickPropagation(container);
		L.DomEvent.disableScrollPropagation(container);

		renderControlContent();
		return container;
	}
});

function handleLayerChange(e: Event) {
	const select = e.target as HTMLSelectElement;
	currentLayer = select.value;
	if (tileLayerInstance) {
		tileLayerInstance.setUrl(`/tiles/${currentLayer}/{z}/{x}/{y}`);
	}
	saveState();
	renderControlContent();
}

export function render_map(element: HTMLElement) {
	const template = updateTemplate();
	render(template, element);

	if (!map) {
		map = L.map(element, {
			minZoom,
			maxZoom,
		}).setView([0, 0], 2);

		tileLayerInstance = L.tileLayer(`/tiles/${currentLayer}/{z}/{x}/{y}`, {
			maxZoom,
			tileSize: 256
		}).addTo(map);

		map.on('click', handleMapClick);
		map.on('moveend', saveState);
		new DownloadControl().addTo(map);

		restoreState();
	}
}


render_map(rootContainer);
