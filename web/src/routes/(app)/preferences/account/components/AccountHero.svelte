<script lang="ts">
	import SettingsHero from '$lib/components/settings/SettingsHero.svelte';

	interface Props {
		avatarPreview: string;
		avatarInitial: string;
		displayName: string | null | undefined;
		username: string;
		memberSince: string;
		emailVerified: boolean | null | undefined;
		onFileChange: (event: Event) => void;
	}

	let {
		avatarPreview,
		avatarInitial,
		displayName,
		username,
		memberSince,
		emailVerified,
		onFileChange
	}: Props = $props();

	let fileInput: HTMLInputElement | undefined = $state();
</script>

<SettingsHero variant="account">
	<div class="hero-avatar-wrap">
		<div class="hero-avatar" aria-label="Avatar">
			{#if avatarPreview}
				<img src={avatarPreview} alt="" class="hero-avatar-img" />
			{:else}
				{avatarInitial}
			{/if}
		</div>
		<button
			type="button"
			class="hero-avatar-edit"
			onclick={() => fileInput?.click()}
			aria-label="Change avatar"
			title="Change avatar"
		>
			<svg viewBox="0 0 24 24" aria-hidden="true">
				<path d="M12 20h9" />
				<path d="M16.5 3.5a2.121 2.121 0 0 1 3 3L7 19l-4 1 1-4z" />
			</svg>
		</button>
		<input
			bind:this={fileInput}
			type="file"
			accept="image/png,image/jpeg,image/webp"
			class="visually-hidden"
			onchange={onFileChange}
		/>
	</div>
	<div class="hero-meta">
		<div class="hero-eyebrow"><span>Your account</span></div>
		<h1 class="hero-name">{displayName || 'Your name'}</h1>
		<div class="hero-pills">
			{#if username}
				<span class="hero-pill">
					<svg viewBox="0 0 24 24" aria-hidden="true">
						<rect x="3" y="6" width="18" height="14" rx="2" />
						<path d="M3 10h18" />
					</svg>
					<span class="key">{username}</span>
				</span>
			{/if}
			{#if memberSince}
				<span class="hero-pill">
					<svg viewBox="0 0 24 24" aria-hidden="true">
						<circle cx="12" cy="12" r="9" />
						<path d="M12 7v5l3 2" />
					</svg>
					Member since {memberSince}
				</span>
			{/if}
			{#if emailVerified}
				<span class="hero-pill verified">
					<svg viewBox="0 0 24 24" aria-hidden="true"><path d="M5 12l4 4 10-10" /></svg>
					Verified
				</span>
			{/if}
		</div>
	</div>
</SettingsHero>
