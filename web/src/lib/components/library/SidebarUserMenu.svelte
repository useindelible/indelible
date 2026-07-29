<script lang="ts">
	import { onMount } from 'svelte';
	import { resolve } from '$app/paths';
	import type { AuthUser } from '$lib/stores/auth.svelte';
	import { getInitials } from './library-sidebar-model';

	interface Props {
		user: AuthUser | null;
		popupOpen: boolean;
		isDark: boolean;
		onPopupOpenChange: (open: boolean) => void;
		onThemeToggle: () => void | Promise<void>;
		onLogout: () => void | Promise<void>;
		onWrapMount: (node: HTMLElement | null) => void;
	}

	let { user, popupOpen, isDark, onPopupOpenChange, onThemeToggle, onLogout, onWrapMount }: Props =
		$props();

	let userPopupWrap: HTMLElement | null = $state(null);

	onMount(() => {
		onWrapMount(userPopupWrap);
		return () => onWrapMount(null);
	});
</script>

<div class="sidebar-footer">
	<div class="user-popup-wrap" bind:this={userPopupWrap}>
		{#if popupOpen}
			<div class="user-popup" role="menu">
				<a
					href={resolve('/preferences/account')}
					class="user-popup-item"
					role="menuitem"
					onclick={() => onPopupOpenChange(false)}
				>
					<svg viewBox="0 0 24 24" aria-hidden="true">
						<circle cx="12" cy="8" r="4" />
						<path d="M4 22a8 8 0 0 1 16 0" />
					</svg>
					Account
				</a>
				<div class="user-popup-divider" role="separator"></div>
				<button type="button" class="user-popup-item danger" role="menuitem" onclick={onLogout}>
					<svg viewBox="0 0 24 24" aria-hidden="true">
						<path d="M9 21H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h4" />
						<polyline points="16 17 21 12 16 7" />
						<line x1="21" y1="12" x2="9" y2="12" />
					</svg>
					Sign out
				</button>
			</div>
		{/if}
		<div class="user-row">
			<button
				type="button"
				class="user-row-btn"
				class:active={popupOpen}
				onclick={() => onPopupOpenChange(!popupOpen)}
				aria-label="Account menu"
				aria-haspopup="menu"
				aria-expanded={popupOpen}
			>
				{#if user?.avatar_url}
					<img src={user.avatar_url} alt={user.display_name} class="user-avatar user-avatar-img" />
				{:else}
					<div class="user-avatar" aria-hidden="true">
						{getInitials(user?.display_name ?? 'U')}
					</div>
				{/if}
				<span class="user-name">{user?.display_name ?? ''}</span>
			</button>
			<button
				type="button"
				class="theme-toggle-sm"
				aria-label="Toggle theme"
				onclick={onThemeToggle}
			>
				{#if isDark}
					<svg
						viewBox="0 0 24 24"
						fill="none"
						stroke="currentColor"
						stroke-width="1.5"
						aria-hidden="true"
					>
						<circle cx="12" cy="12" r="5" />
						<line x1="12" y1="1" x2="12" y2="3" />
						<line x1="12" y1="21" x2="12" y2="23" />
						<line x1="4.22" y1="4.22" x2="5.64" y2="5.64" />
						<line x1="18.36" y1="18.36" x2="19.78" y2="19.78" />
						<line x1="1" y1="12" x2="3" y2="12" />
						<line x1="21" y1="12" x2="23" y2="12" />
						<line x1="4.22" y1="19.78" x2="5.64" y2="18.36" />
						<line x1="18.36" y1="5.64" x2="19.78" y2="4.22" />
					</svg>
				{:else}
					<svg
						viewBox="0 0 24 24"
						fill="none"
						stroke="currentColor"
						stroke-width="1.5"
						aria-hidden="true"
					>
						<path d="M21 12.79A9 9 0 1 1 11.21 3 7 7 0 0 0 21 12.79z" />
					</svg>
				{/if}
			</button>
		</div>
	</div>
</div>
