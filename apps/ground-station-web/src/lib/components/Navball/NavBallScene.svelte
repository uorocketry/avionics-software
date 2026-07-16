<script lang="ts">
	import { T } from '@threlte/core';
	import { Billboard, MeshLineGeometry, MeshLineMaterial, useTexture } from '@threlte/extras';
	import { Quaternion, Vector3 } from 'three';
	import navballfrag from './NavBallShader/fragment.glsl?raw';
	import navballvert from './NavBallShader/vertex.glsl?raw';
	import { useTask } from '@threlte/core';

	const map = useTexture('textures/navball.png');
	map.then((tex) => {
		tex.anisotropy = 32;
		tex.colorSpace = 'srgb-linear';
	});

	let { targetRotation = new Quaternion() } = $props<{ targetRotation?: Quaternion }>();

	let displayRotation = $state(new Quaternion());
	$effect(() => {
		displayRotation = targetRotation.clone();
	});

	useTask((delta) => {
		displayRotation = displayRotation.slerp(targetRotation, 5 * delta);
		displayRotation = displayRotation.clone();
	});
</script>

<T.OrthographicCamera makeDefault={true} position={[0, 0, 20]} zoom={200} />

<Billboard position={[0, 0, 2]}>
	<T.Mesh>
		<T.SphereGeometry args={[0.02]} />
		<T.MeshBasicMaterial color="orange" />
	</T.Mesh>

	<T.Mesh>
		<MeshLineGeometry
			points={[
				new Vector3(0, 0.5),
				new Vector3(0.25, 0.5),
				new Vector3(0.5, 0.25),
				new Vector3(0.75, 0.5),
				new Vector3(1, 0.5)
			].map((v) => {
				v.x *= 0.5;
				v.y *= 0.5;
				v.x -= 0.25;
				v.y -= 0.25;
				return v;
			})}
		/>
		<MeshLineMaterial width={0.015} color="orange" />
	</T.Mesh>
</Billboard>

{#await map then tex}
	<T.Mesh quaternion={[displayRotation.x, displayRotation.y, displayRotation.z, displayRotation.w]}>
		<T.SphereGeometry />
		<T.ShaderMaterial
			fragmentShader={navballfrag}
			vertexShader={navballvert}
			uniforms={{
				uTexture: { value: tex }
			}}
		/>
	</T.Mesh>
{/await}
