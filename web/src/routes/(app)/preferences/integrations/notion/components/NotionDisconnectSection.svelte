<script lang="ts">
	import IntegrationDisconnectDialog from '$lib/components/integrations/IntegrationDisconnectDialog.svelte';

	interface Props {
		open: boolean;
		busy: boolean;
		errorMessage: string | null;
		onCancel: () => void;
		onConfirm: () => void;
	}

	let { open, busy, errorMessage, onCancel, onConfirm }: Props = $props();
</script>

<IntegrationDisconnectDialog
	{open}
	providerName="Notion"
	{busy}
	{errorMessage}
	{onCancel}
	{onConfirm}
/>
