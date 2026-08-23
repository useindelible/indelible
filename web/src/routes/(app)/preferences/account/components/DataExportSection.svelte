<script lang="ts">
	import SettingsGroup from '$lib/components/settings/SettingsGroup.svelte';
	import { t } from '$lib/i18n';
</script>

<SettingsGroup title={$t('account_export_title')}>
	<div class="group-card">
		<div class="row">
			<div class="label-block">
				<div class="label">
					{$t('account_export_archive')}
					<span class="badge coming">{$t('common_coming_soon')}</span>
				</div>
				<div class="hint">{$t('account_export_hint')}</div>
			</div>
			<div>
				<button type="button" class="btn ghost" disabled>
					<svg viewBox="0 0 24 24" aria-hidden="true">
						<path d="M12 3v12" />
						<path d="M7 10l5 5 5-5" />
						<path d="M5 21h14" />
					</svg>
					{$t('account_export_request')}
				</button>
			</div>
		</div>
	</div>
</SettingsGroup>

<style>
	.label {
		display: inline-flex;
		align-items: center;
		gap: 8px;
	}

	.badge.coming {
		display: inline-flex;
		align-items: center;
		padding: 2px 8px;
		border-radius: 980px;
		background: var(--arc-warning-soft);
		color: var(--arc-warning-fg);
		font-size: 10.5px;
		font-weight: 600;
		letter-spacing: 0.02em;
		line-height: 1.4;
		white-space: nowrap;
	}
</style>
