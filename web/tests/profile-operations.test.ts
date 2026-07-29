import { describe, it, expect, beforeEach, vi } from 'vitest';
import { flushSync } from 'svelte';
import { createApiModuleMock } from './helpers/api-module-mock';

const mockUser = {
	id: 'usr_01924b6e-5c3a-7d4f-8e6b-a1b2c3d4e5f6',
	email: 'test@example.com',
	display_name: 'Test User',
	avatar_url: null,
	email_verified: true,
	onboarding_completed: true,
	has_password: true,
	theme: 'system' as const
};

function mockSessionResponse(user = mockUser) {
	return {
		user,
		session: {
			id: 'ses_01924b6e-5c3a-7d4f-8e6b-a1b2c3d4e5f6',
			client_type: 'web',
			last_active_at: '2026-03-21T00:00:00Z',
			expires_at: '2026-04-20T00:00:00Z',
			created_at: '2026-03-21T00:00:00Z'
		}
	};
}

vi.mock('$lib/api', () => createApiModuleMock());

import { api } from '$lib/api';
import { getAuth } from '$lib/stores/auth.svelte';

const mockGET = vi.mocked(api.GET);
const mockPOST = vi.mocked(api.POST);
const mockPATCH = vi.mocked(api.PATCH);
const mockDELETE = vi.mocked(api.DELETE);

async function initAuthenticatedUser() {
	mockPOST.mockResolvedValueOnce({
		data: {
			...mockUser,
			access_token: 'test-access-token',
			expires_at: 9999999999
		},
		error: undefined,
		response: new Response(null, { status: 200 })
	} as never);

	const auth = getAuth();
	await auth.login('test@example.com', 'password123');
	flushSync();
	return auth;
}

