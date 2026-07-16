<script lang="ts">
	import { cubicInOut } from 'svelte/easing';
	import { tweened } from 'svelte/motion';
	import { RAD2DEG } from 'three/src/math/MathUtils.js';

	let w = 100;
	let h = 100;

  export let heading = 0;
  import { carbonTheme } from '$lib/common/theme';
  import { get } from 'svelte/store';

	let headingTweened = tweened(0, {
		duration: 250,
		easing: cubicInOut
	});

	$: headingTweened.set(heading);

	const breaksPoints = {
		0: 'N',
		45: 'NE',
		90: 'E',
		135: 'SE',
		180: 'S',
		225: 'SW',
		270: 'W',
		315: 'NW'
	} as const;

	let angle = 0;
	$: radius = Math.abs(Math.min(w, h - 80) / 2);

	type Lines = {
		x: number;
		y: number;
		x2: number;
		y2: number;
	};

	type Texts = {
		x: number;
		y: number;
		text: string;
		r: number;
		color?: string;
	};

	let lines: Lines[] = [];
	let texts: Texts[] = [];

	$: {
		lines = [];
		texts = [];
		angle = 0;
		while (angle < 360) {
			const center_x = w / 2;
			const center_y = h / 2;

			const x = center_x + Math.cos((angle * Math.PI) / 180) * radius;
			const y = center_y + Math.sin((angle * Math.PI) / 180) * radius;

			let lineDist = 0.05;
			if (angle % 10 === 0) {
				lineDist = 0.1;
			}
			if (angle % 30 === 0) {
				lineDist = 0.1;

				// Add text if not a label
				if (breaksPoints[angle as keyof typeof breaksPoints] === undefined) {
					const text = angle;
					const textDist = 0.15;
					const x = center_x + Math.cos((angle * Math.PI) / 180) * radius * (1 - textDist);
					const y = center_y + Math.sin((angle * Math.PI) / 180) * radius * (1 - textDist);
					const rad = angle + 90;
					texts.push({ x, y, text: text.toString(), r: rad });
				}
			}
			if (angle % 45 === 0) {
				lineDist = 0.2;
			}

			const text = breaksPoints[angle as keyof typeof breaksPoints];
			if (text) {
				const x = center_x + Math.cos((angle * Math.PI) / 180) * radius * 0.75;
				const y = center_y + Math.sin((angle * Math.PI) / 180) * radius * 0.75;
				const rad = angle + 90;
				texts.push({ x, y, text, r: rad, color: 'red' });
			}

			const x2 = center_x + Math.cos((angle * Math.PI) / 180) * radius * (1 - lineDist);
			const y2 = center_y + Math.sin((angle * Math.PI) / 180) * radius * (1 - lineDist);

			lines.push({ x, y, x2, y2 });
			angle += 5;
		}
	}
</script>

<div
	bind:clientHeight={h}
	bind:clientWidth={w}
	class="w-full h-full flex-col text-center select-none"
>
  <svg
		class="w-full h-full"
		transform="rotate({-90 - $headingTweened * RAD2DEG})"
		width={w}
		height={h}
		xmlns="http://www.w3.org/2000/svg"
	>
		<circle cx={w / 2} cy={h / 2} r={radius} stroke="var(--cds-border-subtle)" stroke-width="2" fill="none" />

		{#each lines as line}
			<line x1={line.x} y1={line.y} x2={line.x2} y2={line.y2} stroke="var(--cds-text-secondary)" />
		{/each}

		{#each texts as text}
			<text
				x={text.x}
				y={text.y}
				fill={text.color ?? 'var(--cds-text-secondary)'}
				transform={`rotate(${text.r} ${text.x} ${text.y})`}
				text-anchor="middle"
				alignment-baseline="middle"
				font-size="24"
			>
				{text.text}
			</text>
		{/each}
	</svg>

	<!-- SVG for the compass needle -->
	<svg
		class="absolute top-0 left-0 w-full h-full"
		width={w}
		height={h}
		xmlns="http://www.w3.org/2000/svg"
	>
		<line
			x1={w / 2}
			y1={h / 2 - radius * 0.9}
			x2={w / 2}
			y2={h / 2 - radius}
			stroke="var(--cds-support-error)"
			stroke-width="2"
		/>
		<!-- Text -->
		<rect
			x={w / 2 - 50}
			y={h / 2 - radius - 20}
			width={100}
			height={40}
			fill="var(--cds-background)"
			stroke-width="2"
			rx="5"
			ry="5"
		/>

		<text
			x={w / 2}
			y={h / 2 - radius}
			fill="var(--cds-text-primary)"
			text-anchor="middle"
			alignment-baseline="middle"
			font-size="24"
		>
			{($headingTweened * RAD2DEG).toFixed(2)}°
		</text>
	</svg>
</div>

<style>
	svg {
		transition: transform 0.25s;
	}
</style>
