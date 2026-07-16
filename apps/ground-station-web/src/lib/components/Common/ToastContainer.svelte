<script lang="ts">
	import { toastStore } from '$lib/stores/toastStore';
	import { ToastNotification } from 'carbon-components-svelte';

	const toasts = toastStore;
</script>

{#if $toasts.length > 0}
	<div class="toast-container">
		{#each $toasts as toast (toast.id)}
			<ToastNotification
				title={toast.message}
				kind={toast.type}
				timeout={toast.duration ?? 0}
				lowContrast
				on:close={() => toastStore.remove(toast.id)}
			/>
		{/each}
	</div>
{/if}

<style>
	.toast-container {
		position: fixed;
		top: 1rem;
		right: 1rem;
		display: flex;
		flex-direction: column;
		gap: 0.5rem;
		z-index: 10000;
	}
</style>
