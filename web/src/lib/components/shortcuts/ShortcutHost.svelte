<script lang="ts">
	import { handleGlobalKeydown } from '$lib/shortcuts/registry.svelte';
</script>

<svelte:window onkeydown={handleGlobalKeydown} />
