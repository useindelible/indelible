<script lang="ts">
	import type { ImportJobStatusResponse } from '$lib/api';
	import SettingsGroup from '$lib/components/settings/SettingsGroup.svelte';
	import { normalizeImportStatus } from '$lib/integrations/status';
	import { progressPercent, statusForJob, type ImportSlot } from '../integrations-hub-model';
	import { t } from '$lib/i18n';

	interface Props {
		activeJob: ImportJobStatusResponse | null;
		activeSlot: ImportSlot | null;
		busySlot: ImportSlot | null;
		uploadError: string | null;
		uploadErrorSlot: ImportSlot | null;
		pollError: string | null;
		rollbackNotice: string | null;
		dropHover: ImportSlot | null;
		readwiseUploadLimit: string;
		activeIsTerminal: boolean;
		onDropHover: (slot: ImportSlot | null) => void;
		onFileChange: (event: Event) => void;
		onDrop: (event: DragEvent) => void;
		onOpenRollback: (jobId: string) => void;
		onClearActiveJob: () => void;
	}

	let {
		activeJob,
		activeSlot,
		busySlot,
		uploadError,
		uploadErrorSlot,
		pollError,
		rollbackNotice,
		dropHover,
		readwiseUploadLimit,
		activeIsTerminal,
		onDropHover,
		onFileChange,
		onDrop,
		onOpenRollback,
		onClearActiveJob
	}: Props = $props();
</script>

<SettingsGroup
	title={$t('integrations_hub_one_time_imports')}
	meta={$t('integrations_hub_one_time_imports_hint')}
