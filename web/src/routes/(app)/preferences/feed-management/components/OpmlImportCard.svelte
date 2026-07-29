<script lang="ts">
	import SettingsGroup from '$lib/components/settings/SettingsGroup.svelte';
	import type { OpmlImportResponse } from '$lib/api';

	interface Props {
		uploading: boolean;
		result: OpmlImportResponse | null;
		error: string | null;
		onUpload: (file: File) => void;
	}

	let { uploading, result, error, onUpload }: Props = $props();
	let inputEl = $state<HTMLInputElement | undefined>(undefined);
	let isDragOver = $state(false);

	function uploadFirstFile(fileList: FileList | null | undefined) {
		const file = fileList?.[0];
		if (file) onUpload(file);
	}

	function handleDrop(event: DragEvent) {
		event.preventDefault();
		isDragOver = false;
		uploadFirstFile(event.dataTransfer?.files);
	}

	function handleFileSelect(event: Event) {
		const input = event.target as HTMLInputElement;
		uploadFirstFile(input.files);
		input.value = '';
	}
</script>

<SettingsGroup title="Import OPML" meta="From Feedly, Inoreader, NetNewsWire, etc.">
	<div class="opml-block" id="opml-import">
		<div
			class="opml-drop"
			class:drag-over={isDragOver}
			role="button"
			tabindex="0"
			aria-label="Drop OPML file to import feeds"
			ondrop={handleDrop}
			ondragover={(event) => {
				event.preventDefault();
				isDragOver = true;
			}}
			ondragleave={() => {
				isDragOver = false;
			}}
			onclick={() => inputEl?.click()}
			onkeydown={(event) => {
				if (event.key === 'Enter' || event.key === ' ') {
					event.preventDefault();
					inputEl?.click();
				}
			}}
		>
			<input
				bind:this={inputEl}
				class="opml-input"
				type="file"
				accept=".opml,.xml"
				aria-hidden="true"
				tabindex="-1"
				onchange={handleFileSelect}
			/>
			<div class="drop-icon">
				{#if uploading}
					<span class="spinner" aria-label="Uploading"></span>
				{:else}
					<svg viewBox="0 0 24 24" aria-hidden="true">
						<path d="M12 3v12" />
						<path d="M7 8l5-5 5 5" />
						<path d="M5 21h14" />
					</svg>
				{/if}
			</div>
			<div class="drop-title">{uploading ? 'Uploading…' : 'Drop your OPML file'}</div>
			<div class="drop-hint">
				We'll match feeds against your existing subscriptions and add what's new.
			</div>
			<span class="drop-fallback">or click to choose a file</span>
		</div>
	</div>

	{#if error}
		<p class="opml-error">{error}</p>
	{/if}

	{#if result}
		<div class="opml-result">
			<p class="opml-summary">
				Imported {result.created} feed{result.created === 1 ? '' : 's'}, skipped {result.skipped}
			</p>
			{#if result.errors.length > 0}
				<details class="opml-errors">
					<summary>
						{result.errors.length} error{result.errors.length === 1 ? '' : 's'}
					</summary>
					<ul>
						{#each result.errors as item, index (index)}
							<li>{item}</li>
						{/each}
					</ul>
				</details>
			{/if}
		</div>
	{/if}
</SettingsGroup>

<style>
	.opml-block {
		background: var(--bg-elevated);
		border-radius: 14px;
		padding: 20px 22px;
		box-shadow: var(--feed-card-shadow);
	}

	.opml-drop {
		border-radius: 12px;
		border: 1.5px dashed var(--border-primary);
		background: var(--feed-amber-soft);
		padding: 28px 22px;
		text-align: center;
		cursor: pointer;
		display: flex;
		flex-direction: column;
		align-items: center;
		gap: 8px;
		transition:
			border-color 160ms,
			background 160ms,
			transform 160ms;
	}

	.opml-drop:hover,
	.opml-drop.drag-over {
		border-color: var(--feed-amber);
		background: var(--feed-chip-active-bg);
	}

	.opml-drop.drag-over {
		transform: scale(1.005);
	}

	.opml-input {
		display: none;
	}

	.drop-icon {
		width: 40px;
		height: 40px;
		border-radius: 10px;
		background: var(--feed-amber);
		color: var(--text-on-color);
		display: flex;
		align-items: center;
		justify-content: center;
		box-shadow: var(--feed-amber-shadow);
	}

	.drop-icon svg {
		width: 18px;
		height: 18px;
		stroke: currentColor;
		fill: none;
		stroke-width: 1.9;
		stroke-linecap: round;
		stroke-linejoin: round;
	}

	.spinner {
		width: 16px;
		height: 16px;
		border-radius: 50%;
		border: 1.8px solid var(--feed-metric-card-border);
		border-top-color: var(--text-on-color);
		animation: spin 700ms linear infinite;
	}

	@keyframes spin {
		to {
			transform: rotate(360deg);
		}
	}

	.drop-title {
		font-size: 14px;
		font-weight: 600;
		color: var(--text-primary);
		letter-spacing: 0;
		margin-top: 2px;
		font-family: var(--font-sans);
	}

	.drop-hint {
		font-size: 12px;
		color: var(--text-secondary);
		letter-spacing: 0;
		font-family: var(--font-sans);
	}

	.drop-fallback {
		margin-top: 4px;
		font-size: 12px;
		color: var(--feed-amber-strong);
		font-weight: 600;
		text-decoration: underline;
		text-underline-offset: 2px;
		font-family: var(--font-sans);
	}

	.opml-error {
		font-size: 13px;
		color: var(--destructive);
		font-family: var(--font-sans);
		margin: 8px 0 0;
		line-height: 1.4;
	}

	.opml-result {
		margin-top: 12px;
		padding: 14px 16px;
		border-radius: 10px;
		background: var(--bg-secondary);
		box-shadow: inset 0 0 0 0.5px var(--border-primary);
	}

	.opml-summary {
		font-size: 14px;
		font-weight: 500;
		color: var(--text-primary);
		font-family: var(--font-sans);
		margin: 0;
	}

	.opml-errors {
		margin-top: 8px;
		font-size: 12px;
		color: var(--text-secondary);
		font-family: var(--font-sans);
	}

	.opml-errors summary {
		cursor: pointer;
		color: var(--destructive);
		font-weight: 500;
	}

	.opml-errors ul {
		margin: 4px 0 0;
		padding-left: 18px;
	}

	.opml-errors li {
		line-height: 1.5;
		color: var(--text-secondary);
	}
</style>
