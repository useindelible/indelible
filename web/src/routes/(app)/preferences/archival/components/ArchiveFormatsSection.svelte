<script lang="ts">
	import SettingsGroup from '$lib/components/settings/SettingsGroup.svelte';
	import { t } from '$lib/i18n';
	import {
		ARCHIVE_FORMATS,
		isArchiveFormatOn,
		type ArchiveFormatToggleId,
		type FormatId
	} from '../archival-model';
	import ArchiveFormatRow from './ArchiveFormatRow.svelte';

	interface Props {
		formats: Record<ArchiveFormatToggleId, boolean>;
		onToggleFormat: (id: FormatId) => void;
	}

	let { formats, onToggleFormat }: Props = $props();
</script>

<SettingsGroup title={$t('archival_formats_title')} meta={$t('archival_formats_meta')}>
	<div class="group-card">
		{#each ARCHIVE_FORMATS as format (format.id)}
			<ArchiveFormatRow
				{format}
				on={isArchiveFormatOn(format.id, formats)}
				onToggle={() => onToggleFormat(format.id)}
			/>
		{/each}
	</div>
</SettingsGroup>
