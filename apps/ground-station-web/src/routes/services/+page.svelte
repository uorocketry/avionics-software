<script lang="ts">
	import { onMount, onDestroy, tick } from 'svelte';
	import {
		Grid,
		Row,
		Column,
		Tile,
		Table,
		TableHead,
		TableRow,
		TableHeader,
		TableBody,
		TableCell,
		Button,
		Tag,
		Dropdown,
		TextInput,
		NumberInput,
		CodeSnippet
	} from 'carbon-components-svelte';
	import { toastStore } from '$lib/stores/toastStore';
	import ConfirmModal from '$lib/components/Common/ConfirmModal.svelte';

	type Service = {
		name: string;
		status: 'running' | 'exited' | 'restarting' | 'unknown';
	};

	let services: Service[] = [];
	let loading = false;
	let errorMsg: string | null = null;
	let confirmOpen = false;
	let pendingAction: { name: string; op: 'start' | 'stop' | 'restart' } | null = null;

	let selectedService: string | null = null;

	let logText = '';
	let logFilter = '';
	let logTail = 50;

	let eventSource: EventSource | null = null;
	let streaming = false;

	// The scrollable wrapper we control (bind below)
	let logWrapEl: HTMLDivElement | null = null;

	// Explicit pin toggle for auto-scroll behavior
	let pinToBottom = true;

	// Track changes to tail to re-trim existing logs immediately
	let lastAppliedTail = logTail;

	// Buffer writes to avoid layout thrash on heavy streams
	let buf: string[] = [];
	let flushScheduled = false;

	function scheduleFlush() {
		if (flushScheduled) return;
		flushScheduled = true;
		requestAnimationFrame(() => {
			flushScheduled = false;
			if (buf.length) {
				logText += (logText ? '\n' : '') + buf.join('\n');
				buf = [];
				logText = trimToTail(logText);
				// keep pinned to bottom if toggle is enabled
				if (pinToBottom) queueMicrotask(scrollToBottom);
			}
		});
	}

	function isAtBottom() {
		const el = logWrapEl;
		if (!el) return true;
		const slack = 4; // px tolerance
		return el.scrollTop + el.clientHeight >= el.scrollHeight - slack;
	}

	function scrollToBottom() {
		logWrapEl?.scrollTo({ top: logWrapEl.scrollHeight });
	}

	function trimToTail(text: string): string {
		const limit = Math.max(0, Math.floor(logTail));
		if (limit <= 0) return '';
		// Fast path: small text or already within limit
		let parts = text.split('\n');
		if (parts.length <= limit) return text;
		return parts.slice(-limit).join('\n');
	}

	function validateRegex(pattern: string): RegExp | null {
		if (!pattern) return null;
		try {
			// basic hardening: cap length to avoid pathological inputs
			const p = pattern.slice(0, 256);
			return new RegExp(p, 'i');
		} catch {
			toastStore.error('Invalid regex filter', 3000);
			return null;
		}
	}

	async function fetchServices() {
		loading = true;
		errorMsg = null;
		try {
			const res = await fetch('/services/api');
			if (!res.ok) throw new Error(await res.text());
			services = await res.json();
		} catch (err: any) {
			errorMsg = err?.message ?? 'Failed to load services';
		} finally {
			loading = false;
		}
	}

	function requestAction(name: string, op: 'start' | 'stop' | 'restart') {
		pendingAction = { name, op };
		confirmOpen = true;
	}

	async function runPendingAction() {
		if (!pendingAction) return;
		const { name, op } = pendingAction;
		pendingAction = null;
		loading = true;
		errorMsg = null;
		try {
			const res = await fetch(`/services/api?action=${op}&service=${encodeURIComponent(name)}`, {
				method: 'POST'
			});
			if (!res.ok) throw new Error(await res.text());
			await fetchServices();
		} catch (err: any) {
			errorMsg = err?.message ?? 'Operation failed';
		} finally {
			loading = false;
		}
	}

	onMount(fetchServices);
	onDestroy(() => stopStream());

	// --- Non-stream (one-shot) logs -----------------------------------------
	let lastLogsAbort: AbortController | null = null;

	async function fetchLogs() {
		if (!selectedService) return;
		lastLogsAbort?.abort();
		const ac = new AbortController();
		lastLogsAbort = ac;
		try {
			const params = new URLSearchParams({ service: selectedService, tail: String(logTail) });
			if (logFilter) params.set('grep', logFilter);
			const res = await fetch(`/services/logs/api?${params.toString()}`, { signal: ac.signal });
			if (!res.ok) throw new Error(await res.text());
			logText = trimToTail(await res.text());
			await tick();
			if (pinToBottom) scrollToBottom();
		} catch (e: any) {
			if (e?.name !== 'AbortError') {
				logText = 'Failed to fetch logs';
			}
		}
	}

	// When tail changes, trim currently displayed logs accordingly
	$: if (logTail !== lastAppliedTail) {
		lastAppliedTail = logTail;
		logText = trimToTail(logText);
	}

	// --- Streaming via SSE ---------------------------------------------------
	let reconnectAttempts = 0;
	const maxReconnectDelay = 30_000;

	function startStream() {
		if (!selectedService) return;
		stopStream();

		const params = new URLSearchParams({
			service: selectedService,
			tail: String(logTail)
		});
		// NOTE: we send filter to server for efficiency; we also keep client-side filter as a guard.
		if (logFilter) params.set('grep', logFilter);

		const url = `/services/logs/stream?${params.toString()}`;

		logText = '';
		buf = [];
		streaming = true;

		const re = validateRegex(logFilter); // may be null

		const es = new EventSource(url);
		eventSource = es;

		es.onmessage = (ev) => {
			let msg: any;
			try {
				msg = JSON.parse(ev.data);
			} catch {
				// Fallback: treat as plain line (older servers)
				if (
					!logFilter ||
					(re?.test(ev.data) ?? ev.data.toLowerCase().includes(logFilter.toLowerCase()))
				) {
					buf.push(ev.data);
					scheduleFlush();
				}
				return;
			}

			switch (msg?.type) {
				case 'connected':
					reconnectAttempts = 0;
					break;
				case 'heartbeat':
					// ignored (EventSource would also ignore ":" comment heartbeats)
					break;
				case 'log': {
					const line: string = String(msg.line ?? '');
					if (!line) break;
					if (re && !re.test(line)) break; // extra guard client-side
					buf.push(line);
					scheduleFlush();
					break;
				}
				case 'error': {
					const line: string = String(msg.line ?? 'unknown error');
					toastStore.error(`Log stream error: ${line}`, 4000);
					break;
				}
				case 'closed': {
					toastStore.info(`Stream closed (code ${msg.code ?? 'n/a'})`, 3000);
					stopStream(false); // no reconnect — server closed
					break;
				}
				default:
					// ignore unknown types
					break;
			}
		};

		es.onerror = () => {
			// network/server hiccup — try to reconnect with backoff
			stopStream(false);
			reconnectAttempts += 1;
			const delay = Math.min(maxReconnectDelay, 500 * 2 ** (reconnectAttempts - 1));
			setTimeout(() => {
				if (!streaming) return; // user may have stopped
				startStream();
			}, delay);
		};
	}

	function stopStream(clear = true) {
		if (eventSource) {
			eventSource.close();
			eventSource = null;
		}
		if (clear) {
			streaming = false;
		}
	}

	function toggleStream() {
		streaming ? stopStream() : startStream();
	}

	// Update dropdown items
	$: serviceItems = services.map((s) => ({ id: s.name, text: s.name }));

	// Switch stream when user changes service
	$: if (streaming && selectedService) {
		// restart stream for the new service
		startStream();
	}

	// Toast on error changes
	let lastError: string | null = null;
	$: {
		if (errorMsg && errorMsg !== lastError) {
			toastStore.error(`Services: ${errorMsg}`, 0);
		}
		lastError = errorMsg;
	}
