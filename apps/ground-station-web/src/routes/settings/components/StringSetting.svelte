<script lang="ts">
	import type { Writable } from 'svelte/store';
	import { TextInput } from 'carbon-components-svelte';

	type $$Props = {
		setting: {
			valueDescription: 'string';
			value: Writable<string>;
			description?: string;
			placeholder?: string;
		};
	};
	let { setting }: $$Props = $props();

	let settingVal = $derived(setting.value);
</script>

<TextInput
	labelText={setting.description || 'Text setting'}
	placeholder={setting.placeholder ?? 'Enter text...'}
	bind:value={$settingVal}
	size="sm"
/>
