<script lang="ts">
	import { createSseResource } from '$lib/realtime/sse';
	import type { OverviewSnapshot, TimedValue, TrajectoryPoint } from '$lib/realtime/types';

	const overview = createSseResource<OverviewSnapshot>('/overview/api?sse=1', {
		event: 'overview',
		staleAfterMs: 5_000,
		offlineAfterMs: 60_000
	});

	let clock = $state(Date.now());
	$effect(() => {
		const timer = setInterval(() => (clock = Date.now()), 1_000);
		return () => clearInterval(timer);
	});

	function value<T>(timed: TimedValue<T> | undefined): T | null {
		return timed?.value ?? null;
	}

	function age(timestamp: number | null | undefined): string {
		if (!timestamp) return 'NO DATA';
		const seconds = Math.max(0, Math.floor((clock - timestamp * 1000) / 1000));
		if (seconds < 2) return 'NOW';
		if (seconds < 60) return `${seconds}S AGO`;
		if (seconds < 3600) return `${Math.floor(seconds / 60)}M AGO`;
		return `${Math.floor(seconds / 3600)}H AGO`;
	}

	function duration(timestamp: number | null | undefined): string {
		if (!timestamp) return '--:--:--';
		const seconds = Math.max(0, Math.floor((clock - timestamp * 1000) / 1000));
		const hours = Math.floor(seconds / 3600)
			.toString()
			.padStart(2, '0');
		const minutes = Math.floor((seconds % 3600) / 60)
			.toString()
			.padStart(2, '0');
		const secs = (seconds % 60).toString().padStart(2, '0');
		return `${hours}:${minutes}:${secs}`;
	}

	function fixed(number: number | null, digits = 1): string {
		return number === null || !Number.isFinite(number) ? '---' : number.toFixed(digits);
	}

	function path(points: TrajectoryPoint[]): string {
		if (points.length < 2) return '';
		const lats = points.map((point) => point.lat);
		const lons = points.map((point) => point.lon);
		const minLat = Math.min(...lats);
		const maxLat = Math.max(...lats);
		const minLon = Math.min(...lons);
		const maxLon = Math.max(...lons);
		const latRange = Math.max(maxLat - minLat, 0.000001);
		const lonRange = Math.max(maxLon - minLon, 0.000001);
		return points
			.map((point, index) => {
				const x = 5 + ((point.lon - minLon) / lonRange) * 90;
				const y = 95 - ((point.lat - minLat) / latRange) * 90;
				return `${index === 0 ? 'M' : 'L'} ${x.toFixed(2)} ${y.toFixed(2)}`;
			})
			.join(' ');
	}

	const connectionClass = $derived(
		$overview.status === 'live'
			? 'rgs-status--good'
			: $overview.status === 'stale' || $overview.status === 'connecting'
				? 'rgs-status--warn'
				: 'rgs-status--danger'
	);
</script>

<svelte:head>
	<title>Mission Overview | RGS</title>
	<meta name="description" content="Ground station mission and system overview" />
</svelte:head>

