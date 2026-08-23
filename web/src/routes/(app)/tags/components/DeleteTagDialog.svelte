<script lang="ts">
	import ConfirmDialog from '$lib/components/ui/ConfirmDialog.svelte';
	import { t } from '$lib/i18n';

	interface Props {
		onClose: () => void;
		onConfirm: () => void;
	}

	let { onClose, onConfirm }: Props = $props();
</script>

<ConfirmDialog
	open
	title={$t('tag_delete')}
	message={$t('tag_delete_body')}
	confirmLabel={$t('tag_delete')}
	cancelLabel={$t('common_cancel')}
	variant="destructive"
	onCancel={onClose}
	{onConfirm}
/>
