import { writable, type Writable } from "svelte/store";

// Custom replacement for Skeleton's localStorageStore
export function persistentStore<T>(key: string, initialValue: T): Writable<T> {
	// Check for browser environment
	if (typeof window === 'undefined' || !window.localStorage) {
		console.warn(`localStorage not available for key "${key}", using non-persistent store.`);
		return writable(initialValue);
	}

	let storedValue: T | undefined = undefined;
	try {
		const storedItem = window.localStorage.getItem(key);
		if (storedItem !== null) {
			storedValue = JSON.parse(storedItem) as T;
		}
	} catch (e) {
		console.error(`Error reading localStorage key “${key}”:`, e);
		storedValue = undefined; // Use initial value if parsing fails
	}

	const store = writable<T>(storedValue !== undefined ? storedValue : initialValue);

	store.subscribe((value) => {
		try {
			window.localStorage.setItem(key, JSON.stringify(value));
		} catch (e) {
			console.error(`Error setting localStorage key “${key}”:`, e);
		}
	});

	// Sync store with changes from other tabs
	const handleStorage = (event: StorageEvent) => {
		if (event.key === key && event.newValue !== null) {
			try {
				store.set(JSON.parse(event.newValue) as T);
			} catch (e) {
				console.error(`Error parsing storage event for key “${key}”:`, e);
			}
		} else if (event.key === key && event.newValue === null) {
			// Item was removed in another tab, reset to initial
			store.set(initialValue);
		}
	};

	window.addEventListener('storage', handleStorage);

	// Return a store that cleans up the event listener on unsubscribe
	// (This requires modifying the return slightly if true cleanup is needed,
	// but for settings, they usually live for the app lifetime)
	// For simplicity now, we don't return the enhanced store with cleanup.

	return store;
}