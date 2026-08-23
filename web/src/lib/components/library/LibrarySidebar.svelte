<script lang="ts">
	import { t } from '$lib/i18n';
	import { onMount } from 'svelte';
	import { page } from '$app/state';
	import { goto } from '$app/navigation';
	import { resolve } from '$app/paths';
	import { getAuth } from '$lib/stores/auth.svelte';
	import { getModalStore } from '$lib/stores/addItemModal.svelte';
	import { getSidebar } from '$lib/stores/sidebar.svelte';
	import { getLibrary } from '$lib/stores/library.svelte';
	import { saveTheme } from '$lib/styles/theme';
	import { loadPreferencesSettings, savePreferencesSettings } from '$lib/api/settings';
	import { getAppPreferences } from '$lib/stores/app-preferences.svelte';
	import SidebarHeader from './SidebarHeader.svelte';
	import SidebarNavItem from './SidebarNavItem.svelte';
	import SidebarNavList from './SidebarNavList.svelte';
	import SidebarCollectionsSection from './SidebarCollectionsSection.svelte';
	import SidebarUserMenu from './SidebarUserMenu.svelte';
	import {
		getDefaultHomePath,
		getSmartListHref,
		isSidebarPathActive
	} from './library-sidebar-model';
	import './library-sidebar.css';

	const auth = getAuth();
	const modal = getModalStore();
	const sidebar = getSidebar();
	const lib = getLibrary();
	const appPrefs = getAppPreferences();

	let isDark = $state(false);
	let initialized = $state(false);
	let popupOpen = $state(false);
	let userPopupWrap: HTMLElement | null = null;

	onMount(() => {
		appPrefs.load();
	});

	const homeHref = $derived(resolve(getDefaultHomePath(appPrefs.defaultView)));

	$effect(() => {
		if (!popupOpen) return;
		function onDocClick(e: MouseEvent) {
			if (!userPopupWrap?.contains(e.target as Node)) {
				popupOpen = false;
			}
		}
		function onKeydown(e: KeyboardEvent) {
			if (e.key === 'Escape') popupOpen = false;
		}
		document.addEventListener('click', onDocClick, { capture: true });
		document.addEventListener('keydown', onKeydown);
		return () => {
			document.removeEventListener('click', onDocClick, { capture: true });
			document.removeEventListener('keydown', onKeydown);
		};
	});

	$effect(() => {
		isDark = document.documentElement.dataset.theme === 'dark';
		const observer = new MutationObserver(() => {
			isDark = document.documentElement.dataset.theme === 'dark';
		});
		observer.observe(document.documentElement, {
			attributes: true,
			attributeFilter: ['data-theme']
		});
		return () => observer.disconnect();
	});

	$effect(() => {
		if (!initialized) {
			initialized = true;
			sidebar.initSidebar();
		}
	});

	async function toggleTheme() {
		const next = isDark ? 'light' : 'dark';
		saveTheme(next);
		isDark = next === 'dark';

		const result = await loadPreferencesSettings();
		if (result.success) {
			await savePreferencesSettings({ ...result.data, theme: next });
		}
	}

	function isActive(href: string): boolean {
		return isSidebarPathActive(page.url.pathname, href);
	}

	function smartListHref(id: string): string {
		return getSmartListHref(id, resolve('/library'));
	}

	async function handleLogout() {
		await auth.logout();
		goto(resolve('/login'));
	}

	function handleAddClick(e: MouseEvent) {
		e.stopPropagation();
		modal.togglePopover();
	}
</script>

<nav class="library-sidebar" aria-label={$t('library_application_navigation')}>
	<SidebarHeader
		{homeHref}
		popoverOpen={modal.popoverOpen}
		onHideSidebar={() => lib.toggleSidebarVisibility(true)}
		onAddClick={handleAddClick}
	/>

	<ul class="nav-list" role="list">
		<SidebarNavList
			{isActive}
			showCountBadge={lib.showCountBadge}
			itemTypeCounts={sidebar.itemTypeCounts}
		/>

		<SidebarCollectionsSection
			pinnedSmartLists={sidebar.pinnedSmartLists}
			activeSmartListId={page.url.searchParams.get('smart_list')}
			{isActive}
			{smartListHref}
		/>

		<li>
			<SidebarNavItem
				href={resolve('/trash')}
				label={$t('common_trash')}
				icon="trash"
				active={isActive('/trash')}
				badge={sidebar.trashCount > 0 ? sidebar.trashCount : undefined}
			/>
		</li>

		<li class="nav-divider" role="separator"></li>

		<li>
			<SidebarNavItem
				href={resolve('/search')}
				label={$t('common_search')}
				icon="search"
				active={isActive('/search')}
			/>
		</li>
		<li>
			<SidebarNavItem
				href={resolve('/preferences')}
				label={$t('common_preferences')}
				icon="preferences"
				active={isActive('/preferences')}
			/>
		</li>
	</ul>

	<SidebarUserMenu
		user={auth.user}
		{popupOpen}
		{isDark}
		onPopupOpenChange={(open) => {
			popupOpen = open;
		}}
		onThemeToggle={toggleTheme}
		onLogout={handleLogout}
		onWrapMount={(node) => {
			userPopupWrap = node;
		}}
	/>
</nav>