>
	<div class="imports-shelf">
		<div class="import-card">
			<div class="import-head">
				<div class="conn-mark readwise">R</div>
				<div class="conn-meta">
					<div class="conn-name">Readwise Reader</div>
					<div class="conn-tagline">{$t('integrations_hub_readwise_formats')}</div>
				</div>
			</div>

			{#if activeSlot === 'readwise' && activeJob}
				{@const status = statusForJob(activeJob)}
				<div class="job-strip" data-variant={status.variant}>
					<div class="job-strip-head">
						<span class="status-pill {status.variant}">{$t(status.labelKey)}</span>
						<span class="job-strip-counts">
							{$t('integrations_hub_import_counts', {
								values: {
									imported: activeJob.counts.imported,
									duplicate: activeJob.counts.duplicate,
									failed: activeJob.counts.failed
								}
							})}
						</span>
					</div>
					{#if !activeIsTerminal}
						{@const progress = progressPercent(activeJob)}
						<div class="job-progress" class:indeterminate={progress === null}>
							{#if progress === null}
								<div class="job-progress-fill indeterminate"></div>
							{:else}
								<div class="job-progress-fill" style:width="{progress}%"></div>
							{/if}
						</div>
					{/if}
					<div class="job-strip-actions">
						{#if activeIsTerminal}
							<button
								type="button"
								class="btn ghost compact"
								onclick={() => onOpenRollback(activeJob.id)}
								disabled={normalizeImportStatus(activeJob.status) === 'rolled_back'}
							>
								{$t('integrations_hub_roll_back')}
							</button>
							<button type="button" class="btn ghost compact" onclick={onClearActiveJob}
								>{$t('reader_done')}</button
							>
						{:else}
							<span class="job-strip-hint">{$t('integrations_hub_polling')}</span>
						{/if}
					</div>
				</div>
			{:else}
				<label
					class="import-drop"
					class:drop-hover={dropHover === 'readwise'}
					ondragover={(event) => {
						event.preventDefault();
						onDropHover('readwise');
					}}
					ondragleave={() => onDropHover(null)}
					ondrop={onDrop}
				>
					<div class="drop-icon" aria-hidden="true">
						<svg viewBox="0 0 24 24">
							<path d="M12 3v12" />
							<path d="M7 8l5-5 5 5" />
							<path d="M5 15v4h14v-4" />
						</svg>
					</div>
					<div class="drop-title">
						{busySlot === 'readwise'
							? $t('library_upload_uploading')
							: $t('integrations_hub_readwise_drop')}
					</div>
					<div class="drop-hint">{$t('integrations_hub_readwise_match_hint')}</div>
					<input
						type="file"
						accept=".csv,.zip,.opml,.xml"
						hidden
						multiple
						disabled={busySlot !== null}
						onchange={onFileChange}
					/>
				</label>
			{/if}

			{#if uploadErrorSlot === 'readwise' && uploadError}
				<p class="zone-meta error" role="alert">{uploadError}</p>
			{/if}
			<div class="import-meta-row">
				<span class="accept">.csv .zip .opml</span>
				<span>{readwiseUploadLimit}</span>
			</div>
		</div>
	</div>

	{#if pollError}
		<p class="zone-meta error" role="alert">{pollError}</p>
	{/if}
	{#if rollbackNotice}
		<p class="zone-meta success" role="status">{rollbackNotice}</p>
	{/if}
</SettingsGroup>

<style>
	.imports-shelf {
		display: grid;
		grid-template-columns: repeat(3, 1fr);
		gap: 14px;
	}

	.import-card {
		background: var(--bg-elevated);
		border-radius: 14px;
		padding: 18px;
		box-shadow: var(--int-shadow-card);
		display: flex;
		flex-direction: column;
		gap: 12px;
		transition:
			box-shadow 200ms,
			transform 200ms;
	}

	.import-card:hover {
		box-shadow: var(--int-shadow-card-hover);
		transform: translateY(-1px);
	}

	.import-head {
		display: flex;
		align-items: flex-start;
		gap: 12px;
	}

	.conn-mark {
		width: 40px;
		height: 40px;
		border-radius: 10px;
		flex-shrink: 0;
		display: flex;
		align-items: center;
		justify-content: center;
		background: var(--bg-secondary);
		color: var(--text-secondary);
		box-shadow: inset 0 0 0 0.5px var(--border-primary);
		font-weight: 700;
		font-size: 16px;
	}

	.conn-meta {
		flex: 1;
		min-width: 0;
	}

	.conn-name {
		font-size: 14px;
		font-weight: 600;
		letter-spacing: 0;
		color: var(--text-primary);
		margin-bottom: 3px;
	}

	.conn-tagline {
		font-size: 12px;
		color: var(--text-secondary);
		line-height: 1.4;
	}

	.import-drop {
		border: 1.5px dashed var(--input-border);
		border-radius: 10px;
		padding: 14px 12px;
		text-align: center;
		background: var(--card-bg);
		cursor: pointer;
		display: block;
		transition:
			border-color 160ms,
			background 160ms,
			color 160ms;
	}

	.import-drop:hover,
	.import-drop.drop-hover {
		border-color: var(--int-ring-connected);
		background: var(--int-status-coming-bg);
	}

	.drop-icon {
		width: 26px;
		height: 26px;
		margin: 0 auto 6px;
		color: var(--text-tertiary);
	}

	.import-drop:hover .drop-icon,
	.import-drop.drop-hover .drop-icon {
		color: var(--int-ring-connected);
	}

	.drop-icon svg {
		width: 100%;
		height: 100%;
		stroke: currentColor;
		fill: none;
		stroke-width: 1.5;
		stroke-linecap: round;
		stroke-linejoin: round;
	}

	.drop-title {
		font-size: 12px;
		font-weight: 500;
		color: var(--text-primary);
		letter-spacing: -0.005em;
	}

	.drop-hint,
	.import-meta-row,
	.zone-meta {
		font-size: 11px;
		color: var(--text-tertiary);
	}

	.import-meta-row {
		display: flex;
		justify-content: space-between;
		align-items: center;
		letter-spacing: -0.005em;
		padding-top: 4px;
		border-top: 0.5px solid var(--border-hairline);
	}

	.accept {
		font-family: 'SF Mono', 'Fira Code', 'Menlo', ui-monospace, monospace;
		font-size: 10.5px;
		color: var(--text-secondary);
	}

	.zone-meta {
		font-family: var(--font-sans);
		margin: 0;
	}

	.zone-meta.error {
		color: var(--destructive);
	}

	.zone-meta.success {
		color: var(--success);
	}

	.job-strip {
		border: 1px solid var(--border-primary);
		border-radius: 10px;
		padding: 12px;
		background: var(--card-bg);
		display: flex;
		flex-direction: column;
		gap: 8px;
	}

	.job-strip-head,
	.job-strip-actions {
		display: flex;
		align-items: center;
		gap: 8px;
	}

	.job-strip-head {
		justify-content: space-between;
	}

	.job-strip-actions {
		justify-content: flex-end;
	}

	.job-strip-counts,
	.job-strip-hint {
		font-size: 11px;
		color: var(--text-secondary);
		font-variant-numeric: tabular-nums;
	}

	.job-strip-hint {
		color: var(--text-tertiary);
	}

	.status-pill {
		display: inline-flex;
		align-items: center;
		padding: 2px 8px;
		border-radius: 980px;
		font-size: 10.5px;
		font-weight: 600;
		letter-spacing: 0;
	}

	.status-pill.active {
		background: var(--int-status-active-bg);
		color: var(--int-status-active-text);
	}

	.status-pill.syncing {
		background: var(--int-status-syncing-bg);
		color: var(--int-status-syncing-text);
	}

	.status-pill.attention {
		background: var(--int-status-attention-bg);
		color: var(--int-status-attention-text);
	}

	.status-pill.coming {
		background: var(--int-status-coming-bg);
		color: var(--int-status-coming-text);
	}

	.job-progress {
		height: 4px;
		border-radius: 2px;
		background: var(--int-ring-track);
		overflow: hidden;
	}

	.job-progress-fill {
		height: 100%;
		background: var(--int-ring-connected);
		transition: width 200ms ease;
	}

	.job-progress-fill.indeterminate {
		width: 35%;
		animation: job-progress-slide 1400ms cubic-bezier(0.4, 0, 0.2, 1) infinite;
	}

	@keyframes job-progress-slide {
		to {
			transform: translateX(370%);
		}
	}

	.btn {
		border: none;
		border-radius: 8px;
		padding: 6px 12px;
		font: inherit;
		font-size: 12.5px;
		font-weight: 500;
		letter-spacing: 0;
		cursor: pointer;
	}

	.btn.ghost {
		background: transparent;
		color: var(--text-primary);
		box-shadow: inset 0 0 0 0.5px var(--border-primary);
	}

	.btn.compact {
		padding: 5px 10px;
		font-size: 11.5px;
	}

	.btn:disabled {
		opacity: 0.45;
		cursor: default;
	}

	@media (max-width: 899px) {
		.imports-shelf {
			grid-template-columns: 1fr;
		}
	}
</style>
