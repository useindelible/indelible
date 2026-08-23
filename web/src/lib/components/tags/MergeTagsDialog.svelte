<script lang="ts">
	import type { TagResponse } from '$lib/api/generated/types.gen';
	import { sanitizeColor } from '$lib/utils/color';
	import { t } from '$lib/i18n';

	interface Props {
		sourceTags: TagResponse[];
		allTags: TagResponse[];
		onMerge: (sourceIds: string[], targetId: string) => void;
		onClose: () => void;
	}

	let { sourceTags, allTags, onMerge, onClose }: Props = $props();

	let targetId = $state<string>('');
	let merging = $state(false);

	const sourceIds = $derived(sourceTags.map((t) => t.id));
	const targetOptions = $derived(allTags.filter((t) => !sourceIds.includes(t.id)));
	const totalItems = $derived(
		sourceTags.reduce((sum, t) => sum + t.item_count + t.highlight_count, 0)
	);
	const canMerge = $derived(targetId !== '' && !merging);

	async function handleMerge() {
		if (!canMerge) return;
		merging = true;
		onMerge(sourceIds, targetId);
	}
</script>

<div
	class="cmd-backdrop"
	role="dialog"
	aria-modal="true"
	aria-label={$t('tag_merge')}
	tabindex="-1"
	onclick={onClose}
	onkeydown={(e) => {
		if (e.key === 'Escape') {
			e.preventDefault();
			onClose();
		}
	}}
