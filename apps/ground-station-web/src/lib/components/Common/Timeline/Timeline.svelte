<script lang="ts">
	import { onMount } from 'svelte';

	let zoomLevels: { [key: number]: number } = {
		10: 1000, // 1s
		9: 2000, // 2s
		8: 5000, // 5s
		7: 10_000, // 10s
		6: 15_000, // 15s
		5: 30_000, // 30s
		4: 60_000, // 1m
		3: 60_000 * 2.5, // 2.5m
		2: 60_000 * 5, // 5m
		1: 60_000 * 10, // 10m
		0: 60_000 * 30 // 30m
	};

	let zoomSteps = 10;
	let zoomLevel = 0;
	$: zoomAmount = zoomLevel / zoomSteps;
	let maxZoomPPSPercent = 0.075;
	$: pixelPerSecond = Math.ceil((zoomAmount + 1) * clientWidth * maxZoomPPSPercent);
	$: tickEveryNSecond = zoomLevels[zoomLevel] / 1000;
	$: pixelsPerMillisecond = pixelPerSecond / tickEveryNSecond / 1000;
	$: millisecondsPerPixel = (tickEveryNSecond / pixelPerSecond) * 1000;

	// hh:mm:ss 24h format
	const needleFormatter = new Intl.DateTimeFormat('en', {
		hour: 'numeric',
		minute: 'numeric',
		second: 'numeric',
		hour12: false
	});

	const tickFormatterShort = new Intl.DateTimeFormat('en', {
		minute: 'numeric',
		second: 'numeric',
		hour12: false
	});

	const tickFormatterHour = new Intl.DateTimeFormat('en', {
		hour: 'numeric',
		minute: 'numeric',
		hour12: false
	});

	function handleScroll(event: WheelEvent) {
		zoomLevel = Math.max(0, Math.min(zoomLevel - Math.sign(event.deltaY), zoomSteps));
		event.preventDefault();
	}

	// let needleTimeStart = Date.now();
	let needleTime = 0;

	let container: HTMLElement | null = null;
	let clientWidth = 0;

	let ticks: number[] = [];

	function render() {
		calculateTicks();

		requestAnimationFrame(render);
	}

	function tickPositionToTime(tickPosition: number) {
		let delta = tickPosition - clientWidth / 2;
		return Math.ceil(needleTime + delta * millisecondsPerPixel);
	}

	let memoryLeftStart = -1;
	function calculateTicks() {
		// needleTime = Date.now() - needleTimeStart;
		needleTime = Date.now();
		// Needle Pos is alywas in the middle of the container so we calculate the available space on the left and right
		let availableSpace = clientWidth / 2;
		let leftPixelDiff = needleTime * pixelsPerMillisecond;
		let leftPixelStart = availableSpace - leftPixelDiff;

		const nonNegativeIncrement = (start: number, increment: number) => {
			const remainder = start % increment;
			return remainder === 0 ? 0 : remainder;
		};

		leftPixelStart = nonNegativeIncrement(leftPixelStart, pixelPerSecond);

		ticks = [];
		let maxTicks = 500;

		for (
			let curPixel = leftPixelStart;
			curPixel < clientWidth + pixelPerSecond; // rightPixelStart
			curPixel += pixelPerSecond
		) {
			if (maxTicks < 0) {
				console.error('Max ticks reached');
				break;
			}
			ticks.push(curPixel);
		}

		if (memoryLeftStart !== -1) {
			memoryLeftStart = leftPixelStart;
		} else {
			memoryLeftStart = leftPixelStart;
		}
	}

	onMount(() => {
		render();
	});
</script>

<div
	class="timeline w-full h-full overflow-clip"
	bind:this={container}
	bind:clientWidth
	on:wheel={handleScroll}
>
	{#each ticks as tick}
		<div class="timeline-tick" style="left: {tick}px;">
			<div class="timeline-tick-text">
				{#if zoomLevel === 0}
					{tickFormatterHour.format(tickPositionToTime(tick))}
				{:else}
					{tickFormatterShort.format(tickPositionToTime(tick))}
				{/if}
			</div>
		</div>
	{/each}

	<div class="timeline-needle" style="left: {clientWidth / 2}px;">
		<div class="timeline-needle-text">
			{needleFormatter.format(needleTime)}
		</div>
	</div>
</div>

<style>
	.timeline {
		user-select: none;
	}

	.timeline-needle {
		width: 2px;
		height: 1.5rem;
		background-color: black;
		position: absolute;
		bottom: 0px;
		transform: translateX(-50%);
	}

	.timeline-needle-text {
		position: absolute;
		top: -1.25rem;
		transform: translateX(-50%);
	}

	.timeline-tick-text {
		position: absolute;
		top: -1.25rem;
		font-size: 0.75rem;
		transform: translateX(-50%);
	}
	.timeline-tick {
		width: 1px;
		height: 0.5rem;
		bottom: 0;
		position: absolute;
		transform: translateX(-50%);
		background-color: black;
	}
</style>
