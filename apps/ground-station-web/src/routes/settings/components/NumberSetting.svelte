<script lang="ts">
	import type { Writable } from 'svelte/store';
	import { NumberInput } from 'carbon-components-svelte';

	type $$Props = {
		setting: {
			valueDescription: 'number';
			value: Writable<number>;
			description?: string;
			min?: number;
			max?: number;
			step?: number;
		};
	};
	let { setting }: $$Props = $props();

	let settingVal = $derived(setting.value);
</script>

<NumberInput
	label={setting.description || 'Number setting'}
	min={setting.min}
	max={setting.max}
	step={setting.step ?? 1}
	bind:value={$settingVal}
/>
