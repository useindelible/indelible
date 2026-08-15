import { describe, expect, it } from 'vitest';
import {
	createAccountSnapshot,
	formatMemberSince,
	getAccountAvatarInitial,
	getAccountUsername,
	isDeleteEmailConfirmed
} from '../../src/routes/(app)/preferences/account/account-model';

describe('account settings model', () => {
	it('derives the public username from the account email', () => {
		expect(getAccountUsername('sam@example.com')).toBe('@sam');
		expect(getAccountUsername(null)).toBe('');
	});

	it('uses display name before email for the avatar initial', () => {
		expect(getAccountAvatarInitial({ displayName: 'Mila Stone', email: 'mila@example.com' })).toBe(
			'M'
		);
		expect(getAccountAvatarInitial({ displayName: '', email: 'reader@example.com' })).toBe('R');
		expect(getAccountAvatarInitial({ displayName: '', email: null })).toBe('U');
	});

	it('formats member-since labels defensively', () => {
		expect(formatMemberSince('2026-04-10T12:00:00.000Z')).toBe('Apr 2026');
		expect(formatMemberSince(null)).toBe('');
		expect(formatMemberSince('not-a-date')).toBe('');
	});

	it('keeps the dirty snapshot stable for profile edits', () => {
		expect(
			createAccountSnapshot({
				displayName: 'Sam',
				hasAvatar: true,
				hasPendingAvatar: false
			})
		).toBe(
			JSON.stringify({
				displayName: 'Sam',
				hasAvatar: true,
				hasPendingAvatar: false
			})
		);
	});

	it('requires an exact email confirmation ignoring case and surrounding whitespace', () => {
		expect(isDeleteEmailConfirmed(' USER@example.com ', 'user@example.com')).toBe(true);
		expect(isDeleteEmailConfirmed('', 'user@example.com')).toBe(false);
		expect(isDeleteEmailConfirmed('other@example.com', 'user@example.com')).toBe(false);
	});
});
