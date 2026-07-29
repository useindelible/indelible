<script lang="ts">
	import { resolve } from '$app/paths';
	import { page } from '$app/state';

	type SettingsRoute =
		| '/preferences/account'
		| '/preferences/reading-appearance'
		| '/preferences/integrations'
		| '/preferences/feed-management'
		| '/preferences/email'
		| '/preferences/archival'
		| '/preferences/ai'
		| '/preferences/developer';

	type IconKey =
		| 'account'
		| 'reading'
		| 'integrations'
		| 'feed'
		| 'email'
		| 'archival'
		| 'mila'
		| 'developer';

	interface NavItem {
		label: string;
		href: SettingsRoute;
		icon: IconKey;
	}

	const sections: NavItem[] = [
		{ label: 'Account', href: '/preferences/account', icon: 'account' },
		{
			label: 'Reading & Appearance',
			href: '/preferences/reading-appearance',
			icon: 'reading'
		},
		{ label: 'Integrations', href: '/preferences/integrations', icon: 'integrations' },
		{ label: 'Feed Management', href: '/preferences/feed-management', icon: 'feed' },
		{ label: 'Email', href: '/preferences/email', icon: 'email' },
		{ label: 'Archival', href: '/preferences/archival', icon: 'archival' },
		{ label: 'Mila & AI', href: '/preferences/ai', icon: 'mila' },
		{ label: 'Developer', href: '/preferences/developer', icon: 'developer' }
	];

	function isActive(href: SettingsRoute): boolean {
		return page.url.pathname.startsWith(href);
	}
</script>

