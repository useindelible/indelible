<script lang="ts">
	import { page } from '$app/state';
	import { resolve } from '$app/paths';
	import LibraryShell from '$lib/components/library/LibraryShell.svelte';
	import LibrarySidebar from '$lib/components/library/LibrarySidebar.svelte';
	import SettingsNav from '$lib/components/settings/SettingsNav.svelte';
	import { getViewport } from '$lib/stores/viewport.svelte';

	let { children } = $props();

	const vp = getViewport();

	// Mobile uses drill navigation: the settings nav is a full screen ("root
	// list") and each page is a detail screen with a back pill. Landing on a
	// preference URL shows the detail; back surfaces the root list.
	let showNav = $state(false);

	$effect(() => {
		if (!vp.isMobile) showNav = false;
	});

	const PAGE_TITLES: Record<string, string> = {
		account: 'Account',
		'reading-appearance': 'Reading & Appearance',
		integrations: 'Integrations',
		'feed-management': 'Feed Management',
		email: 'Email',
		archival: 'Archival',
		ai: 'Mila & AI',
		developer: 'Developer',
		'add-to-feed': 'Add to Feed',
		'add-to-library': 'Add to Library',
		'import-export': 'Import & Export',
		notion: 'Notion',
		obsidian: 'Obsidian'
	};

	const pageSlug = $derived(page.url.pathname.split('/').filter(Boolean).pop() ?? '');
	const pageTitle = $derived(PAGE_TITLES[pageSlug] ?? 'Preferences');

	// Nested integration pages back out to Integrations, not the root list.
	const isIntegrationChild = $derived(
		page.url.pathname.includes('/integrations/') && pageSlug !== 'integrations'
	);
</script>

{#snippet sidebar()}
	<LibrarySidebar />
{/snippet}

{#snippet content()}
	<div class="settings-shell" class:mobile={vp.isMobile}>
		{#if !vp.isMobile || showNav}
			{#if vp.isMobile}
				<div class="m-topbar">
					<button
						type="button"
						class="menu-btn"
						onclick={() => vp.openMobileNav()}
						aria-label="Open navigation"
					>
						<svg
							viewBox="0 0 24 24"
							fill="none"
							stroke="currentColor"
							stroke-width="1.7"
							stroke-linecap="round"
							stroke-linejoin="round"
							aria-hidden="true"
						>
							<line x1="3" y1="6" x2="21" y2="6" />
							<line x1="3" y1="12" x2="21" y2="12" />
							<line x1="3" y1="18" x2="21" y2="18" />
						</svg>
					</button>
					<span class="m-title">Preferences</span>
				</div>
			{/if}
			<div
				class="settings-sidebar"
				onclick={() => {
					if (vp.isMobile) showNav = false;
				}}
				onkeydown={(e) => {
					if (vp.isMobile && e.key === 'Enter') showNav = false;
				}}
				role={vp.isMobile ? 'navigation' : undefined}
			>
				<SettingsNav />
			</div>
		{/if}

		{#if !vp.isMobile || !showNav}
			{#if vp.isMobile}
				<div class="m-topbar">
					{#if isIntegrationChild}
						<a class="m-back" href={resolve('/preferences/integrations')}>
							<svg
								viewBox="0 0 24 24"
								fill="none"
								stroke="currentColor"
								stroke-width="2"
								stroke-linecap="round"
								stroke-linejoin="round"
								aria-hidden="true"
							>
								<polyline points="15 18 9 12 15 6" />
							</svg>
							Integrations
						</a>
					{:else}
						<button type="button" class="m-back" onclick={() => (showNav = true)}>
							<svg
								viewBox="0 0 24 24"
								fill="none"
								stroke="currentColor"
								stroke-width="2"
								stroke-linecap="round"
								stroke-linejoin="round"
								aria-hidden="true"
							>
								<polyline points="15 18 9 12 15 6" />
							</svg>
							Preferences
						</button>
					{/if}
					<span class="m-dtitle">{pageTitle}</span>
				</div>
			{/if}
			<div class="settings-content">
				{@render children()}
			</div>
		{/if}
	</div>
{/snippet}

<LibraryShell {sidebar} {content} />

<style>
	.settings-shell {
		display: flex;
		flex: 1;
		min-width: 0;
		overflow: hidden;
		background: var(--bg-primary);
	}

	.settings-sidebar {
		flex-shrink: 0;
	}

	.settings-content {
		flex: 1;
		min-width: 0;
		overflow-y: auto;
		container-type: inline-size;
	}

	.settings-shell.mobile {
		flex-direction: column;
	}

	.settings-shell.mobile .settings-sidebar {
		width: 100%;
		flex: 1;
		min-height: 0;
		overflow-y: auto;
	}

	.m-topbar {
		height: 54px;
		display: flex;
		align-items: center;
		gap: 6px;
		padding: 0 12px;
		border-bottom: 0.5px solid var(--border-primary);
		flex-shrink: 0;
		background: var(--bg-primary);
	}

	.menu-btn {
		width: 34px;
		height: 34px;
		display: flex;
		align-items: center;
		justify-content: center;
		border-radius: var(--radius-sm);
		border: none;
		background: transparent;
		color: var(--text-secondary);
		cursor: pointer;
		flex-shrink: 0;
	}

	.menu-btn:hover {
		background: var(--fill-hover);
	}

	.menu-btn svg {
		width: 20px;
		height: 20px;
	}

	.m-title {
		font-family: var(--font-sans);
		font-size: 19px;
		font-weight: 700;
		letter-spacing: -0.02em;
		color: var(--text-primary);
		padding-left: 2px;
	}

	.m-back {
		display: inline-flex;
		align-items: center;
		gap: 2px;
		color: var(--accent);
		font-family: var(--font-sans);
		font-size: 15px;
		font-weight: 400;
		letter-spacing: -0.01em;
		background: none;
		border: none;
		padding: 6px 8px 6px 4px;
		cursor: pointer;
		flex-shrink: 0;
		text-decoration: none;
	}

	.m-back svg {
		width: 18px;
		height: 18px;
	}

	.m-dtitle {
		font-family: var(--font-sans);
		font-size: 16px;
		font-weight: 600;
		letter-spacing: -0.015em;
		color: var(--text-primary);
		white-space: nowrap;
		overflow: hidden;
		text-overflow: ellipsis;
		min-width: 0;
	}
</style>
