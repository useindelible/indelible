<script lang="ts">
	import ConfirmDialog from '$lib/components/ui/ConfirmDialog.svelte';
	import type { Feed } from '../feed-model';

	interface Props {
		open: boolean;
		feed?: Feed | null;
		onCancel: () => void;
		onConfirm: () => void;
	}

	let { open, feed = null, onCancel, onConfirm }: Props = $props();
</script>

<ConfirmDialog
	{open}
	title="Remove Feed"
	confirmLabel="Remove"
	cancelLabel="Cancel"
	{onConfirm}
	{onCancel}
>
	<p class="delete-copy">
		Are you sure you want to unsubscribe from
		<strong>{feed?.name ?? 'this feed'}</strong>? This cannot be undone.
	</p>
</ConfirmDialog>

<style>
	.delete-copy {
		margin: 0;
		font-family: var(--font-sans);
		font-size: 14px;
		color: var(--text-secondary);
		line-height: 1.5;
	}

	strong {
		color: var(--text-primary);
		font-weight: 600;
	}
</style>