<nav class="settings-nav" aria-label="Settings navigation">
	<div class="settings-nav-section">Preferences</div>
	<ul class="settings-nav-list">
		{#each sections as item (item.href)}
			<li>
				<a
					href={resolve(item.href)}
					class="settings-nav-item"
					class:active={isActive(item.href)}
					aria-current={isActive(item.href) ? 'page' : undefined}
				>
					<span class="nav-glyph" aria-hidden="true">
						{#if item.icon === 'account'}
							<svg viewBox="0 0 24 24">
								<circle cx="12" cy="8" r="4" />
								<path d="M4 22a8 8 0 0 1 16 0" />
							</svg>
						{:else if item.icon === 'reading'}
							<svg viewBox="0 0 24 24">
								<path d="M2 6h7a3 3 0 0 1 3 3v11a2 2 0 0 0-2-2H2z" />
								<path d="M22 6h-7a3 3 0 0 0-3 3v11a2 2 0 0 1 2-2h8z" />
							</svg>
						{:else if item.icon === 'integrations'}
							<svg viewBox="0 0 24 24">
								<path d="M9 7h-3a3 3 0 1 0 0 6h3" />
								<path d="M15 17h3a3 3 0 1 0 0-6h-3" />
								<path d="M9 12h6" />
							</svg>
						{:else if item.icon === 'feed'}
							<svg viewBox="0 0 24 24">
								<path d="M4 11a9 9 0 0 1 9 9" />
								<path d="M4 4a16 16 0 0 1 16 16" />
								<circle cx="5" cy="19" r="1" />
							</svg>
						{:else if item.icon === 'email'}
							<svg viewBox="0 0 24 24">
								<rect x="3" y="5" width="18" height="14" rx="2" />
								<polyline points="3 7 12 13 21 7" />
							</svg>
						{:else if item.icon === 'archival'}
							<svg viewBox="0 0 24 24">
								<rect x="3" y="3" width="18" height="6" rx="1.5" />
								<path d="M5 9v10a2 2 0 0 0 2 2h10a2 2 0 0 0 2-2V9" />
								<path d="M10 13h4" />
							</svg>
						{:else if item.icon === 'mila'}
							<svg viewBox="0 0 24 24">
								<path d="M12 3l1.5 4.5L18 9l-4.5 1.5L12 15l-1.5-4.5L6 9l4.5-1.5z" />
								<path d="M19 13l.75 2.25L22 16l-2.25.75L19 19l-.75-2.25L16 16l2.25-.75z" />
								<path d="M5 17l.5 1.5L7 19l-1.5.5L5 21l-.5-1.5L3 19l1.5-.5z" />
							</svg>
						{:else if item.icon === 'developer'}
							<svg viewBox="0 0 24 24">
								<polyline points="16 18 22 12 16 6" />
								<polyline points="8 6 2 12 8 18" />
							</svg>
						{/if}
					</span>
					<span class="nav-label">{item.label}</span>
				</a>
			</li>
		{/each}
	</ul>
</nav>

<style>
	.settings-nav {
		width: 200px;
		min-width: 200px;
		height: 100%;
		padding: 28px 14px;
		background: var(--bg-primary);
		overflow-y: auto;
		flex-shrink: 0;
		display: flex;
		flex-direction: column;
		gap: 3px;
		position: relative;
	}

	.settings-nav::after {
		content: '';
		position: absolute;
		top: 18%;
		bottom: 18%;
		right: 0;
		width: 0.5px;
		background: linear-gradient(
			to bottom,
			transparent 0%,
			var(--border-primary) 22%,
			var(--border-primary) 78%,
			transparent 100%
		);
		pointer-events: none;
	}

	.settings-nav-section {
		font-size: 10px;
		font-weight: 600;
		letter-spacing: 0.13em;
		text-transform: uppercase;
		color: var(--text-tertiary);
		padding: 10px 14px 12px;
		margin-top: 4px;
	}

	.settings-nav-list {
		list-style: none;
		margin: 0;
		padding: 0;
		display: flex;
		flex-direction: column;
		gap: 3px;
	}

	.settings-nav-item {
		display: flex;
		align-items: center;
		gap: 10px;
		padding: 8px 14px;
		border-radius: 8px;
		color: var(--text-secondary);
		text-decoration: none;
		font-family: var(--font-sans);
		font-size: 13px;
		font-weight: 400;
		letter-spacing: -0.01em;
		line-height: 1.45;
		transition:
			color 140ms ease,
			background 140ms ease;
		white-space: nowrap;
		overflow: hidden;
	}

	.nav-glyph {
		width: 16px;
		height: 16px;
		flex-shrink: 0;
		display: inline-flex;
		align-items: center;
		justify-content: center;
		color: currentColor;
		opacity: 0.6;
		transition: opacity 140ms ease;
	}

	.nav-glyph svg {
		width: 14px;
		height: 14px;
		stroke: currentColor;
		fill: none;
		stroke-width: 1.6;
		stroke-linecap: round;
		stroke-linejoin: round;
	}

	.nav-label {
		overflow: hidden;
		text-overflow: ellipsis;
	}

	.settings-nav-item:hover:not(.active) {
		background: transparent;
		color: var(--text-primary);
	}

	.settings-nav-item:hover:not(.active) .nav-glyph {
		opacity: 1;
	}

	.settings-nav-item.active {
		background: transparent;
		color: var(--accent);
		font-weight: 600;
		position: relative;
	}

	.settings-nav-item.active::before {
		content: '';
		position: absolute;
		left: 4px;
		top: 8px;
		bottom: 8px;
		width: 2px;
		border-radius: 2px;
		background: var(--accent);
	}

	.settings-nav-item.active .nav-glyph {
		opacity: 1;
	}

	/* Mobile root list: the nav is the whole screen, so rows grow to touch
	   size, gain disclosure chevrons, and drop the active accent (the list
	   is a menu here, not a state indicator). */
	@media (max-width: 599px) {
		.settings-nav {
			width: 100%;
			height: 100%;
			border-right: none;
			padding: 10px 12px 28px;
		}

		.settings-nav::after {
			content: none;
		}

		.settings-nav-item {
			padding: 13px 12px;
			font-size: 15px;
			border-radius: 10px;
			color: var(--text-primary);
		}

		.settings-nav-item::after {
			content: '';
			width: 7px;
			height: 7px;
			border-right: 1.6px solid var(--text-quaternary);
			border-top: 1.6px solid var(--text-quaternary);
			transform: rotate(45deg);
			margin-left: auto;
			flex-shrink: 0;
		}

		.nav-glyph {
			width: 20px;
			height: 20px;
		}

		.nav-glyph svg {
			width: 17px;
			height: 17px;
		}

		.settings-nav-item.active {
			color: var(--text-primary);
			font-weight: 400;
		}

		.settings-nav-item.active::before {
			content: none;
		}

		.settings-nav-section {
			font-size: 11px;
			padding: 14px 12px 8px;
		}
	}
</style>
