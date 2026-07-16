<script lang="ts">
	import '../app.css';
	import { goto } from '$app/navigation';
	import { page } from '$app/stores';
	import { carbonTheme } from '$lib/common/theme';
	import ToastContainer from '$lib/components/Common/ToastContainer.svelte';
	import { isImmersiveMode } from '$lib/stores/uiStore';
	import { Theme } from 'carbon-components-svelte';
	import { onMount } from 'svelte';

	let { children } = $props();
	let currentTheme = $state<'white' | 'g10' | 'g80' | 'g90' | 'g100'>('white');
	let railCollapsed = $state(false);
	let mobileOpen = $state(false);
	let isMobile = $state(false);
	let online = $state(true);
	let now = $state(new Date());

	type NavItem = { href: string; label: string; icon: string };
	type NavGroup = { label: string; items: NavItem[] };

	const NAV_GROUPS: NavGroup[] = [
		{ label: 'Mission', items: [{ href: '/', label: 'Overview', icon: 'fa-solid fa-gauge-high' }] },
		{
			label: 'Flight',
			items: [
				{ href: '/state', label: 'State', icon: 'fa-solid fa-flag-checkered' },
				{ href: '/altitude', label: 'Altitude', icon: 'fa-solid fa-mountain' },
				{ href: '/imu', label: 'IMU', icon: 'fa-solid fa-microchip' },
				{ href: '/trajectory', label: 'Trajectory', icon: 'fa-solid fa-route' },
				{ href: '/navball', label: 'Navball', icon: 'fa-solid fa-compass' }
			]
		},
		{
			label: 'Sensors',
			items: [
				{ href: '/sbg', label: 'SBG', icon: 'fa-solid fa-satellite' },
				{ href: '/radio', label: 'Radio', icon: 'fa-solid fa-signal' }
			]
		},
		{
			label: 'Operations',
			items: [
				{ href: '/commands', label: 'Commands', icon: 'fa-solid fa-terminal' },
				{ href: '/services', label: 'Services', icon: 'fa-solid fa-server' },
				{ href: '/services/logs', label: 'Logs', icon: 'fa-solid fa-file-lines' }
			]
		},
		{
			label: 'Tools',
			items: [
				{ href: '/mock', label: 'Mock Data', icon: 'fa-solid fa-sliders' },
				{ href: '/map-downloader', label: 'Offline Maps', icon: 'fa-solid fa-map' },
				{ href: '/debug/coord-mock', label: 'Coordinates', icon: 'fa-solid fa-crosshairs' }
			]
		}
	];

	const isActive = (href: string) =>
		href === '/'
			? $page.url.pathname === '/'
			: $page.url.pathname === href || $page.url.pathname.startsWith(`${href}/`);

	function toggleRail() {
		if (isMobile) mobileOpen = !mobileOpen;
		else {
			railCollapsed = !railCollapsed;
			localStorage.setItem('rgs.ui.shell.v1', JSON.stringify({ railCollapsed }));
		}
	}

	function navigate(href: string) {
		mobileOpen = false;
		goto(href);
	}

	onMount(() => {
		const unsubscribeTheme = carbonTheme.subscribe((theme) => (currentTheme = theme));
		const media = window.matchMedia('(max-width: 767px)');
		const updateMedia = () => {
			isMobile = media.matches;
			if (!isMobile) mobileOpen = false;
		};
		updateMedia();
		media.addEventListener('change', updateMedia);

		try {
			const saved = JSON.parse(localStorage.getItem('rgs.ui.shell.v1') ?? '{}');
			railCollapsed = Boolean(saved.railCollapsed);
		} catch {
			railCollapsed = false;
		}

		const updateOnline = () => (online = navigator.onLine);
		const clock = window.setInterval(() => (now = new Date()), 1000);
		const handleKey = (event: KeyboardEvent) => {
			if (event.shiftKey && event.key.toLowerCase() === 'i') {
				event.preventDefault();
				isImmersiveMode.update((value) => !value);
			}
			if (event.key === 'Escape' && mobileOpen) mobileOpen = false;
		};

		window.addEventListener('online', updateOnline);
		window.addEventListener('offline', updateOnline);
		window.addEventListener('keydown', handleKey);

		return () => {
			unsubscribeTheme();
			media.removeEventListener('change', updateMedia);
			window.clearInterval(clock);
			window.removeEventListener('online', updateOnline);
			window.removeEventListener('offline', updateOnline);
			window.removeEventListener('keydown', handleKey);
		};
	});
