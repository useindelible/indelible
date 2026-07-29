<script lang="ts">
	import { uploadLibraryFile } from '$lib/api/uploads';
	import { getModalStore } from '$lib/stores/addItemModal.svelte';

	const modal = getModalStore();
	const ALLOWED_EXTENSIONS = ['.pdf', '.epub', '.html'] as const;

	let dialogEl = $state<HTMLDialogElement | undefined>(undefined);
	let fileInputEl = $state<HTMLInputElement | undefined>(undefined);
	let file = $state<File | null>(null);
	let fileError = $state('');
	let submitError = $state('');
	let submitting = $state(false);
	let uploadProgress = $state(0);
	let isDragOver = $state(false);

	let isOpen = $derived(modal.active === 'upload');
	let canUpload = $derived(!!file && !submitting);

	$effect(() => {
		if (!dialogEl) return;
		if (isOpen) {
			file = null;
			fileError = '';
			submitError = '';
			submitting = false;
			uploadProgress = 0;
			isDragOver = false;
			dialogEl.showModal();
		} else {
			dialogEl.close();
		}
	});

	function close() {
		modal.close();
	}

	function handleBackdropClick(e: MouseEvent) {
		if (e.target === dialogEl) close();
	}

	function getExtension(name: string): string {
		const idx = name.lastIndexOf('.');
		return idx >= 0 ? name.slice(idx).toLowerCase() : '';
	}

	function validateFile(candidate: File): string {
		const ext = getExtension(candidate.name);
		if (!ALLOWED_EXTENSIONS.includes(ext as (typeof ALLOWED_EXTENSIONS)[number])) {
			return `Unsupported type. Allowed: ${ALLOWED_EXTENSIONS.join(', ')}`;
		}
		return '';
	}

	function selectFile(candidate: File) {
		const validation = validateFile(candidate);
		fileError = validation;
		submitError = '';
		file = validation ? null : candidate;
		uploadProgress = 0;
	}

	function handleFileInput(e: Event) {
		const input = e.target as HTMLInputElement;
		const selected = input.files?.[0];
		if (selected) selectFile(selected);
	}

	function handleDrop(e: DragEvent) {
		e.preventDefault();
		isDragOver = false;
		const dropped = e.dataTransfer?.files?.[0];
		if (dropped) selectFile(dropped);
	}

	async function handleUpload() {
		if (!file || submitting) return;
		submitError = '';
		submitting = true;
		uploadProgress = 0;

		const result = await uploadLibraryFile(file, (progress) => {
			uploadProgress = progress.percent;
		});

		if (result.success) {
			uploadProgress = 100;
			close();
		} else {
			submitError = result.error;
		}

		submitting = false;
	}
</script>

<dialog
	bind:this={dialogEl}
	class="modal-backdrop"
	aria-label="Upload file"
	onclick={handleBackdropClick}
	onclose={close}
