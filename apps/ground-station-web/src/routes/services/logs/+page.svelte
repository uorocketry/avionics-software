<script lang="ts">
	import { onMount, onDestroy } from 'svelte';
	import {
		Grid,
		Row,
		Column,
		Tile,
		Dropdown,
		TextInput,
		NumberInput,
		Button,
		CodeSnippet,
		InlineLoading,
		Tag,
		InlineNotification
	} from 'carbon-components-svelte';
	import { toastStore } from '$lib/stores/toastStore';

	type Service = {
		name: string;
		status: 'running' | 'exited' | 'restarting' | 'unknown';
	};

	let services: Service[] = [];
	let selectedService: string | null = null;
	let logText = '';
	let logFilter = '';
	let logTail = 100;
	let streaming = false;
	let loading = false;
	let errorMsg: string | null = null;
	let eventSource: EventSource | null = null;
	let logEl: HTMLElement | null = null;

	// Maximum log buffer size to prevent memory issues
	const MAX_LOG_SIZE = 50000; // characters

	// Function to trim log text if it gets too long
	function trimLogText(text: string): string {
		if (text.length > MAX_LOG_SIZE) {
			const lines = text.split('\n');
			const trimmedLines = lines.slice(-1000); // Keep last 1000 lines
			return trimmedLines.join('\n');
		}
		return text;
	}

	// Fetch available services
	async function fetchServices() {
		loading = true;
		errorMsg = null;
		try {
			const res = await fetch('/services/api');
			if (!res.ok) throw new Error(await res.text());
			services = await res.json();
		} catch (err: any) {
			errorMsg = err.message ?? 'Failed to load services';
		} finally {
			loading = false;
		}
	}

	// Fetch static logs
	async function fetchLogs() {
		if (!selectedService) return;

		try {
			const params = new URLSearchParams({
				service: selectedService,
				tail: String(logTail)
			});
			if (logFilter) params.set('grep', logFilter);

			const res = await fetch(`/services/logs/api?${params.toString()}`);
			if (!res.ok) throw new Error(await res.text());

			logText = await res.text();
		} catch (e: any) {
			logText = `Failed to fetch logs: ${e.message}`;
			toastStore.error('Failed to fetch logs');
		}
	}

	// Start streaming logs
	function startStream() {
		if (!selectedService) return;

		stopStream();
		const params = new URLSearchParams({
			service: selectedService,
			tail: String(logTail)
		});
		if (logFilter) params.set('grep', logFilter);

		const url = `/services/logs/stream?${params.toString()}`;

		try {
			eventSource = new EventSource(url);
			logText = '';
			streaming = true;

			// Connection opened
			eventSource.onopen = () => {
				logText = `Connecting to ${selectedService} logs...\n`;
			};

			eventSource.onmessage = (ev) => {
				try {
					const data = JSON.parse(ev.data);

					switch (data.type) {
						case 'connected':
							logText = `Connected to ${selectedService} logs...\n`;
							break;
						case 'log':
							logText = trimLogText(logText + (logText ? '\n' : '') + data.line);
							break;
						case 'error':
							logText = trimLogText(logText + (logText ? '\n' : '') + `[ERROR] ${data.line}`);
							break;
						case 'closed':
							logText = trimLogText(
								logText + (logText ? '\n' : '') + `[STREAM CLOSED] Code: ${data.code}`
							);
							stopStream();
							break;
					}

					// Auto-scroll to bottom
					queueMicrotask(() => {
						logEl?.scrollTo({ top: logEl.scrollHeight, behavior: 'smooth' });
					});
				} catch (e) {
					// If JSON parsing fails, treat as plain log line
					logText += (logText ? '\n' : '') + ev.data;
				}
			};

			eventSource.onerror = (event) => {
				console.error('EventSource error:', event);
				logText = trimLogText(logText + (logText ? '\n' : '') + '[STREAM ERROR] Connection lost');
				stopStream();
			};
		} catch (e) {
			console.error('Error creating EventSource:', e);
			logText += (logText ? '\n' : '') + '[ERROR] Failed to start stream';
			streaming = false;
			toastStore.error('Failed to start log stream');
		}
	}

	// Stop streaming logs
	function stopStream() {
		if (eventSource) {
			eventSource.close();
			eventSource = null;
		}
		streaming = false;
	}

	// Toggle streaming
	function toggleStream() {
		streaming ? stopStream() : startStream();
	}

	// Clear logs
	function clearLogs() {
		logText = '';
	}

	// Auto-scroll toggle
	let autoScroll = true;
	function toggleAutoScroll() {
		autoScroll = !autoScroll;
	}

	// Service selection change handler
	function onServiceChange(serviceName: string) {
		selectedService = serviceName;
		stopStream();
		logText = '';
	}

	// Dropdown items from services
	$: serviceItems = services.map((s) => ({ id: s.name, text: s.name }));

	// Toast on error changes
	let lastError: string | null = null;
	$: {
		if (errorMsg && errorMsg !== lastError) {
			toastStore.error(`Logs: ${errorMsg}`, 0);
		}
		lastError = errorMsg;
	}

	onMount(() => {
		fetchServices();
	});

	onDestroy(() => {
		stopStream();
	});
