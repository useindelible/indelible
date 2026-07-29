<script lang="ts">
	import LibraryShell from '$lib/components/library/LibraryShell.svelte';
	import LibrarySidebar from '$lib/components/library/LibrarySidebar.svelte';
	import { getLibrary } from '$lib/stores/library.svelte';

	let { children } = $props();
	const lib = getLibrary();

	$effect(() => {
		function onKeydown(e: KeyboardEvent) {
			const target = e.target as HTMLElement;
			if (target.tagName === 'INPUT' || target.tagName === 'TEXTAREA' || target.isContentEditable) {
				return;
			}

			const { items, selectedId } = lib;
			const idx = items.findIndex((i) => i.id === selectedId);

			switch (e.key) {
				case 'j':
				case 'ArrowDown': {
					e.preventDefault();
					const next = idx < items.length - 1 ? idx + 1 : idx;
					lib.setSelectedId(items[next]?.id ?? null);
					break;
				}
				case 'k':
				case 'ArrowUp': {
					e.preventDefault();
					const prev = idx > 0 ? idx - 1 : 0;
					lib.setSelectedId(items[prev]?.id ?? null);
					break;
				}
				case 'a': {
					if (selectedId) {
						e.preventDefault();
						lib.triageAction(selectedId, 'archive');
					}
					break;
				}
			}
		}

		document.addEventListener('keydown', onKeydown);
		return () => document.removeEventListener('keydown', onKeydown);
	});
</script>

{#snippet sidebar()}
	<LibrarySidebar />
{/snippet}

{#snippet content()}
	{@render children()}
{/snippet}

<LibraryShell {sidebar} {content} />
