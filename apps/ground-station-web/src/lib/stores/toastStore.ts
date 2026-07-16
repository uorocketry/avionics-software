import { writable } from 'svelte/store';

export type ToastType = 'info' | 'success' | 'warning' | 'error';

export interface Toast {
	id: number;
	message: string;
	type: ToastType;
	duration?: number; // Optional duration in ms; used by Carbon's ToastNotification
}

const { subscribe, update } = writable<Toast[]>([]);

let nextId = 0;

function addToast(message: string, type: ToastType = 'info', duration: number = 3000) {
	const id = nextId++;
	update((toasts) => [...toasts, { id, message, type, duration }]);
}

function removeToast(id: number) {
	update((toasts) => toasts.filter((toast) => toast.id !== id));
}

export const toastStore = {
	subscribe,
	add: addToast,
	remove: removeToast,
	info: (message: string, duration?: number) => addToast(message, 'info', duration),
	success: (message: string, duration?: number) => addToast(message, 'success', duration),
	warning: (message: string, duration?: number) => addToast(message, 'warning', duration),
	error: (message: string, duration?: number) => addToast(message, 'error', duration)
};