>
	<div class="cmd-card" role="document">
		<div class="cmd-body">
			<button
				type="button"
				class="drop-zone"
				class:drag-over={isDragOver}
				class:has-file={!!file}
				class:has-error={!!fileError}
				aria-label="Choose file"
				ondrop={handleDrop}
				ondragover={(e) => {
					e.preventDefault();
					isDragOver = true;
				}}
				ondragleave={() => {
					isDragOver = false;
				}}
				onclick={() => fileInputEl?.click()}
			>
				<input
					bind:this={fileInputEl}
					type="file"
					class="file-input-hidden"
					accept={ALLOWED_EXTENSIONS.join(',')}
					aria-hidden="true"
					tabindex="-1"
					onchange={handleFileInput}
				/>
				{#if file}
					<svg viewBox="0 0 24 24" aria-hidden="true">
						<path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z" />
						<polyline points="14 2 14 8 20 8" />
					</svg>
					<span class="dz-filename">{file.name}</span>
					<span class="dz-sub">{(file.size / 1024).toFixed(0)} KB</span>
				{:else}
					<svg viewBox="0 0 24 24" aria-hidden="true">
						<path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4" />
						<polyline points="17 8 12 3 7 8" />
						<line x1="12" y1="3" x2="12" y2="15" />
					</svg>
					<span class="dz-title">Drop file here</span>
					<span class="dz-sub">or choose from your computer</span>
					<span class="dz-formats">PDF, EPUB, HTML</span>
				{/if}
			</button>

			{#if fileError}
				<p class="error-text" role="alert">{fileError}</p>
			{/if}

			{#if submitting}
				<div
					class="progress-wrap"
					role="progressbar"
					aria-valuenow={uploadProgress}
					aria-valuemin={0}
					aria-valuemax={100}
				>
					<div class="progress-fill" style:width={uploadProgress + '%'}></div>
				</div>
			{/if}

			{#if submitError}
				<p class="error-text" role="alert">{submitError}</p>
			{/if}
		</div>

		<div class="cmd-controls">
			<button type="button" class="cmd-secondary" onclick={close}>Cancel</button>
			<button type="button" class="cmd-action" disabled={!canUpload} onclick={handleUpload}>
				{#if submitting}
					<span class="spinner" aria-hidden="true"></span>
					<span class="sr-only">Uploading</span>
				{:else}
					Upload
				{/if}
			</button>
		</div>
	</div>
</dialog>

<style>
	.modal-backdrop {
		position: fixed;
		inset: 0;
		width: 100%;
		height: 100%;
		max-width: 100%;
		max-height: 100%;
		margin: 0;
		padding: 80px 0 0;
		border: none;
		background: transparent;
		display: flex;
		align-items: flex-start;
		justify-content: center;
		box-sizing: border-box;
	}

	.modal-backdrop::backdrop {
		background: rgba(0, 0, 0, 0.4);
		backdrop-filter: blur(4px);
		-webkit-backdrop-filter: blur(4px);
	}

	.cmd-card {
		width: 440px;
		max-width: calc(100vw - 32px);
		background: var(--bg-elevated);
		border-radius: 14px;
		box-shadow:
			0 24px 80px rgba(0, 0, 0, 0.22),
			0 0 0 0.5px rgba(0, 0, 0, 0.06);
		overflow: hidden;
	}

	:global([data-theme='dark']) .cmd-card {
		box-shadow:
			0 24px 80px rgba(0, 0, 0, 0.55),
			0 0 0 0.5px rgba(255, 255, 255, 0.08);
	}

	.cmd-body {
		padding: 16px 16px 0;
	}

	.drop-zone {
		width: 100%;
		border-radius: 12px;
		border: 1.5px dashed var(--border-secondary);
		padding: 28px 16px;
		display: flex;
		flex-direction: column;
		align-items: center;
		gap: 4px;
		text-align: center;
		cursor: pointer;
		background: var(--bg-secondary);
		color: var(--text-primary);
		transition:
			border-color 0.15s ease,
			background 0.15s ease;
	}

	.drop-zone:hover,
	.drop-zone.drag-over {
		border-color: var(--accent);
		background: var(--fill-selected);
	}

	.drop-zone.has-file {
		border-style: solid;
		border-color: var(--accent);
		background: var(--fill-selected);
	}

	.drop-zone.has-error {
		border-color: var(--destructive);
	}

	.drop-zone svg {
		width: 24px;
		height: 24px;
		stroke: var(--text-tertiary);
		fill: none;
		stroke-width: 1.5;
		stroke-linecap: round;
		stroke-linejoin: round;
		margin-bottom: 2px;
	}

	.drop-zone.has-file svg {
		stroke: var(--accent);
	}

	.dz-title,
	.dz-filename {
		font-family: var(--font-sans);
		font-size: 13px;
		color: var(--text-primary);
	}

	.dz-filename {
		font-weight: 500;
		word-break: break-word;
	}

	.dz-sub {
		font-family: var(--font-sans);
		font-size: 12px;
		color: var(--text-secondary);
	}

	.dz-formats {
		font-family: var(--font-sans);
		font-size: 10px;
		color: var(--text-tertiary);
		margin-top: 1px;
	}

	.file-input-hidden {
		display: none;
	}

	.error-text {
		font-family: var(--font-sans);
		font-size: 12px;
		color: var(--destructive);
		margin: 6px 0 0;
	}

	.progress-wrap {
		width: 100%;
		height: 3px;
		background: var(--border-primary);
		border-radius: 2px;
		overflow: hidden;
		margin-top: 8px;
	}

	.progress-fill {
		height: 100%;
		background: var(--accent);
		border-radius: 2px;
		transition: width 0.1s ease;
	}

	.cmd-controls {
		display: flex;
		align-items: center;
		justify-content: flex-end;
		gap: 8px;
		padding: 12px 16px 16px;
	}

	.cmd-secondary,
	.cmd-action {
		border: none;
		font-family: var(--font-sans);
		font-size: 13px;
		font-weight: 600;
		cursor: pointer;
	}

	.cmd-secondary {
		padding: 6px 12px;
		border-radius: 8px;
		color: var(--text-secondary);
		background: var(--bg-secondary);
	}

	.cmd-secondary:hover {
		background: var(--bg-tertiary);
		color: var(--text-primary);
	}

	.cmd-action {
		padding: 6px 16px;
		border-radius: 980px;
		color: var(--text-on-color);
		background: var(--accent);
		display: flex;
		align-items: center;
		gap: 6px;
		transition: opacity 120ms ease;
	}

	.cmd-action:hover:not(:disabled) {
		opacity: 0.88;
	}

	.cmd-action:disabled {
		opacity: 0.32;
		cursor: not-allowed;
	}

	.spinner {
		display: inline-block;
		width: 12px;
		height: 12px;
		border: 2px solid rgba(255, 255, 255, 0.35);
		border-top-color: var(--text-on-color);
		border-radius: 50%;
		animation: spin 0.6s linear infinite;
	}

	@keyframes spin {
		to {
			transform: rotate(360deg);
		}
	}

	.sr-only {
		position: absolute;
		width: 1px;
		height: 1px;
		padding: 0;
		margin: -1px;
		overflow: hidden;
		clip: rect(0, 0, 0, 0);
		white-space: nowrap;
		border: 0;
	}
</style>
