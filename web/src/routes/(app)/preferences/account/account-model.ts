export interface AccountSnapshotInput {
	displayName: string;
	bio: string;
	hasAvatar: boolean;
	hasPendingAvatar: boolean;
}

export function createAccountSnapshot(input: AccountSnapshotInput): string {
	return JSON.stringify({
		displayName: input.displayName,
		bio: input.bio,
		hasAvatar: input.hasAvatar,
		hasPendingAvatar: input.hasPendingAvatar
	});
}

export function getAccountUsername(email: string | null | undefined): string {
	return email ? `@${email.split('@')[0]}` : '';
}

export function getAccountAvatarInitial({
	displayName,
	email
}: {
	displayName: string | null | undefined;
	email: string | null | undefined;
}): string {
	return (displayName?.[0] ?? email?.[0] ?? 'U').toUpperCase();
}

export function formatMemberSince(iso: string | null | undefined): string {
	if (!iso) return '';
	const date = new Date(iso);
	if (Number.isNaN(date.getTime())) return '';
	return date.toLocaleDateString(undefined, { month: 'short', year: 'numeric' });
}

export function isDeleteEmailConfirmed(
	confirmEmail: string,
	accountEmail: string | null | undefined
): boolean {
	return (
		confirmEmail.trim().toLowerCase() === (accountEmail ?? '').toLowerCase() &&
		confirmEmail.length > 0
	);
}
