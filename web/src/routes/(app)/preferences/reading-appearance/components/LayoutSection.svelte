<script lang="ts">
	import SettingsGroup from '$lib/components/settings/SettingsGroup.svelte';
	import { t } from '$lib/i18n';
	import type { DefaultViewDto, ListDensityDto, SidePanelModeDto, SidebarModeDto } from '$lib/api';

	interface Props {
		sidebarMode: SidebarModeDto;
		defaultView: DefaultViewDto;
		listDensity: ListDensityDto;
		sidePanel: SidePanelModeDto;
		onSidebarModeChange: (value: SidebarModeDto) => void;
		onDefaultViewChange: (value: DefaultViewDto) => void;
		onListDensityChange: (value: ListDensityDto) => void;
		onSidePanelChange: (value: SidePanelModeDto) => void;
	}

	let {
		sidebarMode,
		defaultView,
		listDensity,
		sidePanel,
		onSidebarModeChange,
		onDefaultViewChange,
		onListDensityChange,
		onSidePanelChange
	}: Props = $props();
</script>

<SettingsGroup title={$t('prefs_reading_layout_navigation')}>
	<div class="group-card">
		<div class="row">
			<div class="label-block">
				<div class="label">{$t('prefs_reading_sidebar_mode')}</div>
				<div class="hint">{$t('prefs_reading_sidebar_mode_hint')}</div>
			</div>
			<select
				class="select"
				value={sidebarMode}
				aria-label={$t('prefs_reading_sidebar_mode')}
				onchange={(event) => onSidebarModeChange(event.currentTarget.value as SidebarModeDto)}
			>
				<option value="expanded">{$t('prefs_reading_always_show')}</option>
				<option value="auto">{$t('prefs_reading_auto_collapse')}</option>
				<option value="collapsed">{$t('prefs_reading_always_hide')}</option>
			</select>
		</div>
		<div class="row">
			<div class="label-block">
				<div class="label">{$t('prefs_reading_default_view')}</div>
				<div class="hint">{$t('prefs_reading_default_view_hint')}</div>
			</div>
			<select
				class="select"
				value={defaultView}
				aria-label={$t('prefs_reading_default_view')}
				onchange={(event) => onDefaultViewChange(event.currentTarget.value as DefaultViewDto)}
			>
				<option value="library">{$t('common_library')}</option>
				<option value="feed">{$t('common_feed')}</option>
				<option value="search">{$t('common_search')}</option>
			</select>
		</div>
		<div class="row">
			<div class="label-block">
				<div class="label">{$t('prefs_reading_list_density')}</div>
				<div class="hint">{$t('prefs_reading_list_density_hint')}</div>
			</div>
			<select
				class="select"
				value={listDensity}
				aria-label={$t('prefs_reading_list_density')}
				onchange={(event) => onListDensityChange(event.currentTarget.value as ListDensityDto)}
			>
				<option value="compact">{$t('prefs_reading_compact')}</option>
				<option value="comfortable">{$t('prefs_reading_comfortable')}</option>
			</select>
		</div>
		<div class="row">
			<div class="label-block">
				<div class="label">{$t('prefs_reading_side_panel')}</div>
				<div class="hint">{$t('prefs_reading_side_panel_hint')}</div>
			</div>
			<select
				class="select"
				value={sidePanel}
				aria-label={$t('prefs_reading_side_panel')}
				onchange={(event) => onSidePanelChange(event.currentTarget.value as SidePanelModeDto)}
			>
				<option value="open">{$t('prefs_reading_show_in_reader')}</option>
				<option value="closed">{$t('prefs_reading_hide_by_default')}</option>
				<option value="auto">{$t('prefs_reading_floating_overlay')}</option>
			</select>
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

	.select {
		appearance: none;
		-webkit-appearance: none;
		background-color: var(--input-bg);
		color: var(--text-primary);
		border: none;
		outline: none;
		border-radius: 8px;
		padding: 8px 12px;
		font-family: var(--font-sans);
		font-size: 13.5px;
		box-shadow: var(--input-shadow);
		transition: box-shadow 120ms;
		width: 240px;
		background-image: url("data:image/svg+xml;utf8,<svg xmlns='http://www.w3.org/2000/svg' width='12' height='12' viewBox='0 0 24 24' fill='none' stroke='%236E6E73' stroke-width='1.8' stroke-linecap='round' stroke-linejoin='round'><polyline points='6 9 12 15 18 9'/></svg>");
		background-repeat: no-repeat;
		background-position: right 12px center;
		cursor: pointer;
	}

	.select:focus {
		box-shadow:
			var(--input-shadow),
			0 0 0 3px var(--accent-soft);
	}

	/* A 240px select + label can't share a narrow row; wrap so the select
	   drops under the label at full width. */
	@container settings-card (max-width: 539px) {
		.row {
			flex-wrap: wrap;
		}

		.label-block {
			flex: 1 1 100%;
		}

		.select {
			width: 100%;
		}
	}
</style>
