<script lang="ts">
	import ShortcutHost from '$lib/components/shortcuts/ShortcutHost.svelte';
	import { registerShortcuts } from '$lib/shortcuts/registry.svelte';
	import type { ScopeLayer } from '$lib/shortcuts/dispatch';

	let { layers }: { layers: ScopeLayer[] } = $props();

	// svelte-ignore state_referenced_locally
	for (const layer of layers) {
		registerShortcuts(() => layer);
	}
</script>

<ShortcutHost />