</script>

<Grid padding={true} class="rgs-route">
	<Row>
		<Column>
			<div style="display: flex; justify-content: space-between; align-items: center;">
				<h1>Service Manager</h1>
				<a href="/services/logs" class="cds--link"> View Service Logs → </a>
			</div>
		</Column>
	</Row>

	<Row>
		<Column sm={16} md={8} lg={8}>
			<Tile>
				<Table>
					<TableHead>
						<TableRow>
							<TableHeader>Service</TableHeader>
							<TableHeader>Status</TableHeader>
							<TableHeader>Actions</TableHeader>
						</TableRow>
					</TableHead>
					<TableBody>
						{#if !loading && services.length === 0}
							<TableRow>
								<TableCell>No services found</TableCell>
								<TableCell></TableCell>
								<TableCell></TableCell>
							</TableRow>
						{:else}
							{#each services as s}
								<TableRow>
									<TableCell>{s.name}</TableCell>
									<TableCell>
										{#if s.status === 'running'}
											<Tag type="green">running</Tag>
										{:else if s.status === 'exited'}
											<Tag type="cool-gray">stopped</Tag>
										{:else if s.status === 'restarting'}
											<Tag type="high-contrast">restarting</Tag>
										{:else}
											<Tag type="gray">unknown</Tag>
										{/if}
									</TableCell>
									<TableCell>
										<div class="action-grid">
											{#if s.status === 'running'}
												<Button
													size="small"
													kind="danger"
													on:click={() => requestAction(s.name, 'stop')}>Stop</Button
												>
											{:else}
												<Button
													size="small"
													kind="secondary"
													on:click={() => requestAction(s.name, 'start')}>Start</Button
												>
											{/if}
											<Button
												size="small"
												kind="danger"
												on:click={() => requestAction(s.name, 'restart')}
												disabled={s.status !== 'running'}>Restart</Button
											>
											<Button
												size="small"
												kind="tertiary"
												on:click={() => {
													selectedService = s.name;
													startStream();
												}}>Logs</Button
											>
										</div>
									</TableCell>
								</TableRow>
							{/each}
						{/if}
					</TableBody>
				</Table>
				<Button size="small" kind="primary" on:click={fetchServices} disabled={loading}
					>Refresh</Button
				>
			</Tile>
		</Column>

		<Column sm={16} md={8} lg={8}>
			<Tile>
				<Row>
					<Column>
						<Dropdown
							items={serviceItems}
							selectedId={selectedService}
							titleText="Service"
							itemToString={(i) => i?.text ?? ''}
							on:select={(e) => {
								selectedService = e.detail.selectedItem?.id ?? null;
							}}
							size="sm"
						/>
					</Column>

					<Column>
						<TextInput
							labelText="Filter (regex)"
							bind:value={logFilter}
							placeholder="e.g. error|warn"
							size="sm"
						/>
					</Column>
				</Row>
				<Row>
					<Column>
						<NumberInput
							label="Tail lines"
							bind:value={logTail}
							min={10}
							max={500}
							step={5}
							size="sm"
						/>
					</Column>
				</Row>
				<Row>
					<Column>
						<Button size="small" kind="primary" on:click={fetchLogs} disabled={!selectedService}
							>Get logs</Button
						>
						<Button
							size="small"
							kind={streaming ? 'danger' : 'tertiary'}
							on:click={toggleStream}
							disabled={!selectedService}
						>
							{streaming ? 'Stop stream' : 'Start stream'}
						</Button>
						<Button
							size="small"
							kind={pinToBottom ? 'secondary' : 'tertiary'}
							on:click={() => {
								pinToBottom = !pinToBottom;
								if (pinToBottom) scrollToBottom();
							}}
							style="margin-left: 0.5rem;"
						>
							{pinToBottom ? 'Unpin' : 'Pin to bottom'}
						</Button>
					</Column>
				</Row>
				<Row>
					<Column>
						<!-- Wrap CodeSnippet to control scrolling explicitly -->
						<div class="logwrap" bind:this={logWrapEl}>
							<CodeSnippet type="multi" wrapText>{logText}</CodeSnippet>
						</div>
					</Column>
				</Row>
			</Tile>
		</Column>
	</Row>
</Grid>

<ConfirmModal
	bind:open={confirmOpen}
	title="Confirm service action"
	message={pendingAction
		? `${pendingAction.op.toUpperCase()} service ${pendingAction.name}? This can interrupt ground-station functionality.`
		: ''}
	confirmLabel={pendingAction?.op.toUpperCase() ?? 'CONFIRM'}
	danger={pendingAction?.op !== 'start'}
	onconfirm={runPendingAction}
/>

<svelte:head>
	<style>
		.action-grid {
			display: grid;
			grid-template-columns: 1fr 1fr;
			grid-template-areas:
				'main main'
				'left right';
			gap: var(--cds-spacing-02);
		}
		.action-grid > :nth-child(1) {
			grid-area: main;
		}
		.action-grid > :nth-child(2) {
			grid-area: left;
		}
		.action-grid > :nth-child(3) {
			grid-area: right;
		}

		@media (max-width: 768px) {
			.action-grid {
				grid-template-columns: 1fr;
				grid-template-areas:
					'main'
					'left'
					'right';
			}
		}

		/* Make logs scrollable and keep layout stable */
		.logwrap {
			max-height: 50vh;
			overflow: auto;
		}
	</style>
</svelte:head>
