import { error, json } from '@sveltejs/kit';
import { dockerRequest, listServices } from '$lib/server/services';

async function containerAction(service: string, action: 'start' | 'stop' | 'restart') {
	let p = `/containers/${encodeURIComponent(service)}/${action}`;
	if (action === 'restart') p += '?t=5';
	const res = await dockerRequest(p, 'POST');
	if (!(res.status === 204 || res.status === 304)) {
		throw new Error(`Docker ${action} failed with status ${res.status}`);
	}
}

export async function GET() {
	try {
		const services = await listServices();
		return json(services);
	} catch (e: any) {
		throw error(500, e.message ?? 'Failed to list services');
	}
}

export async function POST({ url }) {
	const action = (url.searchParams.get('action') as 'start' | 'stop' | 'restart') ?? 'start';
	const service = url.searchParams.get('service');
	if (!service) throw error(400, 'service is required');
	try {
		await containerAction(service, action);
		return new Response(null, { status: 204 });
	} catch (e: any) {
		throw error(500, e.message ?? 'Operation failed');
	}
}
