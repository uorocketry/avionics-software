import { browser } from '$app/environment';
import { findSetting } from '$lib/common/settings';
import { derived, writable, type Readable, type Writable } from 'svelte/store';

export type CarbonTheme = 'white' | 'g10' | 'g80' | 'g90' | 'g100';

const allowedThemes: CarbonTheme[] = ['white', 'g10', 'g80', 'g90', 'g100'];

export function resolveCarbonTheme(value: string): CarbonTheme {
	return (allowedThemes as readonly string[]).includes(value) ? (value as CarbonTheme) : 'g100';
}

// Source store from settings; fallback to writable if missing
const themeSetting = findSetting('ui.theme');
const rawThemeStore: Writable<string> =
	themeSetting && themeSetting.valueDescription === 'enum'
		? (themeSetting.value as Writable<string>)
		: writable<string>('white');

export const carbonTheme: Readable<CarbonTheme> = derived(rawThemeStore, (v, set) => {
	set(resolveCarbonTheme(v));
});

// Apply to document root for Carbon styling
if (browser) {
	carbonTheme.subscribe((t) => {
		const html = document.documentElement;
		html.setAttribute('theme', t);
		html.setAttribute('data-carbon-theme', t);
	});
}

// Optional setter for programmatic changes
export function setAppTheme(value: string) {
	rawThemeStore.set(value);
}
