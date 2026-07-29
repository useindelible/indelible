<script lang="ts">
	import SettingsGroup from '$lib/components/settings/SettingsGroup.svelte';
	import type { DuplicateAction, DuplicateSensitivity } from '../archival-model';

	interface Props {
		dupEnabled: boolean;
		dupSensitivity: DuplicateSensitivity;
		dupAction: DuplicateAction;
		onEnabledChange: (enabled: boolean) => void;
		onSensitivityChange: (sensitivity: DuplicateSensitivity) => void;
		onActionChange: (action: DuplicateAction) => void;
	}

	let {
		dupEnabled,
		dupSensitivity,
		dupAction,
		onEnabledChange,
		onSensitivityChange,
		onActionChange
	}: Props = $props();

	function handleSensitivityInput(e: Event) {
		const value = Number((e.target as HTMLInputElement).value);
		if (value === 1 || value === 2 || value === 3) onSensitivityChange(value);
	}

	function handleActionChange(e: Event) {
		const value = (e.target as HTMLSelectElement).value;
		if (value === 'notify' || value === 'skip' || value === 'merge') onActionChange(value);
	}
</script>

<SettingsGroup
	title="Duplicate detection"
	meta="When you save something you’ve already saved, Indelible can warn you, skip silently, or merge into the existing item."
>
	<div class="group-card">
		<div class="row">
			<div class="label-block">
				<div class="label">Detect near-duplicates</div>
				<div class="hint">
					Uses SimHash similarity scoring on the readable text — catches reposts, re-publishes, and
					tiny edits.
				</div>
			</div>
			<button
				type="button"
				class="toggle"
				class:on={dupEnabled}
				role="switch"
				aria-checked={dupEnabled}
				aria-label="Enable duplicate detection"
				onclick={() => onEnabledChange(!dupEnabled)}
			></button>
		</div>

		<div class="slider-row" class:disabled={!dupEnabled}>
			<div>
				<div class="row-label">Sensitivity</div>
				<div class="row-hint">
					How similar two items must be before they’re flagged. Higher = more aggressive matching.
				</div>
				<div class="sensitivity">
					<div class="sensitivity-track">
						<input
							type="range"
							class="sensitivity-input"
							min="1"
							max="3"
							step="1"
							value={dupSensitivity}
							disabled={!dupEnabled}
							oninput={handleSensitivityInput}
							aria-label="Duplicate sensitivity"
						/>
					</div>
					<div class="sensitivity-meta">
						<span class:current={dupEnabled && dupSensitivity === 1}>Low</span>
						<span class:current={dupEnabled && dupSensitivity === 2}>Medium</span>
						<span class:current={dupEnabled && dupSensitivity === 3}>High</span>
					</div>
				</div>
			</div>
			<div class="input-group narrow">
				<select
					class="select"
					value={dupAction}
					disabled={!dupEnabled}
					aria-label="Duplicate action"
					onchange={handleActionChange}
				>
					<option value="notify">Notify me</option>
					<option value="skip">Skip silently</option>
					<option value="merge">Merge with existing</option>
				</select>
				<div class="help">When a duplicate is found</div>
			</div>
		</div>
	</div>
</SettingsGroup>
