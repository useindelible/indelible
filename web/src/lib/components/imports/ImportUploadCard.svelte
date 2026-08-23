<script lang="ts">
	import Button from '$lib/components/ui/Button.svelte';
	import { t } from '$lib/i18n';

	interface Props {
		title: string;
		description?: string;
		acceptedMimeTypes?: string[];
		acceptedExtensions?: string[];
		maxBytes?: number;
		busy?: boolean;
		disabled?: boolean;
		errorMessage?: string | null;
		submitLabel?: string;
		onSubmit: (file: File) => void;
	}

	let {
		title,
		description,
		acceptedMimeTypes = [],
		acceptedExtensions = [],
		maxBytes,
		busy = false,
		disabled = false,
		errorMessage,
		submitLabel,
		onSubmit
	}: Props = $props();

	let dragActive = $state(false);
	let selectedFile: File | null = $state(null);
	let validationError = $state<string | null>(null);
	let inputElement = $state<HTMLInputElement | undefined>(undefined);

	const acceptValue = $derived([...acceptedMimeTypes, ...acceptedExtensions].join(','));

	function isAccepted(file: File): boolean {
		if (acceptedMimeTypes.length === 0 && acceptedExtensions.length === 0) {
			return true;
		}
		if (acceptedMimeTypes.includes(file.type)) {
			return true;
		}
		const lower = file.name.toLowerCase();
		return acceptedExtensions.some((ext) => lower.endsWith(ext.toLowerCase()));
	}

	function validate(file: File): string | null {
		if (!isAccepted(file)) {
			const list =
				[...acceptedExtensions, ...acceptedMimeTypes].join(', ') || $t('imports_this_provider');
			return $t('imports_unsupported_file_type', { values: { types: list } });
		}
		if (maxBytes !== undefined && file.size > maxBytes) {
			const mb = Math.round(maxBytes / (1024 * 1024));
			return $t('imports_file_too_large', { values: { size: mb } });
		}
		return null;
	}

	function setFile(file: File | null) {
		validationError = null;
		if (!file) {
			selectedFile = null;
			return;
		}
		const err = validate(file);
		if (err) {
			validationError = err;
			selectedFile = null;
			return;
		}
		selectedFile = file;
	}

	function onDragOver(event: DragEvent) {
		event.preventDefault();
		dragActive = true;
	}

	function onDragLeave() {
		dragActive = false;
	}

	function onDrop(event: DragEvent) {
		event.preventDefault();
		dragActive = false;
		const file = event.dataTransfer?.files?.[0] ?? null;
		setFile(file);
	}

	function onChange(event: Event) {
		const input = event.currentTarget as HTMLInputElement;
		const file = input.files?.[0] ?? null;
		setFile(file);
	}

	function triggerPicker() {
		inputElement?.click();
	}

	function submit() {
		if (!selectedFile || busy || disabled) return;
		onSubmit(selectedFile);
	}

	function clearSelection() {
		selectedFile = null;
		validationError = null;
		if (inputElement) {
			inputElement.value = '';
		}
	}
</script>

<section class="upload-card">
	<header>
		<h3 class="title">{title}</h3>
		{#if description}
			<p class="description">{description}</p>
		{/if}
	</header>

	<div
		class="dropzone"
		class:active={dragActive}
		class:has-file={selectedFile !== null}
		class:has-error={validationError !== null}
		ondragover={onDragOver}
		ondragleave={onDragLeave}
		ondrop={onDrop}
		role="button"
		tabindex="0"
		onclick={triggerPicker}
		onkeydown={(event) => {
			if (event.key === 'Enter' || event.key === ' ') {
				event.preventDefault();
				triggerPicker();
			}
		}}
	>
		<input
			type="file"
			class="hidden-input"
			bind:this={inputElement}
			onchange={onChange}
			accept={acceptValue || undefined}
			data-testid="file-input"
		/>
		{#if selectedFile}
			<p class="filename">{selectedFile.name}</p>
			<p class="filemeta">{Math.round(selectedFile.size / 1024)} KB</p>
		{:else}
			<p class="prompt">{$t('imports_drop_file')}</p>
			{#if acceptedExtensions.length > 0}
				<p class="hint">
					{$t('imports_accepted_types', { values: { types: acceptedExtensions.join(', ') } })}
				</p>
			{/if}
		{/if}
	</div>

	{#if validationError}
		<p class="error" role="alert">{validationError}</p>
	{/if}

	{#if errorMessage}
		<p class="error" role="alert">{errorMessage}</p>
	{/if}

	<div class="actions">
		{#if selectedFile}
			<Button variant="tertiary" size="sm" onclick={clearSelection} disabled={busy || disabled}>
				{$t('common_clear')}
			</Button>
		{/if}
		<Button
			variant="primary"
			size="sm"
			onclick={submit}
			loading={busy}
			disabled={!selectedFile || disabled}
		>
			{submitLabel ?? $t('imports_start')}
		</Button>
	</div>
</section>

<style>
	.upload-card {
		display: flex;
		flex-direction: column;
		gap: 12px;
		padding: 16px;
		border-radius: 12px;
		background: var(--bg-secondary);
		border: 0.5px solid var(--border-primary);
	}

	.title {
		font-family: var(--font-sans);
		font-size: 15px;
		font-weight: 600;
		letter-spacing: -0.01em;
		color: var(--text-primary);
		margin: 0 0 4px;
	}

	.description {
		font-family: var(--font-sans);
		font-size: 13px;
		color: var(--text-secondary);
		margin: 0;
	}

	.dropzone {
		border: 1px dashed var(--border-primary);
		border-radius: 10px;
		padding: 24px;
		text-align: center;
		cursor: pointer;
		background: var(--bg-primary);
		transition:
			border-color 120ms ease,
			background 120ms ease;
	}

	.dropzone:hover,
	.dropzone:focus-visible,
	.dropzone.active {
		border-color: var(--accent);
		background: var(--fill-selected);
		outline: none;
	}

	.dropzone.has-file {
		border-style: solid;
		border-color: var(--accent);
	}

	.dropzone.has-error {
		border-color: var(--destructive);
	}

	.hidden-input {
		display: none;
	}

	.prompt {
		font-family: var(--font-sans);
		font-size: 14px;
		color: var(--text-primary);
		margin: 0;
	}

	.hint {
		font-family: var(--font-sans);
		font-size: 12px;
		color: var(--text-tertiary);
		margin: 6px 0 0;
	}

	.filename {
		font-family: var(--font-sans);
		font-size: 14px;
		font-weight: 500;
		color: var(--text-primary);
		margin: 0;
		word-break: break-all;
	}

	.filemeta {
		font-family: var(--font-sans);
		font-size: 12px;
		color: var(--text-tertiary);
		margin: 4px 0 0;
	}

	.error {
		font-family: var(--font-sans);
		font-size: 13px;
		color: var(--destructive);
		margin: 0;
	}

	.actions {
		display: flex;
		gap: 8px;
		justify-content: flex-end;
	}
</style>
