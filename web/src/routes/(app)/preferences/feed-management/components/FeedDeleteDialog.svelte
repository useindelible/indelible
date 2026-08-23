<script lang="ts">
	import ConfirmDialog from '$lib/components/ui/ConfirmDialog.svelte';
	import type { Feed } from '../feed-model';
	import { t } from '$lib/i18n';

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
	title={$t('feed_management_remove_feed')}
	confirmLabel={$t('common_remove')}
	cancelLabel={$t('common_cancel')}
	{onConfirm}
	{onCancel}
>
	<p class="delete-copy">
		{$t('feed_management_remove_confirm', {
			values: { feed: feed?.name ?? $t('feed_management_this_feed') }
		})}
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
</style>