</script>

<Grid padding={true} class="rgs-route">
	<Row>
		<Column>
			<h1>Service Logs</h1>
			<p>View and stream logs from Docker services</p>
		</Column>
	</Row>

	{#if errorMsg}
		<Row>
			<Column>
				<InlineNotification kind="error" title="Error" subtitle={errorMsg} hideCloseButton />
			</Column>
		</Row>
	{/if}

	<Row>
		<Column sm={16} md={8} lg={6}>
			<Tile>
				<h3>Configuration</h3>

				<Dropdown
					items={serviceItems}
					selectedId={selectedService}
					titleText="Select Service"
					itemToString={(i) => i?.text ?? ''}
					on:select={(e) => onServiceChange(e.detail.selectedItem?.id ?? '')}
					size="sm"
					disabled={loading}
				/>

				<TextInput
					labelText="Filter (regex)"
					bind:value={logFilter}
					placeholder="e.g. error|warn|INFO"
					size="sm"
					disabled={!selectedService}
				/>

				<NumberInput
					label="Tail lines"
					bind:value={logTail}
					min={10}
					max={1000}
					step={10}
					size="sm"
					disabled={!selectedService}
				/>

				<div style="margin-top: 1rem;">
					<Button
						size="small"
						kind="primary"
						on:click={fetchLogs}
						disabled={!selectedService || streaming}
					>
						Get Logs
					</Button>

					<Button
						size="small"
						kind={streaming ? 'danger' : 'tertiary'}
						on:click={toggleStream}
						disabled={!selectedService}
						style="margin-left: 0.5rem;"
					>
						{streaming ? 'Stop Stream' : 'Start Stream'}
					</Button>
				</div>

				<div style="margin-top: 0.5rem;">
					<Button size="small" kind="secondary" on:click={clearLogs} disabled={!logText}>
						Clear Logs
					</Button>

					<Button
						size="small"
						kind="tertiary"
						on:click={toggleAutoScroll}
						style="margin-left: 0.5rem;"
					>
						{autoScroll ? 'Disable' : 'Enable'} Auto-scroll
					</Button>
				</div>

				{#if selectedService}
					<div style="margin-top: 1rem;">
						<Tag type="blue" size="sm">
							Service: {selectedService}
						</Tag>
						{#if streaming}
							<Tag type="green" size="sm" style="margin-left: 0.5rem;">Streaming</Tag>
						{/if}
					</div>
				{/if}
			</Tile>
		</Column>

		<Column sm={16} md={8} lg={10}>
			<Tile>
				<div
					style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 1rem;"
				>
					<h3>Logs</h3>
					{#if streaming}
						<InlineLoading description="Streaming..." />
					{/if}
				</div>

				{#if !selectedService}
					<div style="text-align: center; padding: 2rem; color: var(--cds-text-secondary);">
						Select a service to view logs
					</div>
				{:else if !logText && !streaming}
					<div style="text-align: center; padding: 2rem; color: var(--cds-text-secondary);">
						Click "Get Logs" to fetch logs or "Start Stream" for real-time logs
					</div>
				{:else}
					<div
						bind:this={logEl}
						style="max-height: 70vh; overflow-y: auto; border: 1px solid var(--cds-border-subtle-01); border-radius: 4px;"
					>
						<CodeSnippet type="multi" wrapText>
							{logText || 'Connecting to stream...'}
						</CodeSnippet>
					</div>
				{/if}
			</Tile>
		</Column>
	</Row>
</Grid>

<svelte:head>
	<style>
		:global(.cds--code-snippet__content) {
			background: transparent !important;
		}
	</style>
</svelte:head>
