<script lang="ts">
	import SettingsGroup from '$lib/components/settings/SettingsGroup.svelte';

	// No capture, worker, renderer or extension path reads the stored proxy, so the
	// controls stay read-only rather than accepting settings that change nothing. The
	// saved values are still shown and still persist for the release that applies them.
	interface Props {
		proxyUrl: string;
		proxyAll: boolean;
	}

	let { proxyUrl, proxyAll }: Props = $props();
</script>

<SettingsGroup
	title="Proxy"
	meta="Route archival fetches through a proxy. Useful for paywalled regional content or self-hosted scrubbers."
>
	<div class="group-card">
		<div class="row">
			<div class="label-block">
				<div class="label">
					Proxy URL
					<span class="badge coming">Coming soon</span>
				</div>
				<div class="hint">SOCKS5, HTTP, or HTTPS.</div>
			</div>
			<div class="input-group">
				<input
					class="input mono"
					type="text"
					placeholder="socks5://127.0.0.1:1080"
					value={proxyUrl}
					aria-label="Proxy URL"
					disabled
				/>
			</div>
		</div>

		<div class="row">
			<div class="label-block">
				<div class="label">Proxy for all requests</div>
				<div class="hint">
					Including image and asset fetches during archival. Slower, but consistent network
					identity.
				</div>
			</div>
			<button
				type="button"
				class="toggle locked"
				class:on={proxyAll}
				role="switch"
				aria-checked={proxyAll}
				aria-label="Proxy for all requests"
				disabled
			></button>
		</div>

		<div class="proxy-status idle">
			<span class="status-text">
				<span class="status-dot"></span>
				<span>Coming soon · saved proxy settings are not active.</span>
			</span>
			<span class="status-route">{proxyUrl.trim()}</span>
		</div>
	</div>
</SettingsGroup>
