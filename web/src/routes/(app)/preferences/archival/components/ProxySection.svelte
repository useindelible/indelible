<script lang="ts">
	import SettingsGroup from '$lib/components/settings/SettingsGroup.svelte';
	import { t } from '$lib/i18n';

	// No capture, worker, renderer or extension path reads the stored proxy, so the
	// controls stay read-only rather than accepting settings that change nothing. The
	// saved values are still shown and still persist for the release that applies them.
	interface Props {
		proxyUrl: string;
		proxyAll: boolean;
	}

	let { proxyUrl, proxyAll }: Props = $props();
</script>

<SettingsGroup title={$t('archival_proxy_title')} meta={$t('archival_proxy_meta')}>
	<div class="group-card">
		<div class="row">
			<div class="label-block">
				<div class="label">
					{$t('archival_proxy_url')}
					<span class="badge coming">{$t('common_coming_soon')}</span>
				</div>
				<div class="hint">{$t('archival_proxy_protocols')}</div>
			</div>
			<div class="input-group">
				<input
					class="input mono"
					type="text"
					placeholder="socks5://127.0.0.1:1080"
					value={proxyUrl}
					aria-label={$t('archival_proxy_url')}
					disabled
				/>
			</div>
		</div>

		<div class="row">
			<div class="label-block">
				<div class="label">{$t('archival_proxy_all')}</div>
				<div class="hint">{$t('archival_proxy_all_hint')}</div>
			</div>
			<button
				type="button"
				class="toggle locked"
				class:on={proxyAll}
				role="switch"
				aria-checked={proxyAll}
				aria-label={$t('archival_proxy_all')}
				disabled
			></button>
		</div>

		<div class="proxy-status idle">
			<span class="status-text">
				<span class="status-dot"></span>
				<span>{$t('archival_proxy_inactive')}</span>
			</span>
			<span class="status-route">{proxyUrl.trim()}</span>
		</div>
	</div>
</SettingsGroup>
