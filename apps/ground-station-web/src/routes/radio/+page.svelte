<script lang="ts">
	import { onMount, onDestroy } from 'svelte';
	import {
		Grid,
		Row,
		Column,
		Tile,
		Button,
		InlineNotification,
		InlineLoading,
		Table,
		TableHead,
		TableRow,
		TableHeader,
		TableBody,
		TableCell,
		Tag,
		ProgressBar
	} from 'carbon-components-svelte';

	// Types for radio metrics
	type RadioMetric = {
		timestamp: number;
		rssi: number | null;
		packets_lost: number;
	};

	type Totals = {
		total_packets_lost: number;
		total_radio_frames: number;
		packet_loss_ratio: number;
		packet_loss_percentage: number;
		frames_per_second?: number;
	};

	// Reactive state
	let metrics: RadioMetric[] = $state([]);
	let totals: Totals | null = $state(null);
	let error: string | null = $state(null);
	let loading = $state(false);
	let connected = $state(false);
	let eventSource: EventSource | null = null;

	// Function to format timestamps
	function formatTimestamp(ts: number): string {
		try {
			return new Date(ts * 1000).toLocaleString();
		} catch (e) {
			return 'Invalid Date';
		}
	}

	// Function to get RSSI status and color
	function getRssiStatus(rssi: number | null): {
		status: string;
		color: 'green' | 'blue' | 'teal' | 'gray' | 'red';
	} {
		if (rssi === null) return { status: 'Unknown', color: 'gray' };
		if (rssi >= -50) return { status: 'Excellent', color: 'green' };
		if (rssi >= -60) return { status: 'Good', color: 'blue' };
		if (rssi >= -70) return { status: 'Fair', color: 'teal' };
		if (rssi >= -80) return { status: 'Poor', color: 'teal' };
		return { status: 'Very Poor', color: 'red' };
	}

	// Function to get packet loss status and color
	function getPacketLossStatus(lost: number): {
		status: string;
		color: 'green' | 'blue' | 'teal' | 'red';
	} {
		if (lost === 0) return { status: 'None', color: 'green' };
		if (lost <= 5) return { status: 'Low', color: 'blue' };
		if (lost <= 15) return { status: 'Medium', color: 'teal' };
		return { status: 'High', color: 'red' };
	}

	// Function to calculate packet loss percentage
	function getPacketLossPercentage(lost: number): number {
		// Assuming we're tracking packets over a window, this is a rough estimate
		// In a real implementation, you'd track total packets sent vs received
		return Math.min(100, Math.max(0, lost));
	}

	// Function to start SSE connection
	function startSSE() {
		if (eventSource) {
			eventSource.close();
		}

		eventSource = new EventSource('/radio/api?sse=1');
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
				if (Array.isArray(data)) {
					metrics = data;
					totals = null;
				} else if (data && data.metrics) {
					metrics = data.metrics;
					totals = data.totals ?? null;
				}
			} catch (e) {
				console.error('Error parsing SSE data:', e);
			}
		};

		eventSource.addEventListener('metrics', (event) => {
			try {
				const data = JSON.parse(event.data);
				if (Array.isArray(data)) {
					metrics = data;
					totals = null;
				} else if (data && data.metrics) {
					metrics = data.metrics;
					totals = data.totals ?? null;
				}
			} catch (e) {
				console.error('Error parsing metrics event:', e);
			}
		});

		eventSource.addEventListener('error', (event) => {
			console.error('SSE error:', event);
			error = 'Connection error. Attempting to reconnect...';
			connected = false;
		});

		eventSource.onerror = () => {
			error = 'Connection lost. Attempting to reconnect...';
			connected = false;
			// Auto-reconnect after a delay
			setTimeout(() => {
				if (eventSource && eventSource.readyState === EventSource.CLOSED) {
					startSSE();
				}
			}, 5000);
		};
	}

	// Function to stop SSE connection
	function stopSSE() {
		if (eventSource) {
			eventSource.close();
			eventSource = null;
		}
		connected = false;
	}

	// Function to refresh data manually
	async function refreshData() {
		loading = true;
		try {
			const res = await fetch('/radio/api');
			if (!res.ok) throw new Error(await res.text());
			const data = await res.json();
			if (Array.isArray(data)) {
				metrics = data;
				totals = null;
			} else {
				metrics = data.metrics ?? [];
				totals = data.totals ?? null;
			}
			error = null;
		} catch (e: any) {
			error = e.message;
		} finally {
			loading = false;
		}
	}

	// Lifecycle
	onMount(() => {
		startSSE();
	});

	onDestroy(() => {
		stopSSE();
	});
</script>

