<script lang="ts">
	import { resolve } from '$app/paths';
	import SidebarSection from '$lib/components/layout/SidebarSection.svelte';
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
		label: string;
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
		{ href: '/library/articles', label: 'Articles', icon: 'articles', countKey: 'article' },
		{ href: '/library/books', label: 'Books', icon: 'books', countKey: 'book' },
		{ href: '/library/emails', label: 'Emails', icon: 'emails', countKey: 'email' },
		{ href: '/library/pdfs', label: 'PDFs', icon: 'pdfs', countKey: 'pdf' },
		{ href: '/library/tweets', label: 'Tweets', icon: 'tweets', countKey: 'tweet' },
		{ href: '/library/videos', label: 'Videos', icon: 'videos', countKey: 'video' },
		{ href: '/library/podcasts', label: 'Podcasts', icon: 'podcasts', countKey: 'podcast' },
		{ href: '/tags', label: 'Tags', icon: 'tags' },
		{ href: '/feed', label: 'Feed', icon: 'feed' }
	];

	function countFor(key?: string): number | undefined {
		if (!showCountBadge || !key || itemTypeCounts[key] === undefined) return undefined;
		return itemTypeCounts[key];
	}
</script>

<li>
	<SidebarNavItem
		href={resolve('/dashboard')}
		label="Home"
		icon="home"
		active={isActive('/dashboard')}
	/>
</li>

<li>
	<SidebarSection label="Library">
		<ul class="nav-sublist" role="list">
			{#each libraryItems as item (item.href)}
				<li>
					<SidebarNavItem
						href={resolve(item.href)}
						label={item.label}
						icon={item.icon}
						active={isActive(item.href)}
						badge={countFor(item.countKey)}
					/>
				</li>
			{/each}
		</ul>
	</SidebarSection>
</li>
