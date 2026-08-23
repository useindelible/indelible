<script lang="ts">
	import SettingsGroup from '$lib/components/settings/SettingsGroup.svelte';
	import { t } from '$lib/i18n';

	interface Props {
		displayName: string;
		username: string;
		onDisplayNameChange: (value: string) => void;
	}

	let { displayName, username, onDisplayNameChange }: Props = $props();
</script>

<SettingsGroup title={$t('account_identity')}>
	<div class="group-card">
		<div class="row">
			<div class="label-block">
				<div class="label">{$t('auth_display_name')}</div>
				<div class="hint">{$t('account_display_name_hint')}</div>
			</div>
			<div class="input-group">
				<input
					class="input"
					type="text"
					value={displayName}
					maxlength={100}
					aria-label={$t('auth_display_name')}
					oninput={(event) => onDisplayNameChange((event.currentTarget as HTMLInputElement).value)}
				/>
			</div>
		</div>
		<div class="row">
			<div class="label-block">
				<div class="label">{$t('account_username')}</div>
				<div class="hint">{$t('account_username_hint')}</div>
			</div>
			<div class="input-group">
				<input
					class="input mono readonly"
					type="text"
					value={username}
					readonly
					aria-label={$t('account_username')}
				/>
			</div>
		</div>
	</div>
</SettingsGroup>
