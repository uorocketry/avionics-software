<script lang="ts">
	import { onMount, onDestroy } from 'svelte';

	type Item = {
		id: number;
		kind: 'State' | 'Event';
		value: string;
		ts: number | null;
		duration_s: number | null;
	};

	let items = $state<Item[]>([]);
	let connected = $state(false);
	let error = $state<string | null>(null);
	let eventSource: EventSource | null = null;
	let filter = $state<'All' | 'State' | 'Event'>('All');
	let visibleItems = $derived(
		filter === 'All' ? items : items.filter((item) => item.kind === filter)
	);

	function connect() {
		try {
			const es = new EventSource('/state/api');
			eventSource = es;
			connected = true;
			error = null;

			es.addEventListener('snapshot', (e: MessageEvent) => {
				try {
					const newItems = JSON.parse(e.data) as Item[];
					// Deduplicate items by id and timestamp to prevent key conflicts
					const seen = new Set<string>();
					items = newItems.filter((item) => {
						const key = `${item.id}-${item.ts}`;
						if (seen.has(key)) return false;
						seen.add(key);
						return true;
					});
				} catch {}
			});

			es.addEventListener('append', (e: MessageEvent) => {
				try {
					const newItems = JSON.parse(e.data) as Item[];
					// Deduplicate and merge new items
					const seen = new Set<string>();
					const existingKeys = new Set(items.map((item) => `${item.id}-${item.ts}`));

					const uniqueNewItems = newItems.filter((item) => {
						const key = `${item.id}-${item.ts}`;
						if (seen.has(key) || existingKeys.has(key)) return false;
						seen.add(key);
						return true;
					});

					items = [...uniqueNewItems, ...items].slice(0, 100);
				} catch {}
			});

			es.onerror = (e) => {
				console.error('SSE error', e);
				connected = false;
				error = 'Connection lost';
				es.close();
			};
		} catch (e) {
			error = 'Failed to connect';
			connected = false;
		}
	}

	onMount(connect);
	onDestroy(() => eventSource?.close());

	function formatTs(ts: number | null): string {
		if (!ts) return 'N/A';
		const d = new Date(ts * 1000);
		return d.toLocaleString();
	}

	function formatDuration(seconds: number | null): string {
		if (seconds == null || !isFinite(seconds)) return '—';
		const s = Math.max(0, Math.floor(seconds));
		const hrs = Math.floor(s / 3600);
		const mins = Math.floor((s % 3600) / 60);
		const secs = s % 60;
		if (hrs > 0) return `${hrs}h ${mins}m ${secs}s`;
		if (mins > 0) return `${mins}m ${secs}s`;
		return `${secs}s`;
	}

	// Create unique keys for each items to prevent Svelte key conflicts
	function getUniqueKey(item: Item, index: number): string {
		return `${item.id}-${item.ts}-${index}`;
	}
</script>

<svelte:head>
	<title>Flight State</title>
</svelte:head>

