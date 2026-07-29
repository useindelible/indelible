<script lang="ts">
	import ConfirmDialog from '$lib/components/ui/ConfirmDialog.svelte';

	interface Props {
		onClose: () => void;
		onConfirm: () => void;
	}

	let { onClose, onConfirm }: Props = $props();
</script>

<ConfirmDialog
	open
	title="Delete tag"
	message="This tag will be removed from all items and highlights. This cannot be undone."
	confirmLabel="Delete tag"
	cancelLabel="Cancel"
	variant="destructive"
	onCancel={onClose}
	{onConfirm}
/>