<Grid padding={true} class="rgs-route telemetry-route">
	<Row>
		<Column>
			<Tile>
				<div style="display: flex; justify-content: space-between; align-items: center;">
					<h2>Radio Metrics</h2>
					<div>
						<Button
							kind="secondary"
							size="default"
							on:click={refreshData}
							disabled={loading}
							title="Refresh Data"
						>
							{#if loading}
								<InlineLoading description="Refreshing..." />
							{:else}
								Refresh
							{/if}
						</Button>
						{#if connected}
							<Button
								kind="danger"
								size="default"
								on:click={stopSSE}
								style="margin-left: 0.5rem;"
								title="Stop Real-time Updates"
							>
								Stop Live
							</Button>
						{:else}
							<Button
								kind="primary"
								size="default"
								on:click={startSSE}
								style="margin-left: 0.5rem;"
								title="Start Real-time Updates"
							>
								Start Live
							</Button>
						{/if}
					</div>
				</div>

				{#if error}
					<InlineNotification
						kind="error"
						title="Error"
						subtitle={error}
						hideCloseButton
						style="margin: 1rem 0;"
					/>
				{/if}

				{#if metrics.length === 0 && !loading}
					<InlineNotification
						kind="info"
						title="No Data"
						subtitle="No radio metrics available"
						hideCloseButton
						style="margin: 1rem 0;"
					/>
				{/if}

				{#if metrics.length > 0}
					<!-- Summary Cards -->
					<Row style="margin: 1rem 0;">
						<Column>
							<Tile>
								<h3>Latest RSSI</h3>
								{#if metrics[0]?.rssi !== null}
									<div
										style="font-size: 2rem; font-weight: bold; color: {getRssiStatus(
											metrics[0].rssi
										).color === 'green'
											? '#24a148'
											: getRssiStatus(metrics[0].rssi).color === 'blue'
												? '#0f62fe'
												: getRssiStatus(metrics[0].rssi).color === 'teal'
													? '#009d9a'
													: getRssiStatus(metrics[0].rssi).color === 'red'
														? '#da1e28'
														: '#8d8d8d'};"
									>
										{metrics[0].rssi} dBm
									</div>
									<Tag type={getRssiStatus(metrics[0].rssi).color} size="sm">
										{getRssiStatus(metrics[0].rssi).status}
									</Tag>
								{:else}
									<div style="font-size: 2rem; font-weight: bold; color: gray;">N/A</div>
								{/if}
							</Tile>
						</Column>
						<Column>
							<Tile>
								<h3>Frames per second</h3>
								{#if totals?.frames_per_second != null}
									<div style="font-size: 2rem; font-weight: bold;">
										{totals.frames_per_second.toFixed(2)} fps
									</div>
									<Tag
										type={totals.frames_per_second > 10
											? 'green'
											: totals.frames_per_second > 3
												? 'blue'
												: 'red'}
										size="sm"
									>
										{totals.frames_per_second > 10
											? 'High'
											: totals.frames_per_second > 3
												? 'Moderate'
												: 'Low'}
									</Tag>
								{:else}
									<div style="font-size: 2rem; font-weight: bold; color: gray;">N/A</div>
								{/if}
							</Tile>
						</Column>
						<Column>
							<Tile>
								<h3>Total Packet Loss</h3>
								{#if totals}
									<div style="font-size: 2rem; font-weight: bold; color: #da1e28;">
										{totals.total_packets_lost}
									</div>
									<Tag
										type={totals.packet_loss_percentage < 1
											? 'green'
											: totals.packet_loss_percentage < 5
												? 'blue'
												: totals.packet_loss_percentage < 15
													? 'teal'
													: 'red'}
										size="sm"
									>
										{totals.packet_loss_percentage.toFixed(2)}%
									</Tag>
									<ProgressBar
										value={Math.min(100, Math.max(0, totals.packet_loss_percentage))}
										size="sm"
										style="margin-top: 0.5rem;"
									/>
								{:else}
									<div style="font-size: 2rem; font-weight: bold; color: gray;">N/A</div>
								{/if}
							</Tile>
						</Column>
						<Column>
							<Tile>
								<h3>Connection Status</h3>
								<div
									style="font-size: 2rem; font-weight: bold; color: {connected ? 'green' : 'red'};"
								>
									{connected ? 'Live' : 'Static'}
								</div>
								<Tag type={connected ? 'green' : 'gray'} size="sm">
									{connected ? 'Real-time' : 'Manual Refresh'}
								</Tag>
							</Tile>
						</Column>
					</Row>

					<!-- Metrics Table -->
					<Table>
						<TableHead>
							<TableRow>
								<TableHeader>Timestamp</TableHeader>
								<TableHeader>RSSI</TableHeader>
								<TableHeader>RSSI Status</TableHeader>
								<TableHeader>Packets Lost</TableHeader>
								<TableHeader>Packet Loss Status</TableHeader>
							</TableRow>
						</TableHead>
						<TableBody>
							{#each metrics as metric (metric.timestamp)}
								<TableRow>
									<TableCell>{formatTimestamp(metric.timestamp)}</TableCell>
									<TableCell>
										{metric.rssi !== null ? `${metric.rssi} dBm` : 'N/A'}
									</TableCell>
									<TableCell>
										<Tag type={getRssiStatus(metric.rssi).color} size="sm">
											{getRssiStatus(metric.rssi).status}
										</Tag>
									</TableCell>
									<TableCell>{metric.packets_lost}</TableCell>
									<TableCell>
										<Tag type={getPacketLossStatus(metric.packets_lost).color} size="sm">
											{getPacketLossStatus(metric.packets_lost).status}
										</Tag>
									</TableCell>
								</TableRow>
							{/each}
						</TableBody>
					</Table>
				{/if}
			</Tile>
		</Column>
	</Row>
</Grid>
