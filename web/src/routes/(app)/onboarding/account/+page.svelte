<script lang="ts">
	import { goto } from '$app/navigation';
	import { resolve } from '$app/paths';
	import StepLayout from '$lib/components/onboarding/StepLayout.svelte';
	import { getAuth } from '$lib/stores/auth.svelte';
	import { getOnboarding } from '$lib/stores/onboarding.svelte';
	import { applyTheme, saveTheme, type ThemePreference } from '$lib/styles/theme';
	import { updateProfile } from '$lib/api';
	import { uploadAvatar, MAX_AVATAR_SIZE_BYTES } from '$lib/api/avatar';

	const auth = getAuth();
	const onboarding = getOnboarding();

	let displayName = $state(auth.user?.display_name ?? '');
	let theme = $state<ThemePreference>(auth.user?.theme ?? 'system');
	let submitting = $state(false);
	let avatarFile = $state<File | null>(null);
	let avatarPreview = $state<string | null>(null);
	let avatarError = $state('');
	let formError = $state('');
	let fileInput: HTMLInputElement | undefined = $state();

	const themeOptions: { value: ThemePreference; label: string }[] = [
		{ value: 'light', label: 'Light' },
		{ value: 'dark', label: 'Dark' },
		{ value: 'system', label: 'Auto' }
	];

	function selectTheme(value: ThemePreference) {
		theme = value;
		applyTheme(value);
	}

	function handleAvatarClick() {
		fileInput?.click();
	}

	function handleFileChange(e: Event) {
		const target = e.target as HTMLInputElement;
		const file = target.files?.[0];
		if (!file) return;

		if (!['image/jpeg', 'image/png', 'image/webp'].includes(file.type)) {
			avatarError = 'Please select a JPEG, PNG, or WebP image';
			return;
		}
		if (file.size > MAX_AVATAR_SIZE_BYTES) {
			avatarError = 'Image must be smaller than 2 MB';
			return;
		}

		avatarError = '';
		avatarFile = file;
		if (avatarPreview) URL.revokeObjectURL(avatarPreview);
		avatarPreview = URL.createObjectURL(file);
	}

	async function handleContinue() {
		submitting = true;
		formError = '';
		try {
			const { data: profile, error } = await updateProfile({
				body: { display_name: displayName, theme }
			});
			if (!profile || error) {
				formError = 'Could not save your profile. Please try again.';
				return;
			}

			if (avatarFile) {
				try {
					const result = await uploadAvatar(avatarFile);
					if (!result.success) {
						switch (result.error.code) {
							case 'invalid_type':
								avatarError = 'Unsupported image type';
								break;
							case 'too_large':
								avatarError = 'Image must be smaller than 2 MB';
								break;
							default:
								avatarError = result.error.message;
						}
						return;
					}
				} catch {
					avatarError = 'Upload failed. Please try again or skip.';
					return;
				}
			}

			saveTheme(theme);
			const completed = await onboarding.completeStep(1, { display_name: displayName, theme });
			if (!completed) return;
			await auth.refresh();
			goto(resolve('/onboarding/add-content'));
		} finally {
			submitting = false;
		}
	}
</script>

<StepLayout
	title="Set up your profile"
	description="Personalize your Indelible experience."
	currentStep={1}
	{submitting}
	onContinue={handleContinue}
