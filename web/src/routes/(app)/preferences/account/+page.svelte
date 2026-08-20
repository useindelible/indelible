<script lang="ts">
	import { untrack } from 'svelte';
	import { getAuth } from '$lib/stores/auth.svelte';
	import { uploadAvatar, MAX_AVATAR_SIZE_BYTES } from '$lib/api/avatar';
	import SavePill from '$lib/components/settings/SavePill.svelte';
	import SettingsGroup from '$lib/components/settings/SettingsGroup.svelte';
	import {
		createAccountSnapshot,
		formatMemberSince,
		getAccountAvatarInitial,
		getAccountUsername,
		isDeleteEmailConfirmed
	} from './account-model';
	import AccountHero from './components/AccountHero.svelte';
	import IdentitySection from './components/IdentitySection.svelte';
	import EmailVerificationSection from './components/EmailVerificationSection.svelte';
	import SecuritySection from './components/SecuritySection.svelte';
	import DataExportSection from './components/DataExportSection.svelte';
	import DeleteAccountDialog from './components/DeleteAccountDialog.svelte';
	import './components/account-shared.css';

	const auth = getAuth();

	let displayName = $state(auth.user?.display_name ?? '');
	let avatarPreview = $state<string>(auth.user?.avatar_url ?? '');
	let pendingAvatarFile = $state<File | null>(null);
	let saving = $state(false);
	let showSaved = $state(false);
	let saveError = $state('');

	let pwOpen = $state(false);
	let pwCurrent = $state('');
	let pwNew = $state('');
	let pwConfirm = $state('');
	let pwSaving = $state(false);
	let pwError = $state('');
	let pwSuccess = $state(false);

	const pwMismatch = $derived(pwConfirm.length > 0 && pwNew !== pwConfirm);
	const pwCanSubmit = $derived(
		!pwSaving &&
			pwCurrent.length > 0 &&
			pwNew.length >= 12 &&
			pwConfirm.length > 0 &&
			pwNew === pwConfirm
	);

	let showDeleteModal = $state(false);
	let deleteConfirmEmail = $state('');
	let deleting = $state(false);
	let deleteError = $state('');

	function snapshot() {
		return createAccountSnapshot({
			displayName,
			hasAvatar: !!avatarPreview,
			hasPendingAvatar: !!pendingAvatarFile
		});
	}

	let savedSnapshot = $state(snapshot());
	let isDirty = $derived(snapshot() !== savedSnapshot);

	$effect(() => {
		const user = auth.user;
		if (user) {
			untrack(() => {
				displayName = user.display_name;
				if (!pendingAvatarFile) avatarPreview = user.avatar_url ?? '';
				savedSnapshot = snapshot();
			});
		}
	});

	const username = $derived(getAccountUsername(auth.user?.email));
	const avatarInitial = $derived(
		getAccountAvatarInitial({
			displayName: auth.user?.display_name,
			email: auth.user?.email
		})
	);
	const memberSince = $derived(formatMemberSince(auth.user?.created_at));
	const deleteEmailMatches = $derived(isDeleteEmailConfirmed(deleteConfirmEmail, auth.user?.email));

	function handleFileChange(e: Event) {
		const target = e.target as HTMLInputElement;
		const file = target.files?.[0];
		if (!file) return;

		if (!['image/jpeg', 'image/png', 'image/webp'].includes(file.type)) {
			saveError = 'Please select a JPEG, PNG, or WebP image';
			return;
		}
		if (file.size > MAX_AVATAR_SIZE_BYTES) {
			saveError = 'Image must be smaller than 2 MB';
			return;
		}

		saveError = '';
		pendingAvatarFile = file;
		if (avatarPreview.startsWith('blob:')) URL.revokeObjectURL(avatarPreview);
		avatarPreview = URL.createObjectURL(file);
	}

	function discard() {
		displayName = auth.user?.display_name ?? '';
		pendingAvatarFile = null;
		avatarPreview = auth.user?.avatar_url ?? '';
		saveError = '';
		savedSnapshot = snapshot();
	}

	async function save() {
		saving = true;
		saveError = '';

		if (pendingAvatarFile) {
			const result = await uploadAvatar(pendingAvatarFile);
			if (!result.success) {
				switch (result.error.code) {
					case 'invalid_type':
						saveError = 'Unsupported image type';
						break;
					case 'too_large':
						saveError = 'Image must be smaller than 2 MB';
						break;
					default:
						saveError = result.error.message;
				}
				saving = false;
				return;
			}
			pendingAvatarFile = null;
		}

		const result = await auth.updateProfile({
			display_name: displayName.trim() || undefined
		});

		if (result.success) {
			savedSnapshot = snapshot();
			showSaved = true;
			setTimeout(() => {
				showSaved = false;
			}, 2000);
		} else {
			saveError = result.error ?? 'Update failed';
		}
		saving = false;
	}

	function openPwReveal() {
		pwOpen = true;
	}

	function cancelPwReveal() {
		pwOpen = false;
		pwCurrent = '';
		pwNew = '';
		pwConfirm = '';
		pwError = '';
		pwSuccess = false;
	}

	async function handlePasswordChange() {
		if (pwNew !== pwConfirm) {
			pwError = 'New passwords do not match';
			return;
		}
		if (pwNew.length < 12) {
			pwError = 'Password must be at least 12 characters';
			return;
		}
		pwSaving = true;
		pwError = '';
		pwSuccess = false;
		const result = await auth.changePassword(pwCurrent, pwNew);
		if (result.success) {
			pwCurrent = '';
			pwNew = '';
			pwConfirm = '';
			pwSuccess = true;
		} else {
			pwError = result.error ?? 'Password change failed';
		}
		pwSaving = false;
	}

	function closeDeleteModal() {
		showDeleteModal = false;
		deleteConfirmEmail = '';
		deleteError = '';
	}

	async function handleDeleteAccount() {
		if (!deleteEmailMatches) {
			deleteError = 'Email does not match';
			return;
		}
		deleting = true;
		deleteError = '';
		const result = await auth.deleteAccount(deleteConfirmEmail);
		if (result.success) {
			window.location.href = '/login';
		} else {
			deleteError = result.error ?? 'Account deletion failed';
			deleting = false;
		}
	}
