<script lang="ts">
	import type { Writable } from 'svelte/store';
	import { Toggle } from 'carbon-components-svelte';

	type $$Props = {
		setting: {
			valueDescription: 'boolean';
			value: Writable<boolean>;
			description?: string;
		};
	};
	let { setting }: $$Props = $props();

	let settingVal = $derived(setting.value);
</script>

<Toggle labelText={setting.description || 'Toggle setting'} bind:toggled={$settingVal} size="sm" />
