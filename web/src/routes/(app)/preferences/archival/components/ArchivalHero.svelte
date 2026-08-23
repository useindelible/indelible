<script lang="ts">
	import SettingsHero from '$lib/components/settings/SettingsHero.svelte';
	import { t, type MessageKey } from '$lib/i18n';
	import {
		ARCHIVE_FORMATS,
		getArchiveFormatStatus,
		type ArchiveFormatToggleId,
		type FormatId
	} from '../archival-model';

	interface Props {
		formats: Record<ArchiveFormatToggleId, boolean>;
	}

	let { formats }: Props = $props();

	function shortFormatName(id: FormatId): MessageKey {
		if (id === 'readable') return 'archival_format_readable_short';
		if (id === 'monolith') return 'archival_format_monolith_short';
		if (id === 'pdf') return 'archival_format_pdf_short';
		if (id === 'screenshot') return 'archival_format_screenshot_short';
		return 'archival_format_warc_short';
	}
</script>

<SettingsHero variant="archival">
	<div class="hero-text">
		<div class="hero-eyebrow">
			<span class="dot"></span>
			<span>{$t('archival_hero_eyebrow')}</span>
		</div>
		<h1 class="hero-title">
			{$t('archival_hero_title_line_one')}<br />{$t('archival_hero_title_line_two')}
		</h1>
		<p class="hero-sub">{$t('archival_hero_subtitle')}</p>
	</div>

	<div class="format-strip" aria-hidden="true">
		{#each ARCHIVE_FORMATS as fmt (fmt.id)}
			{@const status = getArchiveFormatStatus(fmt.id, formats)}
			<div
				class="format-card status-{status} fmt-{fmt.id}"
				class:coming={status === 'coming'}
				class:off={status === 'off'}
				title={$t(fmt.labelKey)}
			>
				<div class="fmt-status-dot"></div>
				<div class="fmt-icon">
					{#if fmt.id === 'readable'}
						<svg viewBox="0 0 24 24"><path d="M5 6h14M5 10h14M5 14h10M5 18h12" /></svg>
					{:else if fmt.id === 'monolith'}
						<svg viewBox="0 0 24 24"
							><path d="M4 7l8-4 8 4-8 4z" /><path d="M4 12l8 4 8-4" /><path
								d="M4 17l8 4 8-4"
							/></svg
						>
					{:else if fmt.id === 'pdf'}
						<svg viewBox="0 0 24 24"
							><rect x="5" y="3" width="14" height="18" rx="2" /><path
								d="M9 8h6M9 12h6M9 16h4"
							/></svg
						>
					{:else if fmt.id === 'screenshot'}
						<svg viewBox="0 0 24 24"
							><rect x="3" y="5" width="18" height="14" rx="2" /><circle
								cx="9"
								cy="11"
								r="2"
							/><path d="M21 17l-5-5-9 7" /></svg
						>
					{:else if fmt.id === 'warc'}
						<svg viewBox="0 0 24 24"
							><rect x="3" y="6" width="18" height="14" rx="2" /><path d="M3 10h18" /><path
								d="M8 3v3M16 3v3"
							/></svg
						>
					{/if}
				</div>
				<div class="fmt-preview">
					{#if fmt.id === 'readable' || fmt.id === 'monolith'}
						<div class="line long"></div>
						<div class="line med"></div>
						<div class="line long"></div>
						<div class="line short"></div>
						<div class="line med"></div>
					{/if}
				</div>
				<div class="fmt-name">{$t(shortFormatName(fmt.id))}</div>
			</div>
		{/each}
	</div>
</SettingsHero>
