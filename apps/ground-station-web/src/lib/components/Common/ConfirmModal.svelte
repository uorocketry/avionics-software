<script lang="ts">
	let {
		open = $bindable(false),
		title,
		message,
		confirmLabel = 'Confirm',
		danger = false,
		onconfirm,
		oncancel
	} = $props<{
		open: boolean;
		title: string;
		message: string;
		confirmLabel?: string;
		danger?: boolean;
		onconfirm: () => void;
		oncancel?: () => void;
	}>();

	let dialog: HTMLDialogElement;

	$effect(() => {
		if (!dialog) return;
		if (open && !dialog.open) dialog.showModal();
		if (!open && dialog.open) dialog.close();
	});

	function cancel() {
		open = false;
		oncancel?.();
	}

	function confirm() {
		open = false;
		onconfirm();
	}
</script>

<dialog
	bind:this={dialog}
	class="confirm-dialog"
	oncancel={(event) => {
		event.preventDefault();
		cancel();
	}}
>
	<div class="dialog-title">
		<i
			class={danger ? 'fa-solid fa-triangle-exclamation' : 'fa-solid fa-circle-question'}
			aria-hidden="true"
		></i>{title}
	</div>
	<p>{message}</p>
	<div class="dialog-actions">
		<button type="button" onclick={cancel}>Cancel</button>
		<button class:danger type="button" onclick={confirm}>{confirmLabel}</button>
	</div>
</dialog>

<style>
	.confirm-dialog {
		width: min(440px, calc(100vw - 24px));
		padding: 0;
		border: 1px solid var(--rgs-border);
		background: var(--rgs-surface);
		color: var(--rgs-text);
		box-shadow: 0 16px 48px rgb(0 0 0 / 35%);
	}
	.confirm-dialog::backdrop {
		background: rgb(0 0 0 / 55%);
	}
	.dialog-title {
		display: flex;
		min-height: 40px;
		align-items: center;
		gap: 8px;
		padding: 0 12px;
		border-bottom: 1px solid var(--rgs-border);
		background: var(--rgs-surface-alt);
		font-weight: 600;
		text-transform: uppercase;
	}
	p {
		margin: 0;
		padding: 16px 12px;
		line-height: 1.5;
	}
	.dialog-actions {
		display: flex;
		justify-content: flex-end;
		border-top: 1px solid var(--rgs-border-subtle);
	}
	button {
		min-width: 96px;
		min-height: 36px;
		padding: 0 12px;
		border: 0;
		border-left: 1px solid var(--rgs-border-subtle);
		background: var(--rgs-surface);
		color: var(--rgs-text);
	}
	button:hover {
		background: var(--rgs-surface-alt);
	}
	button.danger {
		background: var(--rgs-danger);
		color: white;
	}
</style>
