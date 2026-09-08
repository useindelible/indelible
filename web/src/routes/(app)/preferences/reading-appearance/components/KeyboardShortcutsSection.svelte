<script lang="ts">
	import SettingsGroup from '$lib/components/settings/SettingsGroup.svelte';
	import ShortcutTable from '$lib/components/shortcuts/ShortcutTable.svelte';
	import { t } from '$lib/i18n';
</script>

<SettingsGroup title={$t('prefs_reading_shortcuts')}>
	<ShortcutTable />
</SettingsGroup>