<div class="rgs-page">
	<header class="rgs-page-header">
		<div>
			<h1 class="rgs-page-title">Flight State</h1>
			<div class="rgs-muted page-subtitle">STATE MACHINE AND EVENT TRANSITIONS</div>
		</div>
		<div
			class="rgs-status {connected
				? 'rgs-status--good'
				: error
					? 'rgs-status--danger'
					: 'rgs-status--warn'}"
		>
			{connected ? 'LIVE' : (error ?? 'CONNECTING')}
		</div>
	</header>

	<section class="current-state">
		<div>
			<span class="label">CURRENT</span><strong
				>{items.find((item) => item.kind === 'State')?.value ?? 'NO STATE'}</strong
			>
		</div>
		<div>
			<span class="label">LATEST EVENT</span><strong
				>{items.find((item) => item.kind === 'Event')?.value ?? 'NO EVENT'}</strong
			>
		</div>
		<div>
			<span class="label">LAST UPDATE</span><strong
				>{items[0] ? formatTs(items[0].ts) : 'WAITING'}</strong
			>
		</div>
	</section>

	<section class="rgs-panel">
		<div class="rgs-panel-header">
			<h2>Transition Timeline</h2>
			<div class="filter" aria-label="Filter transitions">
				{#each ['All', 'State', 'Event'] as option}
					<button
						class:active={filter === option}
						type="button"
						onclick={() => (filter = option as typeof filter)}>{option}</button
					>
				{/each}
			</div>
		</div>
		{#if visibleItems.length}
			<ol class="transition-list">
				{#each visibleItems as item, index (getUniqueKey(item, index))}
					<li class:event={item.kind === 'Event'}>
						<span class="marker"></span>
						<span class="kind">{item.kind}</span>
						<strong>{item.value}</strong>
						<time>{formatTs(item.ts)}</time>
						<span class="duration">{formatDuration(item.duration_s)}</span>
					</li>
				{/each}
			</ol>
		{:else}<div class="rgs-empty">NO TRANSITIONS RECEIVED</div>{/if}
	</section>
</div>

<style>
	.page-subtitle {
		margin-top: 2px;
		font-size: 0.68rem;
		letter-spacing: 0.08em;
	}
	.current-state {
		display: grid;
		grid-template-columns: 1fr 1fr 1.5fr;
		border: 1px solid var(--rgs-border);
		background: var(--rgs-surface);
	}
	.current-state > div {
		display: flex;
		min-height: 64px;
		flex-direction: column;
		justify-content: center;
		gap: 5px;
		padding: 8px 12px;
		border-right: 1px solid var(--rgs-border-subtle);
	}
	.current-state > div:last-child {
		border-right: 0;
	}
	.current-state strong {
		font-size: 1.2rem;
	}
	.label,
	.kind,
	.duration {
		color: var(--rgs-text-muted);
		font-size: 0.68rem;
		letter-spacing: 0.06em;
	}
	.filter {
		display: flex;
	}
	.filter button {
		height: 24px;
		padding: 0 8px;
		border: 1px solid var(--rgs-border);
		border-right: 0;
		background: var(--rgs-surface);
		color: var(--rgs-text);
		font-size: 0.7rem;
	}
	.filter button:last-child {
		border-right: 1px solid var(--rgs-border);
	}
	.filter button.active {
		background: var(--rgs-text);
		color: var(--rgs-surface);
	}
	.transition-list {
		margin: 0;
		padding: 0;
		list-style: none;
	}
	.transition-list li {
		display: grid;
		grid-template-columns: 14px 64px minmax(120px, 1fr) minmax(180px, auto) 72px;
		align-items: center;
		gap: 8px;
		min-height: 38px;
		padding: 3px 10px;
		border-bottom: 1px solid var(--rgs-border-subtle);
	}
	.transition-list li:last-child {
		border-bottom: 0;
	}
	.marker {
		width: 8px;
		height: 8px;
		border-radius: 50%;
		background: var(--rgs-good);
	}
	.event .marker {
		background: var(--rgs-accent);
	}
	time,
	.duration {
		font-variant-numeric: tabular-nums;
		text-align: right;
	}
	@media (max-width: 767px) {
		.current-state {
			grid-template-columns: 1fr 1fr;
		}
		.current-state > div:nth-child(2) {
			border-right: 0;
		}
		.current-state > div:last-child {
			grid-column: 1 / -1;
			border-top: 1px solid var(--rgs-border-subtle);
		}
		.transition-list li {
			grid-template-columns: 12px 55px 1fr auto;
			padding-block: 7px;
		}
		.transition-list time {
			grid-column: 3 / -1;
			text-align: left;
			color: var(--rgs-text-muted);
			font-size: 0.72rem;
		}
		.duration {
			grid-column: 4;
			grid-row: 1;
		}
	}
</style>
