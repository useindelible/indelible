<script lang="ts">
	import {
		formatDuplicateSavedDate,
		type DuplicateUrlInfo
	} from '$lib/components/library/save-url-model';

	interface Props {
		duplicate: DuplicateUrlInfo | null;
		submitError: string;
		submitting: boolean;
		onRefresh: () => void;
		onSaveAsNew: () => void;
	}

	let { duplicate, submitError, submitting, onRefresh, onSaveAsNew }: Props = $props();
</script>

<div class="cmd-body">
	{#if duplicate}
		<div class="dup-preview">
			<div class="dup-thumb">
				<svg viewBox="0 0 24 24" aria-hidden="true">
					<path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z" />
					<polyline points="14 2 14 8 20 8" />
					<line x1="16" y1="13" x2="8" y2="13" />
					<line x1="16" y1="17" x2="8" y2="17" />
				</svg>
			</div>
			<div class="dup-info">
				<span class="dup-title">{duplicate.title}</span>
				<span class="dup-meta">
					{duplicate.domain ?? ''}
					{#if duplicate.savedDate}
						&middot; Saved {formatDuplicateSavedDate(duplicate.savedDate)}
					{/if}
				</span>
				<span class="dup-status">Inbox</span>
			</div>
		</div>
		<div class="warning-banner">
			<svg viewBox="0 0 24 24" aria-hidden="true">
				<path
					d="M10.29 3.86L1.82 18a2 2 0 0 0 1.71 3h16.94a2 2 0 0 0 1.71-3L13.71 3.86a2 2 0 0 0-3.42 0z"
				/>
				<line x1="12" y1="9" x2="12" y2="13" />
				<line x1="12" y1="17" x2="12.01" y2="17" />
			</svg>
			<span class="wb-text">Already in your library</span>
			<div class="wb-actions">
				<button type="button" class="wb-btn ghost" onclick={onRefresh}>Refresh</button>
				<button type="button" class="wb-btn fill" onclick={onSaveAsNew} disabled={submitting}>
					Save as New
				</button>
			</div>
		</div>
	{:else if submitError}
		<p class="error-text">{submitError}</p>
	{/if}
</div>

<style>
	.cmd-body {
		padding: 0 16px 4px;
	}

	.dup-preview {
		display: flex;
		align-items: center;
		gap: 12px;
		padding: 12px;
		border-radius: 10px;
		background: var(--bg-secondary);
		margin-top: 8px;
	}

	.dup-thumb {
		width: 44px;
		height: 44px;
		border-radius: 10px;
		flex-shrink: 0;
		background: var(--fill-selected-strong);
		display: flex;
		align-items: center;
		justify-content: center;
	}

	.dup-thumb svg {
		width: 18px;
		height: 18px;
		stroke: var(--accent);
		fill: none;
		stroke-width: 1.5;
		stroke-linecap: round;
		stroke-linejoin: round;
	}

	.dup-info {
		flex: 1;
		display: flex;
		flex-direction: column;
		gap: 2px;
		min-width: 0;
	}

	.dup-title {
		font-size: 13px;
		font-weight: 600;
		letter-spacing: -0.01em;
		color: var(--text-primary);
		white-space: nowrap;
		overflow: hidden;
		text-overflow: ellipsis;
	}

	.dup-meta {
		font-size: 11px;
		color: var(--text-secondary);
	}

	.dup-status {
		display: inline-flex;
		padding: 1px 6px;
		border-radius: 4px;
		background: var(--fill-selected-strong);
		color: var(--accent);
		font-size: 10px;
		font-weight: 500;
		width: fit-content;
		margin-top: 1px;
	}

	.warning-banner {
		display: flex;
		align-items: center;
		gap: 8px;
		padding: 10px 12px;
		border-radius: 8px;
		margin-top: 6px;
		background: var(--fill-warning);
	}

	.warning-banner svg {
		width: 14px;
		height: 14px;
		flex-shrink: 0;
		fill: none;
		stroke: var(--warning);
		stroke-width: 1.5;
		stroke-linecap: round;
		stroke-linejoin: round;
	}

	.wb-text {
		flex: 1;
		font-size: 12px;
		font-family: var(--font-sans);
		color: var(--text-primary);
	}

	.wb-actions {
		display: flex;
		gap: 6px;
	}

	.wb-btn {
		padding: 4px 10px;
		border-radius: 6px;
		font-family: var(--font-sans);
		font-size: 11px;
		font-weight: 500;
		cursor: pointer;
		border: none;
	}

	.wb-btn.ghost {
		background: none;
		color: var(--accent);
	}

	.wb-btn.ghost:hover {
		background: var(--fill-selected);
	}

	.wb-btn.fill {
		background: var(--fill-hover);
		color: var(--text-primary);
	}

	.wb-btn.fill:hover {
		background: var(--bg-tertiary);
	}

	.wb-btn:disabled {
		opacity: 0.4;
		cursor: not-allowed;
	}

	.error-text {
		font-size: 12px;
		color: var(--destructive);
		margin: 8px 0 0;
		font-family: var(--font-sans);
	}
</style>
