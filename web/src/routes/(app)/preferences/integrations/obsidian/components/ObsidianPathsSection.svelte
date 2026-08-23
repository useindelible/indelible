<script lang="ts">
	import type { ObsidianSettingsDto } from '$lib/api';
	import { t, type MessageKey } from '$lib/i18n';

	interface Props {
		settings: ObsidianSettingsDto;
		onChange: (patch: Partial<ObsidianSettingsDto>) => void;
		onFolderTemplateChange: (key: string, value: string) => void;
	}

	let { settings, onChange, onFolderTemplateChange }: Props = $props();

	const folders = [
		{ key: 'books', labelKey: 'integrations_obsidian_books', fallback: 'books' },
		{ key: 'articles', labelKey: 'integrations_obsidian_articles', fallback: 'articles' },
		{ key: 'tweets', labelKey: 'integrations_obsidian_tweets', fallback: 'tweets' }
	] satisfies { key: string; labelKey: MessageKey; fallback: string }[];
</script>

<section class="section">
	<div class="section-head">
		<h2 class="section-title">{$t('integrations_obsidian_paths')}</h2>
		<p class="section-sub">{$t('integrations_obsidian_paths_hint')}</p>
	</div>
	<div class="card card-stack">
		<div class="row-block">
			<div class="row-block-head">
				<p class="row-title">{$t('integrations_obsidian_file_name_template')}</p>
				<span class="row-tag">.md</span>
			</div>
			<p class="row-sub-inline">
				{$t('integrations_obsidian_file_name_template_hint')}
			</p>
			<input
				class="input-base"
				type="text"
				placeholder={'{{title}}'}
				value={settings.file_name_template ?? ''}
				oninput={(event) => onChange({ file_name_template: event.currentTarget.value })}
			/>
		</div>

		<div class="row-block">
			<div class="row-block-head">
				<p class="row-title">{$t('integrations_obsidian_category_folders')}</p>
				<span class="row-count"
					>{$t('integrations_obsidian_folder_group_count', { values: { count: 3 } })}</span
				>
			</div>
			<p class="row-sub-inline">
				{$t('integrations_obsidian_category_folders_hint')}
			</p>
		</div>

		<div class="path-grid">
			{#each folders as folder (folder.key)}
				<div class="path-field">
					<label class="path-label" for={`path-${folder.key}`}>
						{$t(folder.labelKey)} <span class="arr">-&gt;</span>
					</label>
					<input
						id={`path-${folder.key}`}
						class="input-base"
						type="text"
						value={settings.category_folder_templates[folder.key] ?? folder.fallback}
						oninput={(event) => onFolderTemplateChange(folder.key, event.currentTarget.value)}
					/>
				</div>
			{/each}
		</div>
	</div>
</section>

<style>
	.section {
		margin-top: 28px;
	}
	.section-head {
		display: flex;
		align-items: baseline;
		justify-content: space-between;
		margin: 0 4px 12px;
		gap: 12px;
	}
	.section-title {
		font-size: 12px;
		font-weight: 550;
		color: var(--text-tertiary);
		text-transform: uppercase;
		letter-spacing: 0;
		margin: 0;
	}
	.section-sub,
	.row-sub-inline {
		font-size: 12.5px;
		color: var(--text-tertiary);
		margin: 0;
	}
	.card {
		background: var(--bg-elevated);
		border: 1px solid var(--border-hairline);
		border-radius: 14px;
		box-shadow: var(--shadow-1);
		overflow: hidden;
	}
	.card-stack > * + * {
		border-top: 1px solid var(--border-hairline);
	}
	.row-block {
		padding: 18px 22px;
	}
	.row-block-head {
		display: flex;
		align-items: baseline;
		justify-content: space-between;
		gap: 12px;
		margin-bottom: 8px;
	}
	.row-title {
		font-size: 14px;
		font-weight: 500;
		margin: 0;
		color: var(--text-primary);
	}
	.row-sub-inline {
		margin-bottom: 10px;
		line-height: 1.45;
	}
	.row-tag,
	.row-count {
		font-size: 11.5px;
		color: var(--text-tertiary);
	}
	.row-tag {
		font-family: var(--font-mono, ui-monospace, monospace);
	}
	.input-base {
		width: 100%;
		font-family: var(--font-mono, ui-monospace, monospace);
		font-size: 12.5px;
		background: var(--obs-editor-bg);
		border: 1px solid var(--border-hairline);
		border-radius: 8px;
		padding: 9px 12px;
		color: var(--text-primary);
	}
	.input-base:focus {
		outline: 0;
		border-color: var(--obs-accent);
		box-shadow: 0 0 0 3px color-mix(in oklab, var(--obs-accent) 28%, transparent);
		background: var(--bg-elevated);
	}
	.path-grid {
		display: grid;
		grid-template-columns: repeat(2, minmax(0, 1fr));
		gap: 10px 14px;
		padding: 14px 22px 18px;
		border-top: 1px solid var(--border-hairline);
	}
	.path-field {
		display: flex;
		flex-direction: column;
		gap: 5px;
	}
	.path-label {
		font-size: 12px;
		color: var(--text-tertiary);
		font-weight: 500;
		display: inline-flex;
		align-items: center;
		gap: 6px;
	}
	.arr {
		color: var(--text-quaternary);
		font-family: var(--font-mono, ui-monospace, monospace);
	}
	@media (max-width: 980px) {
		.path-grid {
			grid-template-columns: 1fr;
		}
	}
</style>
