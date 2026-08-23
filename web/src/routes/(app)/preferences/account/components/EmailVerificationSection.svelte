<script lang="ts">
	import SettingsGroup from '$lib/components/settings/SettingsGroup.svelte';
	import { t } from '$lib/i18n';

	interface Props {
		email: string;
		emailVerified: boolean | null | undefined;
	}

	let { email, emailVerified }: Props = $props();
</script>

<SettingsGroup title={$t('account_email_verification')}>
	<div class="group-card">
		<div class="row">
			<div class="label-block">
				<div class="label">{$t('account_primary_email')}</div>
				<div class="hint">{$t('account_primary_email_hint')}</div>
			</div>
			<div class="input-row">
				<input
					class="input readonly"
					type="email"
					value={email}
					readonly
					aria-label={$t('account_primary_email')}
				/>
				{#if emailVerified}
					<span class="hero-pill verified">
						<svg viewBox="0 0 24 24" aria-hidden="true"><path d="M5 12l4 4 10-10" /></svg>
						{$t('account_verified')}
					</span>
				{/if}
			</div>
		</div>
		<div class="row">
			<div class="label-block">
				<div class="label">
					{$t('account_change_email')} <span class="badge coming">{$t('common_coming_soon')}</span>
				</div>
				<div class="hint">{$t('account_change_email_hint')}</div>
			</div>
			<div>
				<button type="button" class="btn ghost" disabled>{$t('account_change_email')}</button>
			</div>
		</div>
	</div>
</SettingsGroup>
