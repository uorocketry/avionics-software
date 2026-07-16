import { describe, expect, it } from 'vitest';
import { getFreshnessStatus } from './sse';

describe('getFreshnessStatus', () => {
	const now = 100_000;

	it('reports connecting before the first update', () => {
		expect(getFreshnessStatus(null, now, 5_000, 60_000)).toBe('connecting');
	});

	it('uses the configured live, stale, and offline boundaries', () => {
		expect(getFreshnessStatus(now - 5_000, now, 5_000, 60_000)).toBe('live');
		expect(getFreshnessStatus(now - 5_001, now, 5_000, 60_000)).toBe('stale');
		expect(getFreshnessStatus(now - 60_001, now, 5_000, 60_000)).toBe('offline');
	});
});
