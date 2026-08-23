<script lang="ts">
	import SettingsGroup from '$lib/components/settings/SettingsGroup.svelte';
	import { t } from '$lib/i18n';
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

<SettingsGroup title={$t('archival_duplicate_title')} meta={$t('archival_duplicate_meta')}>
	<div class="group-card">
		<div class="row">
			<div class="label-block">
				<div class="label">{$t('archival_duplicate_detect')}</div>
				<div class="hint">{$t('archival_duplicate_detect_hint')}</div>
			</div>
			<button
				type="button"
				class="toggle"
				class:on={dupEnabled}
				role="switch"
				aria-checked={dupEnabled}
				aria-label={$t('archival_duplicate_enable')}
				onclick={() => onEnabledChange(!dupEnabled)}
			></button>
		</div>

		<div class="slider-row" class:disabled={!dupEnabled}>
			<div>
				<div class="row-label">{$t('archival_sensitivity')}</div>
				<div class="row-hint">{$t('archival_sensitivity_hint')}</div>
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
							aria-label={$t('archival_duplicate_sensitivity')}
						/>
					</div>
					<div class="sensitivity-meta">
						<span class:current={dupEnabled && dupSensitivity === 1}>{$t('archival_low')}</span>
						<span class:current={dupEnabled && dupSensitivity === 2}>{$t('archival_medium')}</span>
						<span class:current={dupEnabled && dupSensitivity === 3}>{$t('archival_high')}</span>
					</div>
				</div>
			</div>
			<div class="input-group narrow">
				<select
					class="select"
					value={dupAction}
					disabled={!dupEnabled}
					aria-label={$t('archival_duplicate_action')}
					onchange={handleActionChange}
				>
					<option value="notify">{$t('archival_duplicate_notify')}</option>
					<option value="skip">{$t('archival_duplicate_skip')}</option>
					<option value="merge">{$t('archival_duplicate_merge')}</option>
				</select>
				<div class="help">{$t('archival_duplicate_action_hint')}</div>
			</div>
		</div>
	</div>
</SettingsGroup>
