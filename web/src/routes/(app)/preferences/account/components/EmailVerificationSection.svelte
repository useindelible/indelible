<script lang="ts">
	import SettingsGroup from '$lib/components/settings/SettingsGroup.svelte';

	interface Props {
		email: string;
		emailVerified: boolean | null | undefined;
		emailRevealOpen: boolean;
		newEmail: string;
		currentPassword: string;
		onOpen: () => void;
		onCancel: () => void;
		onNewEmailChange: (value: string) => void;
		onCurrentPasswordChange: (value: string) => void;
	}

	let {
		email,
		emailVerified,
		emailRevealOpen,
		newEmail,
		currentPassword,
		onOpen,
		onCancel,
		onNewEmailChange,
		onCurrentPasswordChange
	}: Props = $props();
</script>

<SettingsGroup title="Email & verification">
	<div class="group-card">
		<div class="row">
			<div class="label-block">
				<div class="label">Primary email</div>
				<div class="hint">Where account notifications, password resets, and digests are sent.</div>
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
				<div class="label">Change email</div>
				<div class="hint">We'll send a verification link to the new address before switching.</div>
			</div>
			<div>
				<button type="button" class="btn ghost" disabled={emailRevealOpen} onclick={onOpen}>
					Change email
				</button>
			</div>
		</div>
		{#if emailRevealOpen}
			<div class="row with-stack">
				<div class="label-block">
					<div class="label">Switch your email</div>
					<div class="hint">
						For your safety, confirm with your current password. The new address will receive a
						verification link.
					</div>
				</div>
				<div class="input-group input-group--stacked">
					<input
						class="input"
						type="email"
						value={newEmail}
						placeholder="New email address"
						autocomplete="email"
						oninput={(event) => onNewEmailChange((event.currentTarget as HTMLInputElement).value)}
					/>
					<input
						class="input"
						type="password"
						value={currentPassword}
						placeholder="Current password"
						autocomplete="current-password"
						oninput={(event) =>
							onCurrentPasswordChange((event.currentTarget as HTMLInputElement).value)}
					/>
					<div class="reveal-actions">
						<button type="button" class="btn ghost" onclick={onCancel}>Cancel</button>
						<button type="button" class="btn primary" disabled>Send verification</button>
					</div>
				</div>
			</div>
		{/if}
	</div>
</SettingsGroup>
