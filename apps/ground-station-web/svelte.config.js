import adapter from 'svelte-adapter-bun';

/** @type {import('@sveltejs/kit').Config} */
const config = {
	compilerOptions: {
	},
	kit: {
		adapter: adapter(),
		csrf: {
			trustedOrigins: ['http://localhost:5173', 'http://localhost:3000', 'wails://gui']
		}
	}
};

export default config;
