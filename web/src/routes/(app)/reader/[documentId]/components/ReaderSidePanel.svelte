<script lang="ts">
	import type { DocumentListEntry } from '$lib/api';
	import DetailPanel from '$lib/components/library/DetailPanel.svelte';

	interface Props {
		item: DocumentListEntry;
	}

	let { item }: Props = $props();
</script>

<div class="reader-detail">
	<DetailPanel {item} />
</div>

<style>
	.reader-detail {
		display: contents;
		--detail-tabs-height: 44px;
	}
</style>
