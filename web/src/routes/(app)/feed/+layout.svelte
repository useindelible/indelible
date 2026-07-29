<script lang="ts">
	import LibraryShell from '$lib/components/library/LibraryShell.svelte';
	import LibrarySidebar from '$lib/components/library/LibrarySidebar.svelte';

	let { children } = $props();
</script>

{#snippet sidebar()}
	<LibrarySidebar />
{/snippet}

{#snippet content()}
	{@render children()}
{/snippet}

<LibraryShell {sidebar} {content} />
