<script lang="ts">
	import { resolve } from '$app/paths';
	import type { SmartListResponse } from '$lib/api/generated/types.gen';
	import SidebarSection from '$lib/components/layout/SidebarSection.svelte';
	import { t } from '$lib/i18n';
	import SidebarNavItem from './SidebarNavItem.svelte';

	interface Props {
		pinnedSmartLists: SmartListResponse[];
		activeSmartListId: string | null;
		isActive: (href: string) => boolean;
		smartListHref: (id: string) => string;
	}

	let { pinnedSmartLists, activeSmartListId, isActive, smartListHref }: Props = $props();
</script>

<li>
	<SidebarSection label={$t('library_collections')}>
		<SidebarNavItem
			href={resolve('/collections')}
			label={$t('library_all_collections')}
			icon="collections"
			active={isActive('/collections')}
		/>
	</SidebarSection>
</li>

<li class="nav-divider-thin" role="separator"></li>

{#if pinnedSmartLists.length > 0}
	<li>
		<SidebarSection label={$t('library_pinned')}>
			<ul class="nav-sublist" role="list">
				{#each pinnedSmartLists as smartList (smartList.id)}
					<li>
						<SidebarNavItem
							href={smartListHref(smartList.id)}
							label={smartList.name}
							icon="pinned"
							emoji={smartList.icon}
							active={activeSmartListId === smartList.id}
						/>
					</li>
				{/each}
			</ul>
		</SidebarSection>
	</li>
{/if}
