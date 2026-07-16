<script lang="ts">
	import type { Writable } from 'svelte/store';
	import { Select, SelectItem } from 'carbon-components-svelte';

	type $$Props = {
		setting: {
			name?: string;
			valueDescription: 'enum';
			options: string[];
			value: Writable<string>;
			description?: string;
		};
	};
	let { setting }: $$Props = $props();

	let settingVal = $derived(setting.value);
	const optionText = (option: string) =>
		setting.name === 'theme' ? (option === 'white' ? 'Sunlight' : 'Dark') : option;
</script>

<Select labelText={setting.description || 'Select option'} bind:selected={$settingVal}>
	{#each setting.options as option}
		<SelectItem value={option} text={optionText(option)} />
	{/each}
</Select>
