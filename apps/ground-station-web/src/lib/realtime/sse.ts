import { browser } from '$app/environment';
import { readable, type Readable } from 'svelte/store';
import type { ConnectionStatus } from './types';

export type SseResource<T> = {
	data: T | null;
	status: ConnectionStatus;
	error: string | null;
	lastUpdated: number | null;
};

type Options = {
	event?: string;
	staleAfterMs?: number;
	offlineAfterMs?: number;
};

export function getFreshnessStatus(
	lastUpdated: number | null,
	now: number,
	staleAfterMs: number,
	offlineAfterMs: number
): ConnectionStatus {
	if (lastUpdated === null) return 'connecting';
	const age = now - lastUpdated;
	if (age > offlineAfterMs) return 'offline';
	if (age > staleAfterMs) return 'stale';
	return 'live';
}

export function createSseResource<T>(url: string, options: Options = {}): Readable<SseResource<T>> {
	const eventName = options.event ?? 'message';
	const staleAfterMs = options.staleAfterMs ?? 5_000;
	const offlineAfterMs = options.offlineAfterMs ?? 60_000;

	return readable<SseResource<T>>(
		{ data: null, status: 'connecting', error: null, lastUpdated: null },
		(set) => {
			if (!browser) return;

			let source: EventSource | null = null;
			let data: T | null = null;
			let lastUpdated: number | null = null;
			let stopped = false;
			let attempts = 0;
			let reconnectTimer: ReturnType<typeof setTimeout> | null = null;

			const publish = (status: ConnectionStatus, error: string | null = null) =>
				set({ data, status, error, lastUpdated });

			const connect = () => {
				if (stopped) return;
				publish(data ? 'stale' : 'connecting');
				source = new EventSource(url);
				source.onopen = () => {
					attempts = 0;
					if (data) publish('live');
				};
				const handleMessage = (event: MessageEvent) => {
					try {
						data = JSON.parse(event.data) as T;
						lastUpdated = Date.now();
						publish('live');
					} catch {
						publish('error', 'Received malformed realtime data');
					}
				};
				if (eventName === 'message') source.onmessage = handleMessage;
				else source.addEventListener(eventName, handleMessage as EventListener);
				source.onerror = () => {
					source?.close();
					source = null;
					attempts += 1;
					publish(data ? 'stale' : 'offline', 'Realtime connection interrupted');
					const delay = Math.min(30_000, 500 * 2 ** Math.max(0, attempts - 1));
					reconnectTimer = setTimeout(connect, delay);
				};
			};

			const freshnessTimer = setInterval(() => {
				if (!lastUpdated || !data) return;
				const freshness = getFreshnessStatus(lastUpdated, Date.now(), staleAfterMs, offlineAfterMs);
				if (freshness === 'offline') publish('offline', 'No recent data');
				else if (freshness === 'stale') publish('stale', 'Data is stale');
			}, 1_000);

			connect();
			return () => {
				stopped = true;
				source?.close();
				if (reconnectTimer) clearTimeout(reconnectTimer);
				clearInterval(freshnessTimer);
			};
		}
	);
}
