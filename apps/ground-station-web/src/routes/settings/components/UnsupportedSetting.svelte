<script lang="ts">
	type $$Props = {
		setting: {
			valueDescription: string;
			description?: string;
		};
	};
	let { setting }: $$Props = $props();
</script>

<div class="flex items-center gap-2 p-2 border border-dashed border-warning rounded-md">
	<span class="badge badge-warning">{setting.valueDescription.toUpperCase()}</span>
	<span class="label-text text-warning">{setting.description} (Not Implemented)</span>
</div>
