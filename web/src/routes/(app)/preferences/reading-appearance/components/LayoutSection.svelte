<script lang="ts">
	import SettingsGroup from '$lib/components/settings/SettingsGroup.svelte';
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

<SettingsGroup title="Layout & Navigation">
	<div class="group-card">
		<div class="row">
			<div class="label-block">
				<div class="label">Sidebar mode</div>
				<div class="hint">
					Always show keeps the sidebar pinned. Auto-collapse hides it on narrower screens to widen
					the reading column. Always hide tucks it away — use the reveal button to bring it back.
				</div>
			</div>
			<select
				class="select"
				value={sidebarMode}
				aria-label="Sidebar mode"
				onchange={(event) => onSidebarModeChange(event.currentTarget.value as SidebarModeDto)}
			>
				<option value="expanded">Always show</option>
				<option value="auto">Auto-collapse</option>
				<option value="collapsed">Always hide</option>
			</select>
		</div>
		<div class="row">
			<div class="label-block">
				<div class="label">Default view</div>
				<div class="hint">The screen Indelible opens to when you launch the app.</div>
			</div>
			<select
				class="select"
				value={defaultView}
				aria-label="Default view"
				onchange={(event) => onDefaultViewChange(event.currentTarget.value as DefaultViewDto)}
			>
				<option value="library">Library</option>
				<option value="feed">Feed</option>
				<option value="search">Search</option>
			</select>
		</div>
		<div class="row">
			<div class="label-block">
				<div class="label">List density</div>
				<div class="hint">How tightly article rows pack into your library and inbox.</div>
			</div>
			<select
				class="select"
				value={listDensity}
				aria-label="List density"
				onchange={(event) => onListDensityChange(event.currentTarget.value as ListDensityDto)}
			>
				<option value="compact">Compact</option>
				<option value="comfortable">Comfortable</option>
			</select>
		</div>
		<div class="row">
			<div class="label-block">
				<div class="label">Side panel</div>
				<div class="hint">Where annotations, highlights, and outline live in the reader.</div>
			</div>
			<select
				class="select"
				value={sidePanel}
				aria-label="Side panel"
				onchange={(event) => onSidePanelChange(event.currentTarget.value as SidePanelModeDto)}
			>
				<option value="open">Show in reader</option>
				<option value="closed">Hide by default</option>
				<option value="auto">Floating overlay</option>
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
