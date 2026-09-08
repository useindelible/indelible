<script lang="ts">
	import { goto } from '$app/navigation';
	import { resolve } from '$app/paths';
	import LibraryShell from '$lib/components/library/LibraryShell.svelte';
	import LibrarySidebar from '$lib/components/library/LibrarySidebar.svelte';
	import { getLibrarySelection } from '$lib/stores/library-selection.svelte';
	import { registerShortcuts } from '$lib/shortcuts/registry.svelte';

	let { children } = $props();
	const selection = getLibrarySelection();

	function openSelected(): void {
		const item = selection.selectedItem;
		if (item) goto(resolve('/(app)/reader/[documentId]', { documentId: item.id }));
	}

	registerShortcuts(() => ({
		scope: 'library',
		handlers: {
			select_next: () => selection.moveSelection(1),
			select_prev: () => selection.moveSelection(-1),
			triage_inbox: () => selection.triageSelected('inbox'),
			triage_later: () => selection.triageSelected('later'),
			triage_archive: () => selection.triageSelected('archive'),
			open_item: openSelected
		}
	}));
</script>

{#snippet sidebar()}
	<LibrarySidebar />
{/snippet}

{#snippet content()}
	{@render children()}
{/snippet}

<LibraryShell {sidebar} {content} />