</script>

<Theme theme={currentTheme}>
	<ToastContainer />
	{#if $isImmersiveMode}
		<div class="immersive-shell">
			<button class="immersive-exit" type="button" onclick={() => isImmersiveMode.set(false)}>
				<i class="fa-solid fa-compress" aria-hidden="true"></i>
				<span>Exit immersive</span>
			</button>
			{@render children?.()}
		</div>
	{:else}
		<div class:rail-collapsed={railCollapsed} class="app-shell">
			<header class="topbar">
				<button
					class="shell-button"
					type="button"
					aria-label="Toggle navigation"
					onclick={toggleRail}
				>
					<i class="fa-solid fa-bars" aria-hidden="true"></i>
				</button>
				<a class="brand" href="/" aria-label="RGS overview">
					<strong>RGS</strong><span>GROUND STATION</span>
				</a>
				<div class="topbar-status" aria-live="polite">
					<span class:offline={!online} class="network-state">
						<span class="status-dot"></span>{online ? 'CLIENT ONLINE' : 'CLIENT OFFLINE'}
					</span>
					<time datetime={now.toISOString()}>{now.toLocaleTimeString([], { hour12: false })}</time>
					<button
						class="shell-button desktop-only"
						type="button"
						title="Immersive mode (Shift+I)"
						aria-label="Enter immersive mode"
						onclick={() => isImmersiveMode.set(true)}
					>
						<i class="fa-solid fa-expand" aria-hidden="true"></i>
					</button>
				</div>
			</header>

			{#if mobileOpen}
				<button class="nav-scrim" aria-label="Close navigation" onclick={() => (mobileOpen = false)}
				></button>
			{/if}

			<aside class:open={mobileOpen} class="side-rail" aria-label="Primary navigation">
				<nav>
					{#each NAV_GROUPS as group}
						<section class="nav-group">
							<div class="nav-group-label">{group.label}</div>
							{#each group.items as item}
								<a
									href={item.href}
									class:active={isActive(item.href)}
									class="nav-link"
									title={railCollapsed ? item.label : undefined}
									onclick={(event) => {
										event.preventDefault();
										navigate(item.href);
									}}
								>
									<i class={item.icon} aria-hidden="true"></i><span>{item.label}</span>
								</a>
							{/each}
						</section>
					{/each}
				</nav>
				<a
					class:active={isActive('/settings')}
					class="nav-link settings-link"
					href="/settings"
					onclick={(event) => {
						event.preventDefault();
						navigate('/settings');
					}}
				>
					<i class="fa-solid fa-gear" aria-hidden="true"></i><span>Settings</span>
				</a>
			</aside>

			<main class="app-content">{@render children?.()}</main>
		</div>
	{/if}
</Theme>

<style>
	.app-shell {
		min-height: 100vh;
	}
	.topbar {
		position: fixed;
		z-index: 1000;
		inset: 0 0 auto 0;
		display: flex;
		height: var(--rgs-header-h);
		align-items: center;
		border-bottom: 1px solid var(--rgs-border);
		background: var(--rgs-surface);
		color: var(--rgs-text);
	}
	.shell-button {
		display: grid;
		width: 40px;
		height: 40px;
		place-items: center;
		border: 0;
		border-right: 1px solid var(--rgs-border-subtle);
		background: transparent;
		color: inherit;
		cursor: pointer;
	}
	.shell-button:hover {
		background: var(--rgs-surface-alt);
	}
	.brand {
		display: flex;
		align-items: baseline;
		gap: 10px;
		padding: 0 12px;
		color: inherit;
		text-decoration: none;
	}
	.brand strong {
		font-size: 1.05rem;
		letter-spacing: 0.08em;
	}
	.brand span {
		color: var(--rgs-text-muted);
		font-size: 0.72rem;
		letter-spacing: 0.12em;
	}
	.topbar-status {
		display: flex;
		height: 100%;
		align-items: center;
		gap: 16px;
		margin-left: auto;
		font-size: 0.72rem;
	}
	.network-state {
		display: inline-flex;
		align-items: center;
		gap: 6px;
		color: var(--rgs-good);
		font-weight: 600;
	}
	.network-state.offline {
		color: var(--rgs-danger);
	}
	.status-dot {
		width: 7px;
		height: 7px;
		border-radius: 50%;
		background: currentColor;
	}
	.side-rail {
		position: fixed;
		z-index: 900;
		inset: var(--rgs-header-h) auto 0 0;
		display: flex;
		width: var(--rgs-rail-w);
		flex-direction: column;
		border-right: 1px solid var(--rgs-border);
		background: var(--rgs-surface);
		transition: width 120ms ease;
	}
	.side-rail nav {
		overflow-y: auto;
		padding: 4px 0 8px;
	}
	.nav-group {
		padding-top: 6px;
	}
	.nav-group-label {
		padding: 4px 12px;
		color: var(--rgs-text-muted);
		font-size: 0.65rem;
		letter-spacing: 0.12em;
		text-transform: uppercase;
	}
	.nav-link {
		display: flex;
		height: 32px;
		align-items: center;
		gap: 10px;
		padding: 0 12px;
		border-left: 3px solid transparent;
		color: var(--rgs-text);
		text-decoration: none;
		white-space: nowrap;
	}
	.nav-link i {
		width: 20px;
		text-align: center;
	}
	.nav-link:hover {
		background: var(--rgs-surface-alt);
	}
	.nav-link.active {
		border-left-color: var(--rgs-accent);
		background: var(--rgs-surface-active);
		color: var(--rgs-text);
		font-weight: 600;
	}
	.settings-link {
		margin-top: auto;
		border-top: 1px solid var(--rgs-border-subtle);
	}
	.app-content {
		min-width: 0;
		min-height: calc(100vh - var(--rgs-header-h));
		margin-top: var(--rgs-header-h);
		margin-left: var(--rgs-rail-w);
		transition: margin-left 120ms ease;
	}
	.rail-collapsed .side-rail {
		width: var(--rgs-rail-collapsed-w);
	}
	.rail-collapsed .app-content {
		margin-left: var(--rgs-rail-collapsed-w);
	}
	.rail-collapsed .nav-group-label,
	.rail-collapsed .nav-link span {
		display: none;
	}
	.rail-collapsed .nav-link {
		justify-content: center;
		padding-inline: 0;
	}
	.nav-scrim {
		display: none;
	}
	.immersive-shell {
		min-height: 100vh;
	}
	.immersive-exit {
		position: fixed;
		z-index: 2000;
		top: 8px;
		right: 8px;
		display: flex;
		min-height: 32px;
		align-items: center;
		gap: 8px;
		border: 1px solid var(--rgs-border);
		background: var(--rgs-surface);
		color: var(--rgs-text);
		opacity: 0.35;
	}
	.immersive-exit:hover,
	.immersive-exit:focus-visible {
		opacity: 1;
	}
	@media (max-width: 767px) {
		.desktop-only,
		.brand span {
			display: none;
		}
		.topbar-status {
			gap: 8px;
			padding-right: 8px;
		}
		.side-rail {
			width: min(84vw, 280px);
			transform: translateX(-101%);
			transition: transform 140ms ease;
		}
		.side-rail.open {
			transform: translateX(0);
		}
		.nav-link {
			min-height: 44px;
		}
		.app-content,
		.rail-collapsed .app-content {
			margin-left: 0;
		}
		.nav-scrim {
			position: fixed;
			z-index: 850;
			inset: var(--rgs-header-h) 0 0;
			display: block;
			width: 100%;
			border: 0;
			background: rgb(0 0 0 / 45%);
		}
	}
</style>
