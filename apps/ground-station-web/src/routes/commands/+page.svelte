<script lang="ts">
	import { enhance } from '$app/forms';
	import { invalidateAll } from '$app/navigation';
	import { toastStore } from '$lib/stores/toastStore';
	import ConfirmModal from '$lib/components/Common/ConfirmModal.svelte';

	// Types for loaded data and form state
	type Command = {
		id: number;
		command_type: string;
		parameters: string | null;
		status: string;
		created_at: number;
		queued_at: number | null;
		sent_at: number | null;
		attempts: number;
		error_message: string | null;
		source_service: string;
	};

	let { data, form } = $props();

	// Reactive state for the form inputs
	let selectedCommand = $state('Ping');
	let param_board = $state('PressureBoard');
	let param_rate = $state('Fast');
	let dispatching = $state(false);
	let deleting = $state(false); // State for delete operations
	let confirmOpen = $state(false);
	let pendingDeleteForm = $state<HTMLFormElement | null>(null);
	let confirmMessage = $state('');
	let allowDelete = false;

	// Command definitions for the form
	const availableCommands = {
		Ping: { params: [] as string[] },
		DeployDrogue: { params: [] as string[] },
		DeployMain: { params: [] as string[] },
		PowerDown: { params: ['board'] },
		PowerUpCamera: { params: [] as string[] },
		PowerDownCamera: { params: [] as string[] },
		RadioRateChange: { params: ['rate'] }
	};
	type CommandName = keyof typeof availableCommands;

	const commandTypes = Object.keys(availableCommands) as CommandName[];
	const boardOptions = ['PressureBoard', 'StrainBoard', 'TemperatureBoard']; // Add others if needed
	const rateOptions = ['Fast', 'Slow'];
	import {
		Grid,
		Row,
		Column,
		Tile,
		Dropdown,
		Button,
		InlineNotification,
		InlineLoading,
		Table,
		TableHead,
		TableRow,
		TableHeader,
		TableBody,
		TableCell,
		Tag
	} from 'carbon-components-svelte';

	// Function to format timestamps
	function formatTimestamp(ts: number | null): string {
		if (ts === null || ts === undefined) return '-';
		try {
			return new Date(ts * 1000).toLocaleString();
		} catch (e) {
			return 'Invalid Date';
		}
	}

	// Derived state to check if current command needs params
	let needsParams = $derived(availableCommands[selectedCommand as CommandName]?.params.length > 0);
	let currentParams = $derived(availableCommands[selectedCommand as CommandName]?.params);

	// Effect to show toast messages based on form action result
	$effect(() => {
		if (form?.success) {
			toastStore.success(form.message || 'Operation successful!');
			invalidateAll(); // Refresh command list on success
		}
		if (form?.error) {
			toastStore.error(form.error);
		}
		// Reset loading states regardless of outcome
		dispatching = false;
		deleting = false;
	});

	function requestDelete(formElement: HTMLFormElement, message: string, cancel: () => void) {
		if (allowDelete) {
			allowDelete = false;
			deleting = true;
			return true;
		}
		cancel();
		pendingDeleteForm = formElement;
		confirmMessage = message;
		confirmOpen = true;
		return false;
	}

	function confirmDelete() {
		if (!pendingDeleteForm) return;
		allowDelete = true;
		pendingDeleteForm.requestSubmit();
		pendingDeleteForm = null;
	}
</script>

