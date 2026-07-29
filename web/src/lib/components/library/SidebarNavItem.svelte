<script lang="ts">
	import type { SidebarIcon } from './library-sidebar-model';

	interface Props {
		href: string;
		label: string;
		icon?: SidebarIcon;
		emoji?: string | null;
		active?: boolean;
		badge?: number;
	}

	let { href, label, icon, emoji, active = false, badge }: Props = $props();
</script>

<!-- eslint-disable-next-line svelte/no-navigation-without-resolve -- callers pass resolved app hrefs into this generic nav item. -->
<a {href} class="nav-item" class:active aria-current={active ? 'page' : undefined}>
	<span class="nav-icon" aria-hidden="true">
		{#if emoji}
			<span class="icon-emoji">{emoji}</span>
		{:else if icon === 'home'}
			<svg
				viewBox="0 0 24 24"
				fill="none"
				stroke="currentColor"
				stroke-width="1.6"
				stroke-linecap="round"
				stroke-linejoin="round"
			>
				<path d="M3 10L12 3l9 7V22H3V10z" />
				<path d="M9 22V13h6v9" />
			</svg>
		{:else if icon === 'articles'}
			<svg
				viewBox="0 0 24 24"
				fill="none"
				stroke="currentColor"
				stroke-width="1.6"
				stroke-linecap="round"
				stroke-linejoin="round"
			>
				<rect x="3" y="3" width="18" height="18" rx="2" />
				<line x1="7" y1="8" x2="17" y2="8" />
				<line x1="7" y1="12" x2="17" y2="12" />
				<line x1="7" y1="16" x2="13" y2="16" />
			</svg>
		{:else if icon === 'books'}
			<svg
				viewBox="0 0 24 24"
				fill="none"
				stroke="currentColor"
				stroke-width="1.6"
				stroke-linecap="round"
				stroke-linejoin="round"
			>
				<path d="M2 4h6a4 4 0 0 1 4 4v13a3 3 0 0 0-3-3H2V4z" />
				<path d="M22 4h-6a4 4 0 0 0-4 4v13a3 3 0 0 1 3-3h7V4z" />
			</svg>
		{:else if icon === 'emails'}
			<svg
				viewBox="0 0 24 24"
				fill="none"
				stroke="currentColor"
				stroke-width="1.6"
				stroke-linecap="round"
				stroke-linejoin="round"
			>
				<path d="M4 4h16a2 2 0 0 1 2 2v12a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V6a2 2 0 0 1 2-2z" />
				<path d="M22 6L12 13 2 6" />
			</svg>
		{:else if icon === 'pdfs'}
			<svg
				viewBox="0 0 24 24"
				fill="none"
				stroke="currentColor"
				stroke-width="1.6"
				stroke-linecap="round"
				stroke-linejoin="round"
			>
				<path d="M5 3h14a1 1 0 0 1 1 1v16a1 1 0 0 1-1 1H5a1 1 0 0 1-1-1V4a1 1 0 0 1 1-1z" />
				<line x1="4" y1="9" x2="20" y2="9" />
				<line x1="7" y1="13" x2="17" y2="13" />
				<line x1="7" y1="17" x2="13" y2="17" />
			</svg>
		{:else if icon === 'tweets'}
			<svg viewBox="0 0 24 24" fill="currentColor" stroke="none">
				<path
					d="M18.244 2.25h3.308l-7.227 8.26 8.502 11.24H16.17l-5.214-6.817L4.99 21.75H1.68l7.73-8.835L1.254 2.25H8.08l4.713 6.231zm-1.161 17.52h1.833L7.084 4.126H5.117z"
				/>
			</svg>
		{:else if icon === 'videos'}
			<svg
				viewBox="0 0 24 24"
				fill="none"
				stroke="currentColor"
				stroke-width="1.6"
				stroke-linecap="round"
				stroke-linejoin="round"
			>
				<rect x="2" y="5" width="20" height="13" rx="2" />
				<path d="M10 8.5l5 3.5-5 3.5z" fill="currentColor" stroke="none" />
				<line x1="8" y1="21" x2="16" y2="21" />
				<line x1="12" y1="18" x2="12" y2="21" />
			</svg>
		{:else if icon === 'podcasts'}
			<svg
				viewBox="0 0 24 24"
				fill="none"
				stroke="currentColor"
				stroke-width="1.6"
				stroke-linecap="round"
				stroke-linejoin="round"
			>
				<path d="M3 18v-6a9 9 0 0 1 18 0v6" />
				<path d="M21 19a2 2 0 0 1-2 2h-1a2 2 0 0 1-2-2v-3a2 2 0 0 1 2-2h3v5z" />
				<path d="M3 19a2 2 0 0 0 2 2h1a2 2 0 0 0 2-2v-3a2 2 0 0 0-2-2H3v5z" />
			</svg>
		{:else if icon === 'tags'}
			<svg
				viewBox="0 0 24 24"
				fill="none"
				stroke="currentColor"
				stroke-width="1.6"
				stroke-linecap="round"
				stroke-linejoin="round"
			>
				<path d="M20.59 13.41l-7.17 7.17a2 2 0 0 1-2.83 0L2 12V2h10l8.59 8.59a2 2 0 0 1 0 2.82z" />
				<circle cx="7" cy="7" r="1.5" fill="currentColor" stroke="none" />
			</svg>
		{:else if icon === 'feed'}
			<svg
				viewBox="0 0 24 24"
				fill="none"
				stroke="currentColor"
				stroke-width="1.6"
				stroke-linecap="round"
				stroke-linejoin="round"
			>
				<circle cx="4" cy="20" r="1.5" fill="currentColor" stroke="none" />
				<path d="M4 13a7 7 0 0 1 7 7" />
				<path d="M4 6a14 14 0 0 1 14 14" />
			</svg>
		{:else if icon === 'collections'}
			<svg
				viewBox="0 0 24 24"
				fill="none"
				stroke="currentColor"
				stroke-width="1.6"
				stroke-linecap="round"
				stroke-linejoin="round"
			>
				<path d="M3 7v10a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2V9a2 2 0 0 0-2-2h-6l-2-2H5a2 2 0 0 0-2 2z" />
			</svg>
		{:else if icon === 'pinned'}
			<svg
				viewBox="0 0 24 24"
				fill="none"
				stroke="currentColor"
				stroke-width="1.6"
				stroke-linecap="round"
				stroke-linejoin="round"
			>
				<path d="M19 21l-7-5-7 5V5a2 2 0 0 1 2-2h10a2 2 0 0 1 2 2z" />
			</svg>
		{:else if icon === 'trash'}
			<svg
				viewBox="0 0 24 24"
				fill="none"
				stroke="currentColor"
				stroke-width="1.6"
				stroke-linecap="round"
				stroke-linejoin="round"
			>
				<path d="M3 6h18" />
				<path d="M16 6V4a2 2 0 0 0-2-2h-4a2 2 0 0 0-2 2v2" />
				<path d="M19 6l-1 14a2 2 0 0 1-2 2H8a2 2 0 0 1-2-2L5 6" />
				<line x1="10" y1="11" x2="10" y2="17" />
				<line x1="14" y1="11" x2="14" y2="17" />
			</svg>
		{:else if icon === 'search'}
			<svg
				viewBox="0 0 24 24"
				fill="none"
				stroke="currentColor"
				stroke-width="1.6"
				stroke-linecap="round"
				stroke-linejoin="round"
			>
				<circle cx="11" cy="11" r="8" />
				<line x1="21" y1="21" x2="16.65" y2="16.65" />
			</svg>
		{:else if icon === 'preferences'}
			<svg
				viewBox="0 0 24 24"
				fill="none"
				stroke="currentColor"
				stroke-width="1.6"
				stroke-linecap="round"
				stroke-linejoin="round"
			>
				<circle cx="12" cy="12" r="3" />
				<path
					d="M12 2v3M12 19v3M4.22 4.22l2.12 2.12M17.66 17.66l2.12 2.12M2 12h3M19 12h3M4.22 19.78l2.12-2.12M17.66 6.34l2.12-2.12"
				/>
			</svg>
		{/if}
	</span>
	<span class="nav-label">{label}</span>
	{#if badge !== undefined}
		<span class="nav-badge">{badge}</span>
	{/if}
</a>