>
	<div class="account-form">
		{#if formError || onboarding.error}
			<p class="avatar-error">{formError || onboarding.error}</p>
		{/if}
		<div class="avatar-section">
			<button
				type="button"
				class="avatar-placeholder"
				aria-label="Upload profile picture (optional)"
				onclick={handleAvatarClick}
			>
				{#if avatarPreview}
					<img src={avatarPreview} alt="Profile preview" class="avatar-preview" />
				{:else}
					<svg
						width="28"
						height="28"
						viewBox="0 0 24 24"
						fill="none"
						stroke="var(--text-secondary)"
						stroke-width="1.5"
						stroke-linecap="round"
						stroke-linejoin="round"
						aria-hidden="true"
					>
						<path d="M20 21v-2a4 4 0 0 0-4-4H8a4 4 0 0 0-4 4v2" />
						<circle cx="12" cy="7" r="4" />
					</svg>
				{/if}
				<div class="camera-badge" aria-hidden="true">
					<svg
						width="12"
						height="12"
						viewBox="0 0 24 24"
						fill="none"
						stroke="white"
						stroke-width="2"
						stroke-linecap="round"
						stroke-linejoin="round"
					>
						<path
							d="M23 19a2 2 0 0 1-2 2H3a2 2 0 0 1-2-2V8a2 2 0 0 1 2-2h4l2-3h6l2 3h4a2 2 0 0 1 2 2z"
						/>
						<circle cx="12" cy="13" r="4" />
					</svg>
				</div>
			</button>
			<span class="avatar-label">Optional</span>
			{#if avatarError}
				<span class="avatar-error">{avatarError}</span>
			{/if}
			<input
				bind:this={fileInput}
				type="file"
				accept="image/jpeg,image/png,image/webp"
				class="visually-hidden"
				onchange={handleFileChange}
			/>
		</div>

		<label class="field">
			<span class="field-label"
				>Display name <span class="required" aria-hidden="true">*</span></span
			>
			<input
				type="text"
				class="field-input"
				bind:value={displayName}
				placeholder="Your name"
				autocomplete="name"
			/>
		</label>

		<div class="field">
			<span class="field-label">Theme preference</span>
			<div class="theme-grid">
				{#each themeOptions as opt (opt.value)}
					<button
						type="button"
						class="theme-card"
						class:selected={theme === opt.value}
						onclick={() => selectTheme(opt.value)}
					>
						{#if theme === opt.value}
							<div class="theme-check" aria-hidden="true">
								<svg width="10" height="10" viewBox="0 0 10 10" fill="none">
									<path
										d="M2 5l2.5 2.5L8 2.5"
										stroke="white"
										stroke-width="2"
										stroke-linecap="round"
										stroke-linejoin="round"
									/>
								</svg>
							</div>
						{/if}
						<div class="theme-preview theme-preview-{opt.value}">
							{#if opt.value === 'light'}
								<svg
									width="20"
									height="20"
									viewBox="0 0 24 24"
									fill="none"
									stroke="#86868B"
									stroke-width="1.5"
									stroke-linecap="round"
									aria-hidden="true"
								>
									<circle cx="12" cy="12" r="4" />
									<path
										d="M12 2v2M12 20v2M2 12h2M20 12h2M4.93 4.93l1.41 1.41M17.66 17.66l1.41 1.41M4.93 19.07l1.41-1.41M17.66 6.34l1.41-1.41"
									/>
								</svg>
							{:else if opt.value === 'dark'}
								<svg
									width="20"
									height="20"
									viewBox="0 0 24 24"
									fill="none"
									stroke="#98989D"
									stroke-width="1.5"
									stroke-linecap="round"
									aria-hidden="true"
								>
									<path d="M21 12.79A9 9 0 1 1 11.21 3 7 7 0 0 0 21 12.79z" />
								</svg>
							{:else}
								<svg width="20" height="20" viewBox="0 0 24 24" fill="none" aria-hidden="true">
									<circle cx="12" cy="12" r="9" stroke="#86868B" stroke-width="1.5" />
									<path
										d="M12 3a9 9 0 0 1 0 18V3z"
										fill="#2C2C2E"
										stroke="#86868B"
										stroke-width="1.5"
									/>
								</svg>
							{/if}
						</div>
						<span class="theme-label">{opt.label}</span>
					</button>
				{/each}
			</div>
		</div>
	</div>
</StepLayout>

<style>
	.account-form {
		display: flex;
		flex-direction: column;
		gap: 24px;
	}

	.avatar-section {
		display: flex;
		flex-direction: column;
		align-items: center;
		gap: 8px;
	}

	.avatar-placeholder {
		position: relative;
		width: 80px;
		height: 80px;
		border-radius: 50%;
		border: 2px dashed var(--border-secondary);
		background: rgba(0, 0, 0, 0.04);
		display: flex;
		align-items: center;
		justify-content: center;
		cursor: pointer;
		padding: 0;
		overflow: hidden;
	}

	:global([data-theme='dark']) .avatar-placeholder {
		background: rgba(255, 255, 255, 0.06);
	}

	.avatar-preview {
		width: 100%;
		height: 100%;
		object-fit: cover;
		border-radius: 50%;
	}

	.camera-badge {
		position: absolute;
		bottom: 0;
		right: 0;
		width: 26px;
		height: 26px;
		border-radius: 50%;
		background: var(--accent);
		border: 2px solid var(--bg-elevated);
		display: flex;
		align-items: center;
		justify-content: center;
	}

	.avatar-label {
		font-family: var(--font-sans);
		font-size: 12px;
		font-weight: 400;
		letter-spacing: -0.005em;
		color: var(--text-secondary);
	}

	.avatar-error {
		font-family: var(--font-sans);
		font-size: 12px;
		font-weight: 400;
		color: var(--destructive);
	}

	.visually-hidden {
		position: absolute;
		width: 1px;
		height: 1px;
		padding: 0;
		margin: -1px;
		overflow: hidden;
		clip: rect(0, 0, 0, 0);
		white-space: nowrap;
		border: 0;
	}

	.field {
		display: flex;
		flex-direction: column;
		gap: 8px;
	}

	.field-label {
		font-family: var(--font-sans);
		font-size: 13px;
		font-weight: 600;
		letter-spacing: -0.01em;
		color: var(--text-secondary);
	}

	.required {
		color: var(--destructive);
	}

	.field-input {
		width: 100%;
		padding: 10px 14px;
		font-family: var(--font-sans);
		font-size: 15px;
		letter-spacing: -0.01em;
		color: var(--text-primary);
		background: var(--bg-elevated);
		border: 1px solid var(--border-primary);
		border-radius: var(--radius-sm);
		outline: none;
		transition:
			border-color 0.15s ease,
			box-shadow 0.15s ease;
		box-sizing: border-box;
	}

	.field-input:focus {
		border-color: var(--accent);
		box-shadow: 0 0 0 3px var(--fill-selected);
	}

	.theme-grid {
		display: grid;
		grid-template-columns: repeat(3, 1fr);
		gap: 12px;
	}

	.theme-card {
		position: relative;
		display: flex;
		flex-direction: column;
		align-items: center;
		gap: 10px;
		padding: 16px;
		background: var(--bg-elevated);
		border: 1.5px solid var(--border-primary);
		border-radius: 14px;
		cursor: pointer;
		transition:
			border-color 0.15s ease,
			background 0.15s ease;
	}

	.theme-card:hover {
		border-color: var(--border-secondary);
	}

	.theme-card.selected {
		border-color: var(--accent);
		background: var(--fill-selected);
	}

	.theme-check {
		position: absolute;
		top: 8px;
		right: 8px;
		width: 20px;
		height: 20px;
		border-radius: 50%;
		background: var(--accent);
		display: flex;
		align-items: center;
		justify-content: center;
	}

	.theme-preview {
		width: 100%;
		height: 48px;
		border-radius: 8px;
		display: flex;
		align-items: center;
		justify-content: center;
	}

	.theme-preview-light {
		background: var(--bg-primary);
		border: 1px solid rgba(0, 0, 0, 0.08);
	}

	.theme-preview-dark {
		background: var(--text-primary);
		border: 1px solid rgba(255, 255, 255, 0.08);
	}

	.theme-preview-system {
		background: var(--fill-selected-strong);
		border: 1px solid rgba(0, 0, 0, 0.08);
	}

	.theme-label {
		font-family: var(--font-sans);
		font-size: 13px;
		font-weight: 500;
		letter-spacing: -0.01em;
		color: var(--text-primary);
	}
</style>
