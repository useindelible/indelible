<script lang="ts">
	import * as apiSdk from '$lib/api';
	import CollectionPicker from './CollectionPicker.svelte';
	import TagInput from './TagInput.svelte';

	interface Props {
		onSuccess: () => void;
	}

	let { onSuccess }: Props = $props();

	let url = $state('');
	let urlError = $state('');
	let tags = $state<string[]>([]);
	let collectionId = $state<string | null>(null);
	let submitting = $state(false);
	let submitError = $state('');

	// The 409 response body is typed as `unknown` in CreateItemErrors. The backend
	// returns the existing DocumentListEntry when a duplicate is detected so we can show a
	// preview card, but we cast conservatively and only display what we can confirm.
	let duplicate = $state<{ id: string; title: string; domain: string | null } | null>(null);

	function validateUrl(value: string): string {
		if (!value.trim()) return 'Please enter a URL';
		try {
			const parsed = new URL(value.trim());
			if (parsed.protocol !== 'http:' && parsed.protocol !== 'https:') {
				return 'URL must start with http:// or https://';
			}
		} catch {
			return 'Please enter a valid URL';
		}
		return '';
	}

	async function handleSubmit() {
		duplicate = null;
		submitError = '';

		const validationError = validateUrl(url);
		if (validationError) {
			urlError = validationError;
			return;
		}
		urlError = '';

		submitting = true;
		try {
			const { data, error, response } = await apiSdk.createDocumentEntry({
				body: { url: url.trim() }
			});

			if (data) {
				onSuccess();
				return;
			}

			if (response.status === 409) {
				// The 409 body shape is unknown in the generated types. Cast to a minimal
				// shape that matches what the backend actually returns (DocumentListEntry).
				const body = error as Record<string, unknown> | null | undefined;
				if (body && typeof body['id'] === 'string') {
					duplicate = {
						id: body['id'] as string,
						title: typeof body['title'] === 'string' ? (body['title'] as string) : 'Already saved',
						domain: typeof body['domain'] === 'string' ? (body['domain'] as string) : null
					};
				} else {
					submitError = 'This URL is already in your library.';
				}
				return;
			}

			const problem = error as Record<string, unknown> | null | undefined;
			submitError =
				(typeof problem?.['detail'] === 'string' ? problem['detail'] : undefined) ??
				(typeof problem?.['message'] === 'string' ? problem['message'] : undefined) ??
				'Failed to save. Please try again.';
		} catch {
			submitError = 'An unexpected error occurred.';
		} finally {
			submitting = false;
		}
	}
</script>

<form
	onsubmit={(e) => {
		e.preventDefault();
		handleSubmit();
	}}
>
	<div class="url-input-wrapper" class:error={!!urlError}>
		<input
			type="url"
			class="url-input"
			bind:value={url}
			placeholder="https://..."
			aria-label="URL"
			aria-describedby={urlError ? 'url-error' : undefined}
			oninput={() => {
				if (urlError) urlError = validateUrl(url);
			}}
		/>
	</div>
	{#if urlError}
		<p class="field-error" id="url-error" role="alert">{urlError}</p>
	{/if}

	{#if duplicate}
		<div class="duplicate-card" role="alert">
			<div class="duplicate-info">
				<p class="duplicate-title">{duplicate.title}</p>
				{#if duplicate.domain}
					<p class="duplicate-source">{duplicate.domain}</p>
				{/if}
				<span class="duplicate-badge">Already saved</span>
			</div>
		</div>
	{/if}

	<div class="field-row">
		<p class="field-label">Collection</p>
		<CollectionPicker bind:value={collectionId} />
	</div>

	<div class="field-row">
		<p class="field-label">Tags</p>
		<TagInput bind:tags />
	</div>

	{#if submitError}
		<p class="submit-error" role="alert">{submitError}</p>
	{/if}

	<button type="submit" class="save-btn" disabled={submitting} aria-busy={submitting}>
		{#if submitting}
			<span class="spinner" aria-hidden="true"></span>
			<span class="sr-only">Saving...</span>
		{:else}
			Save to Library
		{/if}
	</button>
</form>

<style>
	.url-input-wrapper {
		width: 100%;
		height: 48px;
		border-radius: 12px;
		background: var(--input-bg, var(--bg-secondary));
		border: 1px solid var(--input-border, var(--border-primary));
		padding: 0 16px;
		display: flex;
		align-items: center;
		margin-bottom: 8px;
		transition:
			border-color 0.15s ease,
			box-shadow 0.15s ease;
	}

	.url-input-wrapper:focus-within {
		border-color: var(--accent);
		box-shadow: 0 0 0 3px var(--fill-selected);
	}

	.url-input-wrapper.error {
		border-color: var(--destructive);
	}

	.url-input {
		flex: 1;
		border: none;
		background: transparent;
		outline: none;
		font-family: var(--font-sans);
		font-size: 15px;
		font-weight: 400;
		letter-spacing: -0.01em;
		color: var(--text-primary);
	}

	.url-input::placeholder {
		color: var(--text-tertiary);
	}

	.field-error {
		font-size: 12px;
		color: var(--destructive);
		margin: 0 0 12px;
		font-family: var(--font-sans);
	}

	.duplicate-card {
		border-radius: 12px;
		border: 1px solid var(--border-primary);
		background: var(--preview-bg, var(--bg-secondary));
		padding: 14px;
		margin-bottom: 16px;
	}

	.duplicate-info {
		display: flex;
		flex-direction: column;
		gap: 4px;
	}

	.duplicate-title {
		font-size: 14px;
		font-weight: 600;
		letter-spacing: -0.01em;
		color: var(--text-primary);
		margin: 0;
	}

	.duplicate-source {
		font-size: 12px;
		color: var(--text-secondary);
		margin: 0;
	}

	.duplicate-badge {
		display: inline-flex;
		align-items: center;
		padding: 2px 8px;
		border-radius: 5px;
		font-size: 11px;
		font-weight: 500;
		background: var(--tag-bg);
		color: var(--tag-text);
		align-self: flex-start;
	}

	.field-row {
		margin-bottom: 14px;
	}

	.field-label {
		display: block;
		font-size: 12px;
		font-weight: 500;
		letter-spacing: 0.01em;
		color: var(--text-tertiary);
		text-transform: uppercase;
		margin-bottom: 6px;
		font-family: var(--font-sans);
	}

	.submit-error {
		font-size: 13px;
		color: var(--destructive);
		margin: 0 0 12px;
		font-family: var(--font-sans);
	}

	.save-btn {
		display: flex;
		align-items: center;
		justify-content: center;
		gap: 8px;
		width: 100%;
		height: 44px;
		border-radius: 10px;
		border: none;
		background: var(--accent);
		color: var(--text-on-color);
		font-family: var(--font-sans);
		font-size: 15px;
		font-weight: 500;
		letter-spacing: -0.01em;
		cursor: pointer;
		transition:
			background 120ms ease,
			opacity 120ms ease;
		margin-top: 8px;
	}

	.save-btn:hover:not(:disabled) {
		background: var(--accent-hover, var(--accent));
		opacity: 0.9;
	}

	.save-btn:disabled {
		opacity: 0.6;
		cursor: not-allowed;
	}

	.spinner {
		display: inline-block;
		width: 16px;
		height: 16px;
		border: 2px solid rgba(255, 255, 255, 0.35);
		border-top-color: var(--text-on-color);
		border-radius: 50%;
		animation: spin 0.6s linear infinite;
		flex-shrink: 0;
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

	@keyframes spin {
		to {
			transform: rotate(360deg);
		}
	}
</style>
