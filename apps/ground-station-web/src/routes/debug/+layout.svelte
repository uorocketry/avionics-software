<script lang="ts">
	import { Content } from 'carbon-components-svelte';

	let { children } = $props();
</script>

<Content>
	{@render children?.()}
</Content>
