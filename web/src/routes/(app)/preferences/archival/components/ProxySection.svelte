<script lang="ts">
	import SettingsGroup from '$lib/components/settings/SettingsGroup.svelte';

	interface Props {
		proxyUrl: string;
		proxyAll: boolean;
		proxyConfigured: boolean;
		onProxyUrlChange: (url: string) => void;
		onProxyAllChange: (enabled: boolean) => void;
	}

	let { proxyUrl, proxyAll, proxyConfigured, onProxyUrlChange, onProxyAllChange }: Props = $props();

	function handleProxyUrlInput(e: Event) {
		onProxyUrlChange((e.target as HTMLInputElement).value);
	}
</script>

<SettingsGroup
	title="Proxy"
	meta="Route archival fetches through a proxy. Useful for paywalled regional content or self-hosted scrubbers."
>
	<div class="group-card">
		<div class="row">
			<div class="label-block">
				<div class="label">Proxy URL</div>
				<div class="hint">SOCKS5, HTTP, or HTTPS. Leave blank to fetch directly.</div>
			</div>
			<div class="input-group">
				<input
					class="input mono"
					type="text"
					placeholder="socks5://127.0.0.1:1080"
					value={proxyUrl}
					oninput={handleProxyUrlInput}
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
				class="toggle"
				class:on={proxyAll && proxyConfigured}
				class:locked={!proxyConfigured}
				role="switch"
				aria-checked={proxyAll && proxyConfigured}
				aria-disabled={!proxyConfigured}
				aria-label="Proxy for all requests"
				onclick={() => {
					if (!proxyConfigured) return;
					onProxyAllChange(!proxyAll);
				}}
			></button>
		</div>

		<div class="proxy-status" class:live={proxyConfigured} class:idle={!proxyConfigured}>
			<span class="status-text">
				<span class="status-dot"></span>
				<span>
					{#if !proxyConfigured}
						No proxy configured · fetches go direct
					{:else if proxyAll}
						All requests routed through proxy
					{:else}
						Page fetches routed through proxy
					{/if}
				</span>
			</span>
			<span class="status-route">{proxyConfigured ? proxyUrl.trim() : ''}</span>
		</div>
	</div>
</SettingsGroup>
