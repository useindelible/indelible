<script lang="ts">
	import Button from '$lib/components/ui/Button.svelte';
	import ProviderIcon from './ProviderIcon.svelte';
	import ImportUploadCard from './ImportUploadCard.svelte';
	import ImportProgressCard from './ImportProgressCard.svelte';
	import ImportReport from './ImportReport.svelte';
	import type { IntegrationProvider } from '$lib/integrations/providers';
	import type { ImportJobStatusResponse } from '$lib/api';

	interface Props {
		provider: IntegrationProvider;
		activeJob: ImportJobStatusResponse | null;
		isTerminal: boolean;
		busySlug: string | null;
		uploadError: string | null;
		uploadErrorSlug: string | null;
		pollError: string | null;
		rollbackNotice: string | null;
		onBack: () => void;
		onUpload: (slug: string, file: File) => void;
		onRollback: () => void;
		onDismiss: () => void;
	}

	let {
		provider,
		activeJob,
		isTerminal,
		busySlug,
		uploadError,
		uploadErrorSlug,
		pollError,
		rollbackNotice,
		onBack,
		onUpload,
		onRollback,
		onDismiss
	}: Props = $props();

	const slug = $derived(provider.importSlug as string);
	const isBusy = $derived(busySlug === slug);
	const isDisabled = $derived(busySlug !== null && busySlug !== slug);
	const errorMessage = $derived(uploadErrorSlug === slug ? uploadError : null);

	function handleSubmit(file: File) {
		onUpload(slug, file);
	}
</script>

<div class="screen">
	<header class="screen-header">
		<Button variant="tertiary" size="sm" onclick={onBack}>← Back</Button>
		<div class="provider-identity">
			<ProviderIcon provider={provider.id} size={32} />
			<div class="provider-text">
				<h2 class="provider-name">{provider.displayName}</h2>
				{#if provider.importDescription}
					<p class="provider-desc">{provider.importDescription}</p>
				{/if}
			</div>
		</div>
	</header>

	<div class="content">
		{#if !activeJob}
			<ImportUploadCard
				title="Upload file"
				acceptedMimeTypes={provider.acceptedMimeTypes ?? []}
				acceptedExtensions={provider.acceptedExtensions ?? []}
				maxBytes={provider.maxBytes}
				busy={isBusy}
				disabled={isDisabled}
				{errorMessage}
				onSubmit={handleSubmit}
			/>
		{:else if !isTerminal}
			<ImportProgressCard job={activeJob} />
			{#if pollError}
				<p class="poll-error" role="alert">{pollError}</p>
			{/if}
			<button type="button" class="dismiss-btn" onclick={onDismiss}>Stop tracking</button>
		{:else}
			<ImportReport job={activeJob} canRollback={true} {onRollback} />
			{#if rollbackNotice}
				<p class="rollback-notice" role="status">{rollbackNotice}</p>
			{/if}
			<button type="button" class="dismiss-btn" onclick={onDismiss}>Dismiss</button>
		{/if}
	</div>
</div>

<style>
	.screen {
		display: flex;
		flex-direction: column;
		gap: 24px;
	}

	.screen-header {
		display: flex;
		flex-direction: column;
		gap: 16px;
	}

	.provider-identity {
		display: flex;
		align-items: flex-start;
		gap: 12px;
	}

	.provider-text {
		display: flex;
		flex-direction: column;
		gap: 4px;
	}

	.provider-name {
		font-family: var(--font-sans);
		font-size: 22px;
		font-weight: 700;
		letter-spacing: -0.02em;
		color: var(--text-primary);
		margin: 0;
	}

	.provider-desc {
		font-family: var(--font-sans);
		font-size: 13px;
		color: var(--text-secondary);
		margin: 0;
		line-height: 1.4;
	}

	.content {
		display: flex;
		flex-direction: column;
		gap: 12px;
	}

	.dismiss-btn {
		align-self: flex-end;
		font-family: var(--font-sans);
		font-size: 12px;
		color: var(--text-tertiary);
		background: transparent;
		border: none;
		cursor: pointer;
		padding: 4px 0;
	}

	.dismiss-btn:hover {
		color: var(--text-primary);
	}

	.poll-error {
		font-family: var(--font-sans);
		font-size: 13px;
		color: var(--destructive);
		margin: 0;
	}

	.rollback-notice {
		font-family: var(--font-sans);
		font-size: 13px;
		color: var(--success);
		margin: 0;
	}
</style>
