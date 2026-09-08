<script lang="ts">
	import LibraryShell from '$lib/components/library/LibraryShell.svelte';
	import LibrarySidebar from '$lib/components/library/LibrarySidebar.svelte';
	import { getLibrary } from '$lib/stores/library.svelte';
	import { registerShortcuts } from '$lib/shortcuts/registry.svelte';

	let { children } = $props();
	const lib = getLibrary();

	function moveSelection(offset: number): void {
		const { items, selectedId } = lib;
		const index = items.findIndex((item) => item.id === selectedId);
		const next = Math.min(Math.max(index + offset, 0), items.length - 1);
		lib.setSelectedId(items[next]?.id ?? null);
	}

	registerShortcuts(() => ({
		scope: 'library',
		handlers: {
			select_next: () => moveSelection(1),
			select_prev: () => moveSelection(-1),
			triage_archive: () => {
				const { selectedId } = lib;
				if (selectedId) lib.triageAction(selectedId, 'archive');
			}
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
