import http from 'node:http';
import type { IncomingMessage } from 'node:http';

export type ServiceStatus = 'running' | 'exited' | 'restarting' | 'unknown';
export type ServiceSummary = { name: string; status: ServiceStatus; composeService: string | null };

const DOCKER_SOCKET = '/var/run/docker.sock';
const COMPOSE_PROJECT = process.env.COMPOSE_PROJECT_NAME ?? 'ground-station';

export function dockerRequest(
	path: string,
	method = 'GET',
	body?: string
): Promise<{ status: number; text: () => Promise<string>; json: () => Promise<any>; ok: boolean }> {
	return new Promise((resolve, reject) => {
		const req = http.request(
			{
				socketPath: DOCKER_SOCKET,
				path,
				method,
				headers: body
					? { 'Content-Type': 'application/json', 'Content-Length': Buffer.byteLength(body) }
					: undefined
			},
			(res: IncomingMessage) => {
				const chunks: Buffer[] = [];
				res.on('data', (chunk) => chunks.push(Buffer.isBuffer(chunk) ? chunk : Buffer.from(chunk)));
				res.on('end', () => {
					const buffer = Buffer.concat(chunks);
					const status = res.statusCode ?? 500;
					resolve({
						status,
						ok: status >= 200 && status < 300,
						text: async () => buffer.toString('utf8'),
						json: async () => JSON.parse(buffer.toString('utf8') || 'null')
					});
				});
			}
		);
		req.on('error', reject);
		if (body) req.write(body);
		req.end();
	});
}

export async function listServices(): Promise<ServiceSummary[]> {
	const response = await dockerRequest('/containers/json?all=true');
	if (!response.ok) throw new Error(`Docker API error: ${response.status}`);
	const containers: any[] = await response.json();
	return containers
		.filter((container) => container.Labels?.['com.docker.compose.project'] === COMPOSE_PROJECT)
		.map((container) => ({
			name: container.Names?.[0]?.replace(/^\//, '') ?? container.Id.substring(0, 12),
			status: (container.State as ServiceStatus) ?? 'unknown',
			composeService: container.Labels?.['com.docker.compose.service'] ?? null
		}));
}
