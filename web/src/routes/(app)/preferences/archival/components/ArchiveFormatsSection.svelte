<script lang="ts">
	import SettingsGroup from '$lib/components/settings/SettingsGroup.svelte';
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

<SettingsGroup
	title="Archive formats"
	meta="Pick which formats Indelible writes for every new save. Readable text is always preserved."
>
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
