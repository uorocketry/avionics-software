import type { Handle } from '@sveltejs/kit';

function getClientIp(event: Parameters<Handle>[0]['event']): string | null {
    // X-Forwarded-For may include a list. Take the first non-empty token (original client).
    const xff = event.request.headers.get('x-forwarded-for');
    if (xff) {
        const parts = xff.split(',').map((s) => s.trim()).filter(Boolean);
        const first = parts[0];
        if (first) return first;
    }
    const realIp = event.request.headers.get('x-real-ip');
    if (realIp) return realIp;
    // Fall back to platform specific data
    const remote = event.getClientAddress?.() || event.locals.remoteAddress;
    if (typeof remote === 'string' && remote.length > 0) return remote;
    return null;
}

type Cidr = { base: bigint; maskBits: number };

function parseIpToBigInt(ip: string): bigint | null {
    // Strip IPv6 zone or port if present
    const cleaned = ip.replace(/^\[|\]$/g, '').replace(/%.*$/, '');
    if (cleaned.includes(':')) {
        // IPv6
        const parts = cleaned.split('::');
        let hextets: string[] = [];
        if (parts.length === 1) {
            hextets = cleaned.split(':');
        } else if (parts.length === 2) {
            const left = parts[0] ? parts[0].split(':') : [];
            const right = parts[1] ? parts[1].split(':') : [];
            const fill = new Array(8 - (left.length + right.length)).fill('0');
            hextets = [...left, ...fill, ...right];
        } else {
            return null;
        }
        if (hextets.length !== 8) return null;
        let value = 0n;
        for (const h of hextets) {
            const n = BigInt(parseInt(h || '0', 16));
            value = (value << 16n) + n;
        }
        return value;
    } else {
        // IPv4 possibly with port
        const withoutPort = cleaned.split(':')[0];
        const segs = withoutPort.split('.');
        if (segs.length !== 4) return null;
        let val = 0n;
        for (const s of segs) {
            const n = Number(s);
            if (Number.isNaN(n) || n < 0 || n > 255) return null;
            val = (val << 8n) + BigInt(n);
        }
        // map IPv4 into IPv6-mapped space for uniform compare: ::ffff:a.b.c.d
        return (0xffffn << 32n) + val;
    }
}

function parseCidr(cidr: string): Cidr | null {
    const [ip, bitsStr] = cidr.trim().split('/');
    if (!ip || !bitsStr) return null;
    const bits = Number(bitsStr);
    if (Number.isNaN(bits) || bits < 0 || bits > 128) return null;
    const ipVal = parseIpToBigInt(ip);
    if (ipVal === null) return null;
    return { base: ipVal, maskBits: bits };
}

function ipInCidrs(ip: string, cidrs: string[]): boolean {
    const ipVal = parseIpToBigInt(ip);
    if (ipVal === null) return false;
    for (const c of cidrs) {
        const parsed = parseCidr(c);
        if (!parsed) continue;
        const { base, maskBits } = parsed;
        const mask = maskBits === 0 ? 0n : (~0n) << BigInt(128 - maskBits);
        if ((ipVal & mask) === (base & mask)) return true;
    }
    return false;
}

function isProtectedPath(pathname: string, protectedPrefixes: string[]): boolean {
    return protectedPrefixes.some((p) => p && pathname.startsWith(p));
}

export const handle: Handle = async ({ event, resolve }) => {
    const ip = getClientIp(event) ?? 'unknown';

    // Make IP available to endpoints via locals
    event.locals.clientIp = ip;

    // Pull config from env
    const subnetEnv = process.env['ALLOWED_SUBNETS'] ?? '';
    const protectedEnv = process.env['PROTECTED_PATH_PREFIXES'] ?? '';
    const allowedCidrs = subnetEnv.split(',').map((s) => s.trim()).filter(Boolean);
    const protectedPrefixes = protectedEnv.split(',').map((s) => s.trim()).filter(Boolean);

    // Log all incoming requests with IP and path
    console.log(`[req] ${event.request.method} ${event.url.pathname} ip=${ip}`);

    // Enforce subnet on protected paths
    if (isProtectedPath(event.url.pathname, protectedPrefixes)) {
        const ok = ip !== 'unknown' && ipInCidrs(ip, allowedCidrs);
        if (!ok) {
            console.log(`[req] ${event.request.method} ${event.url.pathname} ip=${ip} denied`);
            return new Response('Forbidden! This page is for internal use only. Want to know more? Come visit our booth!', { status: 403 });
        }
    }

    return resolve(event);
};