<div class="rgs-page overview-page">
	<header class="rgs-page-header">
		<div>
			<h1 class="rgs-page-title">Mission Overview</h1>
			<div class="rgs-muted subtitle">REALTIME FLIGHT AND GROUND SYSTEM STATUS</div>
		</div>
		<div class="rgs-status {connectionClass}">{$overview.status}</div>
	</header>

	{#if !$overview.data}
		<div class="loading-grid" aria-label="Loading mission overview">
			{#each Array(9) as _}<div class="skeleton"></div>{/each}
		</div>
	{:else}
		{@const snapshot = $overview.data}
		<section class="phase-banner" aria-label="Current flight state">
			<div class="phase-kicker">CURRENT FLIGHT STATE</div>
			<div class="phase-value">{snapshot.flight.state?.value ?? 'NO STATE'}</div>
			<div class="phase-meta">
				<span>T+ {duration(snapshot.flight.state?.timestamp)}</span>
				<span>EVENT {snapshot.flight.event?.value ?? '---'}</span>
				<span>UPDATED {age(snapshot.flight.state?.timestamp)}</span>
			</div>
		</section>

		<section class="metric-grid" aria-label="Critical telemetry">
			<a class="metric" href="/altitude">
				<span class="metric-label">BARO ALTITUDE</span>
				<span class="rgs-value"
					>{fixed(value(snapshot.flight.altitudeM))}<span class="rgs-unit">M</span></span
				>
				<span class="metric-age">{age(snapshot.flight.altitudeM.updatedAt)}</span>
			</a>
			<a class="metric" href="/imu">
				<span class="metric-label">VERT VELOCITY</span>
				<span class="rgs-value"
					>{fixed(value(snapshot.flight.verticalVelocityMps))}<span class="rgs-unit">M/S</span
					></span
				>
				<span class="metric-age">{age(snapshot.flight.verticalVelocityMps.updatedAt)}</span>
			</a>
			<a class="metric" href="/imu">
				<span class="metric-label">ACCELERATION</span>
				<span class="rgs-value"
					>{fixed(value(snapshot.flight.accelerationMps2), 2)}<span class="rgs-unit">M/S²</span
					></span
				>
				<span class="metric-age">{age(snapshot.flight.accelerationMps2.updatedAt)}</span>
			</a>
			<a class="metric" href="/radio">
				<span class="metric-label">RADIO RSSI</span>
				<span class="rgs-value"
					>{fixed(snapshot.radio.rssiDbm, 0)}<span class="rgs-unit">DBM</span></span
				>
				<span class="metric-age">{age(snapshot.radio.updatedAt)}</span>
			</a>
			<a class="metric" href="/radio">
				<span class="metric-label">PACKET LOSS</span>
				<span class="rgs-value"
					>{fixed(snapshot.radio.packetLossPercent, 2)}<span class="rgs-unit">%</span></span
				>
				<span class="metric-age">{snapshot.radio.framesPerSecond.toFixed(1)} FPS</span>
			</a>
			<a class="metric" href="/trajectory">
				<span class="metric-label">GNSS SATELLITES</span>
				<span class="rgs-value"
					>{fixed(value(snapshot.flight.satellites), 0)}<span class="rgs-unit">SV</span></span
				>
				<span class="metric-age">{age(snapshot.flight.satellites.updatedAt)}</span>
			</a>
		</section>

		<div class="overview-grid">
			<a class="rgs-panel trajectory-panel" href="/trajectory">
				<div class="rgs-panel-header">
					<h2>Trajectory</h2>
					<span>OPEN MAP →</span>
				</div>
				<div class="trajectory-body">
					{#if snapshot.trajectory.length > 1}
						<svg
							viewBox="0 0 100 100"
							preserveAspectRatio="none"
							role="img"
							aria-label="Recent flight trajectory"
						>
							<defs
								><pattern id="grid" width="10" height="10" patternUnits="userSpaceOnUse"
									><path
										d="M 10 0 L 0 0 0 10"
										fill="none"
										stroke="currentColor"
										stroke-width="0.2"
									/></pattern
								></defs
							>
							<rect width="100" height="100" fill="url(#grid)" />
							<path class="track" d={path(snapshot.trajectory)} />
						</svg>
						<div class="position-readout">
							<span>LAT {value(snapshot.flight.position)?.lat.toFixed(6)}</span>
							<span>LON {value(snapshot.flight.position)?.lon.toFixed(6)}</span>
						</div>
					{:else}
						<div class="rgs-empty">NO VALID POSITION TRACK</div>
					{/if}
				</div>
			</a>

			<section class="rgs-panel health-panel">
				<div class="rgs-panel-header"><h2>System Health</h2></div>
				<div class="health-list">
					<a href="/radio" class="health-row">
						<span>RADIO LINK</span><span
							class="rgs-status {snapshot.radio.framesPerSecond > 0
								? 'rgs-status--good'
								: 'rgs-status--danger'}"
							>{snapshot.radio.framesPerSecond > 0 ? 'RECEIVING' : 'NO FRAMES'}</span
						>
					</a>
					<a href="/services" class="health-row">
						<span>SERVICES</span><span
							class="rgs-status {snapshot.services.available &&
							snapshot.services.degraded.length === 0
								? 'rgs-status--good'
								: 'rgs-status--warn'}"
							>{snapshot.services.available
								? `${snapshot.services.running}/${snapshot.services.total} RUNNING`
								: 'UNAVAILABLE'}</span
						>
					</a>
					<div class="health-row">
						<span>GNSS</span><span
							class="rgs-status {value(snapshot.flight.position)
								? 'rgs-status--good'
								: 'rgs-status--warn'}"
							>{value(snapshot.flight.position) ? 'FIX VALID' : 'NO FIX'}</span
						>
					</div>
					<div class="health-row">
						<span>TELEMETRY</span><span class="rgs-status {connectionClass}"
							>{$overview.status}</span
						>
					</div>
				</div>
				{#if snapshot.services.degraded.length}
					<div class="degraded">
						<strong>ATTENTION</strong>{snapshot.services.degraded.join(', ')}
					</div>
				{/if}
			</section>

			<section class="rgs-panel timeline-panel">
				<div class="rgs-panel-header">
					<h2>Recent Transitions</h2>
					<a href="/state">VIEW HISTORY →</a>
				</div>
				{#if snapshot.recentTransitions.length}
					<ol class="timeline">
						{#each snapshot.recentTransitions.slice(0, 8) as transition}
							<li>
								<span class="transition-kind">{transition.kind}</span><strong
									>{transition.value}</strong
								><time>{age(transition.timestamp)}</time>
							</li>
						{/each}
					</ol>
				{:else}<div class="rgs-empty">NO FLIGHT TRANSITIONS RECORDED</div>{/if}
			</section>
		</div>
	{/if}
</div>

<style>
	.overview-page {
		min-height: calc(100vh - var(--rgs-header-h));
	}
	.subtitle {
		margin-top: 2px;
		font-size: 0.68rem;
		letter-spacing: 0.08em;
	}
	.phase-banner {
		display: grid;
		grid-template-columns: 190px minmax(180px, 1fr) auto;
		align-items: center;
		min-height: 64px;
		border: 2px solid var(--rgs-text);
		background: var(--rgs-surface);
		padding: 8px 12px;
	}
	.phase-kicker {
		color: var(--rgs-text-muted);
		font-size: 0.72rem;
		letter-spacing: 0.1em;
	}
	.phase-value {
		font-size: clamp(1.5rem, 3vw, 2.25rem);
		font-weight: 600;
		letter-spacing: 0.03em;
	}
	.phase-meta {
		display: flex;
		gap: 16px;
		font-size: 0.72rem;
	}
	.metric-grid {
		display: grid;
		grid-template-columns: repeat(6, minmax(0, 1fr));
		gap: 1px;
		border: 1px solid var(--rgs-border);
		background: var(--rgs-border);
	}
	.metric {
		display: flex;
		min-width: 0;
		min-height: 104px;
		flex-direction: column;
		gap: 6px;
		padding: 10px;
		background: var(--rgs-surface);
		color: var(--rgs-text);
		text-decoration: none;
	}
	.metric:hover {
		background: var(--rgs-surface-active);
	}
	.metric-label,
	.metric-age {
		color: var(--rgs-text-muted);
		font-size: 0.68rem;
		letter-spacing: 0.06em;
	}
	.metric-age {
		margin-top: auto;
	}
	.overview-grid {
		display: grid;
		min-height: 0;
		grid-template-columns: 2fr 1fr;
		gap: 8px;
	}
	.trajectory-panel {
		color: var(--rgs-text);
		text-decoration: none;
	}
	.trajectory-panel .rgs-panel-header span,
	.timeline-panel .rgs-panel-header a {
		color: var(--rgs-accent);
		font-size: 0.68rem;
		text-decoration: none;
	}
	.trajectory-body {
		position: relative;
		min-height: 290px;
	}
	.trajectory-body svg {
		position: absolute;
		inset: 0;
		width: 100%;
		height: 100%;
		color: var(--rgs-border-subtle);
		background: var(--rgs-surface);
	}
	.track {
		fill: none;
		stroke: var(--rgs-accent);
		stroke-width: 1.8;
		vector-effect: non-scaling-stroke;
	}
	.position-readout {
		position: absolute;
		right: 8px;
		bottom: 8px;
		display: flex;
		gap: 12px;
		padding: 5px 8px;
		border: 1px solid var(--rgs-border);
		background: var(--rgs-surface);
		font-size: 0.7rem;
	}
	.health-list {
		display: flex;
		flex-direction: column;
	}
	.health-row {
		display: flex;
		min-height: 42px;
		align-items: center;
		justify-content: space-between;
		gap: 8px;
		padding: 6px 10px;
		border-bottom: 1px solid var(--rgs-border-subtle);
		color: var(--rgs-text);
		text-decoration: none;
	}
	.health-row:hover {
		background: var(--rgs-surface-alt);
	}
	.degraded {
		display: flex;
		flex-direction: column;
		gap: 4px;
		margin: 8px;
		padding: 8px;
		border-left: 3px solid var(--rgs-warn);
		background: var(--rgs-surface-alt);
		color: var(--rgs-warn);
		font-size: 0.75rem;
	}
	.timeline-panel {
		grid-column: 1 / -1;
	}
	.timeline {
		display: grid;
		grid-template-columns: repeat(2, minmax(0, 1fr));
		margin: 0;
		padding: 0;
		list-style: none;
	}
	.timeline li {
		display: grid;
		grid-template-columns: 58px 1fr auto;
		gap: 8px;
		padding: 7px 10px;
		border-bottom: 1px solid var(--rgs-border-subtle);
	}
	.timeline li:nth-child(odd) {
		border-right: 1px solid var(--rgs-border-subtle);
	}
	.transition-kind,
	.timeline time {
		color: var(--rgs-text-muted);
		font-size: 0.7rem;
		text-transform: uppercase;
	}
	.loading-grid {
		display: grid;
		grid-template-columns: repeat(3, 1fr);
		gap: 8px;
	}
	.skeleton {
		min-height: 120px;
		background: linear-gradient(
			90deg,
			var(--rgs-surface-alt),
			var(--rgs-surface),
			var(--rgs-surface-alt)
		);
		background-size: 200% 100%;
		animation: shimmer 1.2s infinite linear;
	}
	@keyframes shimmer {
		to {
			background-position: -200% 0;
		}
	}
	@media (max-width: 1100px) {
		.metric-grid {
			grid-template-columns: repeat(3, 1fr);
		}
		.overview-grid {
			grid-template-columns: 1fr 1fr;
		}
		.trajectory-panel {
			grid-column: 1 / -1;
		}
	}
	@media (max-width: 767px) {
		.phase-banner {
			grid-template-columns: 1fr;
			gap: 4px;
		}
		.phase-meta {
			flex-wrap: wrap;
			gap: 6px 12px;
		}
		.metric-grid {
			grid-template-columns: repeat(2, 1fr);
		}
		.metric {
			min-height: 96px;
		}
		.overview-grid,
		.timeline {
			grid-template-columns: 1fr;
		}
		.trajectory-panel,
		.timeline-panel {
			grid-column: auto;
		}
		.trajectory-body {
			min-height: 220px;
		}
		.timeline li:nth-child(odd) {
			border-right: 0;
		}
		.loading-grid {
			grid-template-columns: 1fr 1fr;
		}
	}
</style>
