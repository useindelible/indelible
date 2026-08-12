<script lang="ts">
	import type { DocumentListEntry } from '$lib/api';
	import DetailPanel from '$lib/components/library/DetailPanel.svelte';

	interface Props {
		item: DocumentListEntry;
		chatAvailable?: boolean;
	}

	let { item, chatAvailable = true }: Props = $props();
</script>

<div class="reader-detail">
	<DetailPanel {item} {chatAvailable} />
</div>

<style>
	.reader-detail {
		display: contents;
		--detail-tabs-height: 44px;
	}
</style>
