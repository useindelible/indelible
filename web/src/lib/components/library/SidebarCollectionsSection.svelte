<script lang="ts">
	import { resolve } from '$app/paths';
	import type { SmartListResponse } from '$lib/api/generated/types.gen';
	import SidebarSection from '$lib/components/layout/SidebarSection.svelte';
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
	<SidebarSection label="Collections">
		<SidebarNavItem
			href={resolve('/collections')}
			label="All Collections"
			icon="collections"
			active={isActive('/collections')}
		/>
	</SidebarSection>
</li>

<li class="nav-divider-thin" role="separator"></li>

{#if pinnedSmartLists.length > 0}
	<li>
		<SidebarSection label="Pinned">
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