describe('profile operations', () => {
	beforeEach(() => {
		vi.clearAllMocks();
	});

	describe('updateProfile', () => {
		it('calls PATCH /api/v1/me with display_name', async () => {
			const auth = await initAuthenticatedUser();

			mockPATCH.mockResolvedValue({
				data: { ...mockUser, display_name: 'New Name' },
				error: undefined,
				response: new Response(null, { status: 200 })
			} as never);

			mockGET.mockResolvedValue({
				data: mockSessionResponse({ ...mockUser, display_name: 'New Name' }),
				error: undefined,
				response: new Response(null, { status: 200 })
			} as never);

			const result = await auth.updateProfile({ display_name: 'New Name' });
			flushSync();

			expect(result.success).toBe(true);
			expect(mockPATCH).toHaveBeenCalledWith('/api/v1/me', {
				body: { display_name: 'New Name' }
			});
		});

		it('refreshes user data after successful profile update', async () => {
			const auth = await initAuthenticatedUser();

			mockPATCH.mockResolvedValue({
				data: { ...mockUser, display_name: 'Updated' },
				error: undefined,
				response: new Response(null, { status: 200 })
			} as never);

			const updatedUser = { ...mockUser, display_name: 'Updated' };
			mockGET.mockResolvedValue({
				data: mockSessionResponse(updatedUser),
				error: undefined,
				response: new Response(null, { status: 200 })
			} as never);

			await auth.updateProfile({ display_name: 'Updated' });
			flushSync();

			expect(auth.user?.display_name).toBe('Updated');
		});

		it('returns error on failed profile update', async () => {
			const auth = await initAuthenticatedUser();

			mockPATCH.mockResolvedValue({
				data: undefined,
				error: { error: 'Validation failed' },
				response: new Response(null, { status: 422 })
			} as never);

			const result = await auth.updateProfile({ display_name: '' });
			expect(result.success).toBe(false);
			expect(result.error).toBe('Validation failed');
		});

		it('handles network errors during profile update', async () => {
			const auth = await initAuthenticatedUser();

			mockPATCH.mockRejectedValue(new Error('Network error'));

			const result = await auth.updateProfile({ display_name: 'Test' });
			expect(result.success).toBe(false);
			expect(result.error).toBe('An unexpected error occurred');
		});
	});

	describe('changePassword', () => {
		it('calls POST /api/v1/me/password with credentials', async () => {
			const auth = await initAuthenticatedUser();

			mockPOST.mockResolvedValue({
				data: { message: 'password changed' },
				error: undefined,
				response: new Response(null, { status: 200 })
			} as never);

			const result = await auth.changePassword('old-password', 'new-secure-password');

			expect(result.success).toBe(true);
			expect(mockPOST).toHaveBeenCalledWith('/api/v1/me/password', {
				body: {
					current_password: 'old-password',
					new_password: 'new-secure-password'
				}
			});
		});

		it('returns error when current password is wrong', async () => {
			const auth = await initAuthenticatedUser();

			mockPOST.mockResolvedValue({
				data: undefined,
				error: { error: 'Invalid current password' },
				response: new Response(null, { status: 401 })
			} as never);

			const result = await auth.changePassword('wrong-password', 'new-secure-password');

			expect(result.success).toBe(false);
			expect(result.error).toBe('Invalid current password');
		});

		it('handles network errors during password change', async () => {
			const auth = await initAuthenticatedUser();

			mockPOST.mockRejectedValue(new Error('Network error'));

			const result = await auth.changePassword('old', 'new-secure-password');
			expect(result.success).toBe(false);
			expect(result.error).toBe('An unexpected error occurred');
		});
	});

	describe('changeEmail', () => {
		it('calls POST /api/v1/me/email with new email and password', async () => {
			const auth = await initAuthenticatedUser();

			mockPOST.mockResolvedValue({
				data: { message: 'verification email sent to new address' },
				error: undefined,
				response: new Response(null, { status: 200 })
			} as never);

			const result = await auth.changeEmail('new@example.com', 'my-password');

			expect(result.success).toBe(true);
			expect(auth.user).toBeNull();
			expect(mockPOST).toHaveBeenCalledWith('/api/v1/me/email', {
				body: {
					new_email: 'new@example.com',
					password: 'my-password'
				}
			});
		});

		it('returns error when password is invalid', async () => {
			const auth = await initAuthenticatedUser();

			mockPOST.mockResolvedValue({
				data: undefined,
				error: { error: 'Invalid password' },
				response: new Response(null, { status: 401 })
			} as never);

			const result = await auth.changeEmail('new@example.com', 'wrong');

			expect(result.success).toBe(false);
			expect(result.error).toBe('Invalid password');
		});

		it('returns error when email is already in use', async () => {
			const auth = await initAuthenticatedUser();

			mockPOST.mockResolvedValue({
				data: undefined,
				error: { error: 'Email already in use' },
				response: new Response(null, { status: 409 })
			} as never);

			const result = await auth.changeEmail('taken@example.com', 'password');

			expect(result.success).toBe(false);
			expect(result.error).toBe('Email already in use');
		});
	});

	describe('deleteAccount', () => {
		it('calls DELETE /api/v1/me with confirmation', async () => {
			const auth = await initAuthenticatedUser();

			mockDELETE.mockResolvedValue({
				data: undefined,
				error: undefined,
				response: new Response(null, { status: 204 })
			} as never);

			const result = await auth.deleteAccount('my-password');
			flushSync();

			expect(result.success).toBe(true);
			expect(mockDELETE).toHaveBeenCalledWith('/api/v1/me', {
				body: { confirmation: 'my-password' }
			});
			expect(auth.user).toBeNull();
		});

		it('returns error on failed account deletion', async () => {
			const auth = await initAuthenticatedUser();

			mockDELETE.mockResolvedValue({
				data: undefined,
				error: { error: 'Invalid confirmation' },
				response: new Response(null, { status: 400 })
			} as never);

			const result = await auth.deleteAccount('wrong');

			expect(result.success).toBe(false);
			expect(result.error).toBe('Invalid confirmation');
			expect(auth.isAuthenticated).toBe(true);
		});
	});
});
