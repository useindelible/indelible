<script lang="ts">
	import type { ArchiveFormat } from '../archival-model';
	import { t } from '$lib/i18n';

	interface Props {
		format: ArchiveFormat;
		on: boolean;
		onToggle: () => void;
	}

	let { format, on, onToggle }: Props = $props();
</script>

<div class="fmt-row" class:on class:disabled={format.comingSoon}>
	<div class="row-icon">
		{#if format.id === 'readable'}
			<svg viewBox="0 0 24 24"><path d="M5 6h14M5 10h14M5 14h10M5 18h12" /></svg>
		{:else if format.id === 'monolith'}
			<svg viewBox="0 0 24 24"
				><path d="M4 7l8-4 8 4-8 4z" /><path d="M4 12l8 4 8-4" /><path d="M4 17l8 4 8-4" /></svg
			>
		{:else if format.id === 'pdf'}
			<svg viewBox="0 0 24 24"
				><rect x="5" y="3" width="14" height="18" rx="2" /><path d="M9 8h6M9 12h6M9 16h4" /></svg
			>
		{:else if format.id === 'screenshot'}
			<svg viewBox="0 0 24 24"
				><rect x="3" y="5" width="18" height="14" rx="2" /><circle cx="9" cy="11" r="2" /><path
					d="M21 17l-5-5-9 7"
				/></svg
			>
		{:else if format.id === 'warc'}
			<svg viewBox="0 0 24 24"
				><rect x="3" y="6" width="18" height="14" rx="2" /><path d="M3 10h18" /><path
					d="M8 3v3M16 3v3"
				/></svg
			>
		{/if}
	</div>
	<div class="meta">
		<div class="name">
			{$t(format.labelKey)}
			{#if format.alwaysOn}
				<span class="badge always">
					<svg viewBox="0 0 14 14"><path d="M2.5 7.5L5.5 10.5L11.5 3.5" /></svg>
					{$t('archival_always_on')}
				</span>
			{:else if format.comingSoon}
				<span class="badge coming">{$t('common_coming_soon')}</span>
			{/if}
		</div>
		<div class="desc">{$t(format.descKey)}</div>
	</div>
	<div class="size">{format.size}</div>
	<div></div>
	<button
		type="button"
		class="toggle"
		class:on
		class:locked={format.alwaysOn || format.comingSoon}
		role="switch"
		aria-checked={on}
		aria-disabled={format.alwaysOn || format.comingSoon}
		aria-label={format.alwaysOn
			? $t('archival_format_always_enabled', { values: { format: $t(format.labelKey) } })
			: format.comingSoon
				? $t('archival_format_coming_soon', { values: { format: $t(format.labelKey) } })
				: $t(format.labelKey)}
		onclick={onToggle}
	></button>
</div>