>
	<div class="cmd-card" role="none" onclick={(e) => e.stopPropagation()} onkeydown={() => {}}>
		<div class="cmd-body">
			<div class="source-tags">
				<span class="field-label">{$t('tag_merging')}</span>
				<div class="tag-pills">
					{#each sourceTags as tag (tag.id)}
						<span class="tag-pill">
							<span
								class="pill-dot"
								style="background: {sanitizeColor(tag.color) ?? 'var(--text-tertiary)'}"
							></span>
							{tag.name}
						</span>
					{/each}
				</div>
			</div>

			<div class="warning-strip" role="alert">
				<svg viewBox="0 0 24 24" aria-hidden="true">
					<path
						d="M10.29 3.86L1.82 18a2 2 0 0 0 1.71 3h16.94a2 2 0 0 0 1.71-3L13.71 3.86a2 2 0 0 0-3.42 0z"
					/>
					<line x1="12" y1="9" x2="12" y2="13" />
					<line x1="12" y1="17" x2="12.01" y2="17" />
				</svg>
				{$t('tag_merge_warning', { values: { count: totalItems } })}
			</div>

			<label class="merge-field">
				<span class="field-label">{$t('tag_merge_into')}</span>
				<select class="cmd-select" bind:value={targetId}>
					<option value="" disabled>{$t('tag_select_target')}</option>
					{#each targetOptions as tag (tag.id)}
						<option value={tag.id}>{tag.name}</option>
					{/each}
				</select>
			</label>
		</div>

		<div class="cmd-controls">
			<button type="button" class="cmd-secondary" onclick={onClose}>{$t('common_cancel')}</button>
			<button
				type="button"
				class="cmd-action cmd-action-danger"
				disabled={!canMerge}
				onclick={handleMerge}
			>
				{merging ? $t('tag_merging_progress') : $t('tag_merge')}
			</button>
		</div>
	</div>
</div>

<style>
	.cmd-backdrop {
		position: fixed;
		inset: 0;
		background: rgba(0, 0, 0, 0.4);
		backdrop-filter: blur(4px);
		-webkit-backdrop-filter: blur(4px);
		display: flex;
		align-items: flex-start;
		justify-content: center;
		padding-top: 80px;
		z-index: 300;
		box-sizing: border-box;
	}

	:global([data-theme='dark']) .cmd-backdrop {
		background: rgba(0, 0, 0, 0.6);
	}

	.cmd-card {
		width: 460px;
		max-width: calc(100vw - 32px);
		background: var(--bg-elevated);
		border-radius: 14px;
		box-shadow:
			0 24px 80px rgba(0, 0, 0, 0.22),
			0 0 0 0.5px rgba(0, 0, 0, 0.06);
	}

	:global([data-theme='dark']) .cmd-card {
		box-shadow:
			0 24px 80px rgba(0, 0, 0, 0.55),
			0 0 0 0.5px rgba(255, 255, 255, 0.08);
	}

	.cmd-body {
		padding: 16px 16px 4px;
		display: flex;
		flex-direction: column;
		gap: 12px;
	}

	.source-tags {
		display: flex;
		flex-direction: column;
		gap: 8px;
	}

	.field-label {
		font-family: var(--font-sans);
		font-size: 11px;
		font-weight: 600;
		letter-spacing: 0.04em;
		text-transform: uppercase;
		color: var(--text-tertiary);
	}

	.tag-pills {
		display: flex;
		flex-wrap: wrap;
		gap: 6px;
	}

	.tag-pill {
		display: inline-flex;
		align-items: center;
		gap: 6px;
		padding: 4px 10px;
		border-radius: 7px;
		background: var(--fill-selected);
		color: var(--accent);
		font-family: var(--font-sans);
		font-size: 13px;
		font-weight: 500;
	}

	.pill-dot {
		width: 8px;
		height: 8px;
		border-radius: 50%;
		flex-shrink: 0;
	}

	.warning-strip {
		display: flex;
		align-items: flex-start;
		gap: 8px;
		padding: 10px 12px;
		border-radius: 10px;
		background: var(--fill-warning);
		color: var(--warning);
		font-family: var(--font-sans);
		font-size: 13px;
		font-weight: 500;
		line-height: 1.4;
	}

	.warning-strip svg {
		width: 14px;
		height: 14px;
		stroke: var(--warning);
		fill: none;
		stroke-width: 2;
		stroke-linecap: round;
		stroke-linejoin: round;
		flex-shrink: 0;
		margin-top: 1px;
	}

	.merge-field {
		display: flex;
		flex-direction: column;
		gap: 6px;
	}

	.cmd-select {
		width: 100%;
		height: 40px;
		padding: 0 28px 0 12px;
		border-radius: 10px;
		border: none;
		background: var(--bg-secondary);
		font-family: var(--font-sans);
		font-size: 14px;
		color: var(--text-primary);
		cursor: pointer;
		appearance: none;
		background-image: url("data:image/svg+xml,%3Csvg width='10' height='6' viewBox='0 0 10 6' fill='none' xmlns='http://www.w3.org/2000/svg'%3E%3Cpath d='M1 1l4 4 4-4' stroke='%2386868B' stroke-width='1.5' stroke-linecap='round' stroke-linejoin='round'/%3E%3C/svg%3E");
		background-repeat: no-repeat;
		background-position: right 10px center;
		outline: none;
		box-sizing: border-box;
		letter-spacing: -0.01em;
	}

	.cmd-select:focus {
		box-shadow: 0 0 0 3px var(--fill-selected);
	}

	.cmd-controls {
		display: flex;
		align-items: center;
		gap: 6px;
		padding: 10px 16px 14px;
	}

	.cmd-secondary {
		padding: 6px 14px;
		border-radius: 980px;
		border: 1px solid var(--border-primary);
		background: transparent;
		font-family: var(--font-sans);
		font-size: 13px;
		font-weight: 500;
		color: var(--text-secondary);
		cursor: pointer;
		transition: background 120ms ease;
		letter-spacing: -0.01em;
	}

	.cmd-secondary:hover {
		background: var(--fill-hover);
	}

	.cmd-action {
		margin-left: auto;
		padding: 6px 16px;
		border-radius: 980px;
		border: none;
		font-family: var(--font-sans);
		font-size: 13px;
		font-weight: 600;
		cursor: pointer;
		letter-spacing: -0.01em;
		color: var(--text-on-color);
		background: var(--accent);
		flex-shrink: 0;
		transition: opacity 120ms ease;
	}

	.cmd-action:hover:not(:disabled) {
		opacity: 0.88;
	}

	.cmd-action:disabled {
		opacity: 0.32;
		cursor: not-allowed;
	}

	.cmd-action-danger {
		background: var(--destructive);
	}
</style>