<Grid padding={true} class="rgs-route">
	<Row>
		<Column>
			<Tile>
				<h2>Dispatch New Command</h2>
				<form
					method="POST"
					action="?/dispatch"
					use:enhance={({ cancel }) => {
						dispatching = true;
						return async ({ update }) => {
							await update({ reset: false });
						};
					}}
				>
					<Dropdown
						items={commandTypes.map((c) => ({ id: c, text: c }))}
						selectedId={selectedCommand}
						label="Command Type"
						on:select={(e) => (selectedCommand = e.detail.selectedItem.id)}
						size="sm"
					/>
					<input type="hidden" name="command_type" value={selectedCommand} />
					{#if needsParams}
						<div>
							<h3>Parameters</h3>
							{#if currentParams.includes('board')}
								<Dropdown
									items={boardOptions.map((b) => ({ id: b, text: b }))}
									selectedId={param_board}
									label="Target Board"
									on:select={(e) => (param_board = e.detail.selectedItem.id)}
									size="sm"
								/>
								<input type="hidden" name="param_board" value={param_board} />
							{/if}
							{#if currentParams.includes('rate')}
								<Dropdown
									items={rateOptions.map((r) => ({ id: r, text: r }))}
									selectedId={param_rate}
									label="Radio Rate"
									on:select={(e) => (param_rate = e.detail.selectedItem.id)}
									size="sm"
								/>
								<input type="hidden" name="param_rate" value={param_rate} />
							{/if}
						</div>
					{/if}
					<div>
						<Button kind="primary" size="default" type="submit" disabled={dispatching || deleting}
							>{#if dispatching}<InlineLoading description="Dispatching..." />{:else}Dispatch
								Command{/if}</Button
						>
					</div>
				</form>
			</Tile>
		</Column>
	</Row>

	<Row>
		<Column>
			<Tile>
				<div style="display:flex; justify-content: space-between; align-items: center;">
					<h2>Command History (Last 100)</h2>
					<div>
						<Button
							kind="secondary"
							size="default"
							on:click={() => invalidateAll()}
							disabled={deleting || dispatching}>Refresh</Button
						>
						<form
							method="POST"
							action="?/deleteAll"
							use:enhance={({ cancel, formElement }) => {
								if (
									!requestDelete(
										formElement,
										'Delete all command history? This action cannot be undone.',
										cancel
									)
								)
									return;
								return async ({ update }) => {
									await update();
								};
							}}
							style="display:inline-block; margin-left: 0.5rem;"
						>
							<Button
								kind="danger"
								size="default"
								type="submit"
								disabled={deleting || dispatching}
								title="Delete All Commands"
								>{#if deleting}<InlineLoading description="Deleting..." />{/if}Delete All</Button
							>
						</form>
					</div>
				</div>
				{#if data.error}
					<InlineNotification kind="error" title="Error" subtitle={data.error} hideCloseButton />
				{/if}
				<Table>
					<TableHead>
						<TableRow>
							<TableHeader>ID</TableHeader>
							<TableHeader>Type</TableHeader>
							<TableHeader>Params</TableHeader>
							<TableHeader>Status</TableHeader>
							<TableHeader>Attempts</TableHeader>
							<TableHeader>Created</TableHeader>
							<TableHeader>Queued</TableHeader>
							<TableHeader>Sent</TableHeader>
							<TableHeader>Source</TableHeader>
							<TableHeader>Error</TableHeader>
							<TableHeader>Actions</TableHeader>
						</TableRow>
					</TableHead>
					<TableBody>
						{#each data.commands as command (command.id)}
							<TableRow>
								<TableCell>{command.id}</TableCell>
								<TableCell>{command.command_type}</TableCell>
								<TableCell>{command.parameters || '-'}</TableCell>
								<TableCell
									><Tag
										type={command.status === 'Sent'
											? 'green'
											: command.status === 'Failed'
												? 'red'
												: 'cyan'}
										size="sm">{command.status}</Tag
									></TableCell
								>
								<TableCell>{command.attempts}</TableCell>
								<TableCell>{formatTimestamp(command.created_at)}</TableCell>
								<TableCell>{formatTimestamp(command.queued_at)}</TableCell>
								<TableCell>{formatTimestamp(command.sent_at)}</TableCell>
								<TableCell>{command.source_service}</TableCell>
								<TableCell>{command.error_message || '-'}</TableCell>
								<TableCell>
									<form
										method="POST"
										action="?/deleteSingle"
										use:enhance={({ cancel, formElement }) => {
											if (
												!requestDelete(
													formElement,
													`Delete command ${command.id}? This action cannot be undone.`,
													cancel
												)
											)
												return;
											return async ({ update }) => {
												await update();
											};
										}}
										class="inline-block"
									>
										<input type="hidden" name="id" value={command.id} />
										<Button
											kind="danger"
											size="default"
											type="submit"
											disabled={deleting || dispatching}
											title={`Delete Command ${command.id}`}
											>{#if deleting}<InlineLoading
													description="Deleting..."
												/>{:else}Delete{/if}</Button
										>
									</form>
								</TableCell>
							</TableRow>
						{:else}
							<TableRow>
								<TableCell>No commands found.</TableCell>
							</TableRow>
						{/each}
					</TableBody>
				</Table>
			</Tile>
		</Column>
	</Row>
</Grid>

<ConfirmModal
	bind:open={confirmOpen}
	title="Delete command history"
	message={confirmMessage}
	confirmLabel="DELETE"
	danger
	onconfirm={confirmDelete}
	oncancel={() => (pendingDeleteForm = null)}
/>