</script>

<div class="account-content">
	<AccountHero
		{avatarPreview}
		{avatarInitial}
		displayName={auth.user?.display_name}
		{username}
		{memberSince}
		emailVerified={auth.user?.email_verified}
		onFileChange={handleFileChange}
	/>

	<div class="body-area">
		<IdentitySection
			{displayName}
			{username}
			onDisplayNameChange={(value) => (displayName = value)}
		/>

		<EmailVerificationSection
			email={auth.user?.email ?? ''}
			emailVerified={auth.user?.email_verified}
		/>

		<SecuritySection
			passwordOpen={pwOpen}
			currentPassword={pwCurrent}
			newPassword={pwNew}
			confirmPassword={pwConfirm}
			passwordMismatch={pwMismatch}
			canSubmitPassword={pwCanSubmit}
			passwordSaving={pwSaving}
			passwordError={pwError}
			passwordSuccess={pwSuccess}
			onOpenPassword={openPwReveal}
			onCancelPassword={cancelPwReveal}
			onCurrentPasswordChange={(value) => (pwCurrent = value)}
			onNewPasswordChange={(value) => (pwNew = value)}
			onConfirmPasswordChange={(value) => (pwConfirm = value)}
			onChangePassword={handlePasswordChange}
		/>

		<DataExportSection />

		<SettingsGroup title="Danger zone" danger>
			<div class="group-card danger">
				<div class="row">
					<div class="label-block">
						<div class="label">Delete your account</div>
						<div class="hint">
							Permanently delete your account and every item, highlight, and collection within it.
							This cannot be undone.
						</div>
					</div>
					<div>
						<button
							type="button"
							class="btn danger"
							onclick={() => {
								showDeleteModal = true;
								deleteError = '';
							}}
						>
							Delete account
						</button>
					</div>
				</div>
			</div>
		</SettingsGroup>

		{#if saveError}
			<p class="save-error">{saveError}</p>
		{/if}

		<SavePill {isDirty} {saving} {showSaved} onSave={save} onDiscard={discard} />
	</div>
</div>

{#if showDeleteModal}
	<DeleteAccountDialog
		email={auth.user?.email ?? ''}
		confirmEmail={deleteConfirmEmail}
		{deleteEmailMatches}
		{deleting}
		error={deleteError}
		onClose={closeDeleteModal}
		onConfirmEmailChange={(value) => (deleteConfirmEmail = value)}
		onDelete={handleDeleteAccount}
	/>
{/if}
