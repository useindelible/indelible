<script lang="ts">
	import SettingsGroup from '$lib/components/settings/SettingsGroup.svelte';

	interface Props {
		email: string;
		emailVerified: boolean | null | undefined;
	}

	let { email, emailVerified }: Props = $props();
</script>

<SettingsGroup title="Email & verification">
	<div class="group-card">
		<div class="row">
			<div class="label-block">
				<div class="label">Primary email</div>
				<div class="hint">The address this account signs in with.</div>
			</div>
			<div class="input-row">
				<input
					class="input readonly"
					type="email"
					value={email}
					readonly
					aria-label="Primary email"
				/>
				{#if emailVerified}
					<span class="hero-pill verified">
						<svg viewBox="0 0 24 24" aria-hidden="true"><path d="M5 12l4 4 10-10" /></svg>
						Verified
					</span>
				{/if}
			</div>
		</div>
		<div class="row">
			<div class="label-block">
				<div class="label">
					Change email <span class="badge coming">Coming soon</span>
				</div>
				<div class="hint">
					Switching addresses signs you out everywhere and needs a verification link to sign
					back in. This server has no outbound email configured to send that link, so the
					change would lock you out. Available once outbound email is supported.
				</div>
			</div>
			<div>
				<button type="button" class="btn ghost" disabled>Change email</button>
			</div>
		</div>
	</div>
</SettingsGroup>
