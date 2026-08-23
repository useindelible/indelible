<script lang="ts">
	import SettingsGroup from '$lib/components/settings/SettingsGroup.svelte';
	import { t } from '$lib/i18n';
	import type { ReaderOpenModeDto } from '$lib/api';

	interface Props {
		emailOpenMode: ReaderOpenModeDto;
		onEmailOpenModeChange: (value: ReaderOpenModeDto) => void;
	}

	let { emailOpenMode, onEmailOpenModeChange }: Props = $props();
</script>

<SettingsGroup
	title={$t('prefs_reading_content_defaults')}
	meta={$t('prefs_reading_content_defaults_meta')}
>
	<div class="group-card">
		<div class="row">
			<div class="label-block">
				<div class="label">{$t('library_filter_value_article')}</div>
				<div class="hint">{$t('prefs_reading_article_open_hint')}</div>
			</div>
			<div class="content-type-static">{$t('prefs_reading_reader')}</div>
		</div>
		<div class="row">
			<div class="label-block">
				<div class="label">{$t('common_email')}</div>
				<div class="hint">{$t('prefs_reading_email_open_hint')}</div>
			</div>
			<div class="segmented" role="radiogroup" aria-label={$t('prefs_reading_email_open_mode')}>
				<button
					type="button"
					class="seg"
					class:active={emailOpenMode === 'reader'}
					role="radio"
					aria-checked={emailOpenMode === 'reader'}
					onclick={() => onEmailOpenModeChange('reader')}
				>
					{$t('prefs_reading_reader')}
				</button>
				<button
					type="button"
					class="seg"
					class:active={emailOpenMode === 'original'}
					role="radio"
					aria-checked={emailOpenMode === 'original'}
					onclick={() => onEmailOpenModeChange('original')}
				>
					{$t('reader_view_original')}
				</button>
			</div>
		</div>
	</div>
</SettingsGroup>

<style>
	.group-card {
		background: var(--card-bg);
		border-radius: 14px;
		overflow: hidden;
		box-shadow: inset 0 0 0 0.5px var(--border-primary);
		container-type: inline-size;
		container-name: settings-card;
	}

	.row {
		display: flex;
		align-items: center;
		gap: 16px;
		padding: 14px 18px;
		min-height: 52px;
		border-top: 0.5px solid var(--border-hairline);
	}

	.row:first-child {
		border-top: none;
	}

	.label-block {
		flex: 1;
		min-width: 0;
	}

	.label {
		font-family: var(--font-sans);
		font-size: 13px;
		font-weight: 500;
		color: var(--text-primary);
		margin-bottom: 2px;
	}

	.hint {
		font-family: var(--font-sans);
		font-size: 12px;
		color: var(--text-secondary);
		line-height: 1.4;
	}

	.content-type-static {
		font-family: var(--font-sans);
		font-size: 12.5px;
		font-weight: 500;
		color: var(--text-tertiary);
		padding: 6px 14px;
		border-radius: 7px;
		background: var(--bg-secondary);
		box-shadow: inset 0 0 0 0.5px var(--border-primary);
	}

	.segmented {
		display: inline-flex;
		background: var(--bg-secondary);
		padding: 3px;
		border-radius: 10px;
		box-shadow: inset 0 0 0 0.5px var(--border-primary);
	}

	.seg {
		padding: 6px 14px;
		border-radius: 7px;
		font-family: var(--font-sans);
		font-size: 12.5px;
		font-weight: 500;
		color: var(--text-secondary);
		transition:
			background 140ms,
			color 140ms,
			box-shadow 140ms;
		display: inline-flex;
		align-items: center;
		gap: 5px;
		background: transparent;
		border: none;
		cursor: pointer;
	}

	.seg.active {
		background: var(--bg-elevated);
		color: var(--text-primary);
		box-shadow: var(--shadow-1);
	}

	/* Wrap the segmented control under its label on narrow cards. */
	@container settings-card (max-width: 539px) {
		.row:has(.segmented) {
			flex-wrap: wrap;
		}

		.row:has(.segmented) .label-block {
			flex: 1 1 100%;
		}
	}
</style>
