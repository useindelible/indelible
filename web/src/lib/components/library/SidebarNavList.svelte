<script lang="ts">
	import { resolve } from '$app/paths';
	import SidebarSection from '$lib/components/layout/SidebarSection.svelte';
	import { t, type MessageKey } from '$lib/i18n';
	import SidebarNavItem from './SidebarNavItem.svelte';
	import type { SidebarIcon } from './library-sidebar-model';

	type LibraryItemHref =
		| '/library/articles'
		| '/library/books'
		| '/library/emails'
		| '/library/pdfs'
		| '/library/tweets'
		| '/library/videos'
		| '/library/podcasts'
		| '/tags'
		| '/feed';

	interface LibraryNavItem {
		href: LibraryItemHref;
		labelKey: MessageKey;
		icon: SidebarIcon;
		countKey?: string;
	}

	interface Props {
		isActive: (href: string) => boolean;
		showCountBadge: boolean;
		itemTypeCounts: Record<string, number>;
	}

	let { isActive, showCountBadge, itemTypeCounts }: Props = $props();

	const libraryItems: LibraryNavItem[] = [
		{
			href: '/library/articles',
			labelKey: 'library_nav_articles',
			icon: 'articles',
			countKey: 'article'
		},
		{ href: '/library/books', labelKey: 'library_nav_books', icon: 'books', countKey: 'book' },
		{ href: '/library/emails', labelKey: 'library_nav_emails', icon: 'emails', countKey: 'email' },
		{ href: '/library/pdfs', labelKey: 'library_nav_pdfs', icon: 'pdfs', countKey: 'pdf' },
		{ href: '/library/tweets', labelKey: 'library_nav_tweets', icon: 'tweets', countKey: 'tweet' },
		{ href: '/library/videos', labelKey: 'library_nav_videos', icon: 'videos', countKey: 'video' },
		{ href: '/tags', labelKey: 'common_tags', icon: 'tags' },
		{ href: '/feed', labelKey: 'common_feed', icon: 'feed' }
	];

	function countFor(key?: string): number | undefined {
		if (!showCountBadge || !key || itemTypeCounts[key] === undefined) return undefined;
		return itemTypeCounts[key];
	}
</script>

<li>
	<SidebarNavItem
		href={resolve('/dashboard')}
		label={$t('library_nav_home')}
		icon="home"
		active={isActive('/dashboard')}
	/>
</li>

<li>
	<SidebarSection label={$t('common_library')}>
		<ul class="nav-sublist" role="list">
			{#each libraryItems as item (item.href)}
				<li>
					<SidebarNavItem
						href={resolve(item.href)}
						label={$t(item.labelKey)}
						icon={item.icon}
						active={isActive(item.href)}
						badge={countFor(item.countKey)}
					/>
				</li>
			{/each}
		</ul>
	</SidebarSection>
</li>
