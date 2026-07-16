/**
 * Least Recently Used (LRU) cache implementation.
 */
export class LRUCache<K, V> {
	public cache = new Map<K, V>();
	public capacity: number;

	constructor(capacity: number) {
		this.capacity = capacity;
	}

	get(key: K): V | undefined {
		const value = this.cache.get(key);
		if (value) {
			this.cache.delete(key);
			this.cache.set(key, value); // Move to the end
		}
		return value;
	}

	put(key: K, value: V): void {
		this.cache.delete(key);
		this.cache.set(key, value);
		if (this.cache.size > this.capacity) {
			const oldest = this.cache.keys().next();
			if (!oldest.done) this.cache.delete(oldest.value);
		}
	}

	clear(): void {
		this.cache.clear();
	}
}
