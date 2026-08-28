<script lang="ts">
	import { setDocumentTitle } from '$lib/stores/page-title.svelte';

	let { value = null }: { value?: string | null } = $props();

	setDocumentTitle(() => value);
</script>
