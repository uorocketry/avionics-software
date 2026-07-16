<script lang="ts">
	import { settings } from '$lib/common/settings';
	import BooleanSetting from './components/BooleanSetting.svelte';
	import StringSetting from './components/StringSetting.svelte';
	import NumberSetting from './components/NumberSetting.svelte';
	import EnumSetting from './components/EnumSetting.svelte';
	import UnsupportedSetting from './components/UnsupportedSetting.svelte';

	const componentMap: Record<string, any> = {
		boolean: BooleanSetting,
		string: StringSetting,
		number: NumberSetting,
		enum: EnumSetting
	};
	const format = (value: string) =>
		value.replace(/([A-Z])/g, ' $1').replace(/^./, (letter) => letter.toUpperCase());
	const componentFor = (description: string) => componentMap[description] ?? UnsupportedSetting;

	function resetUi() {
		localStorage.removeItem('rgs.ui.shell.v1');
		localStorage.removeItem('ui.theme');
		window.location.reload();
	}
</script>

<svelte:head><title>Settings | RGS</title></svelte:head>

<div class="rgs-page">
	<header class="rgs-page-header">
		<div>
			<h1 class="rgs-page-title">Settings</h1>
			<div class="rgs-muted subtitle">LOCAL OPERATOR PREFERENCES</div>
		</div>
		<button class="reset" type="button" onclick={resetUi}>RESET UI PREFERENCES</button>
	</header>

	{#each settings as group}
		<section class="rgs-panel settings-group">
			<div class="rgs-panel-header"><h2>{format(group.name)}</h2></div>
			<div class="setting-rows">
				{#each group.settings as setting}
					{@const Component = componentFor(setting.valueDescription)}
					<div class="setting-row">
						<div>
							<strong>{format(setting.name)}</strong>
							<p>{setting.description ?? 'No description available'}</p>
						</div>
						<div class="setting-control"><Component {setting} compact /></div>
					</div>
				{/each}
			</div>
		</section>
	{/each}
</div>

<style>
	.subtitle {
		margin-top: 2px;
		font-size: 0.68rem;
		letter-spacing: 0.08em;
	}
	.reset {
		min-height: 32px;
		padding: 0 12px;
		border: 1px solid var(--rgs-border);
		background: var(--rgs-surface);
		color: var(--rgs-text);
		font-size: 0.72rem;
	}
	.reset:hover {
		background: var(--rgs-surface-alt);
	}
	.setting-row {
		display: grid;
		grid-template-columns: minmax(220px, 1fr) minmax(260px, 420px);
		align-items: center;
		gap: 16px;
		min-height: 56px;
		padding: 7px 12px;
		border-bottom: 1px solid var(--rgs-border-subtle);
	}
	.setting-row:last-child {
		border-bottom: 0;
	}
	.setting-row strong {
		font-size: 0.82rem;
	}
	.setting-row p {
		margin: 2px 0 0;
		color: var(--rgs-text-muted);
		font-size: 0.72rem;
	}
	@media (max-width: 767px) {
		.setting-row {
			grid-template-columns: 1fr;
			gap: 8px;
			padding: 10px 8px;
		}
		.setting-control {
			width: 100%;
		}
	}
</style>
