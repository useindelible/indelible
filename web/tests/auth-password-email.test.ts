import { describe, it, expect, beforeEach, vi } from 'vitest';
import { flushSync } from 'svelte';
import { createApiModuleMock } from './helpers/api-module-mock';

const mockUser = {
	id: 'usr_01924b6e-5c3a-7d4f-8e6b-a1b2c3d4e5f6',
	email: 'test@example.com',
	display_name: 'Test User',
	avatar_url: null,
	email_verified: false,
	onboarding_completed: false,
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

describe('auth store: forgotPassword', () => {
	beforeEach(() => {
		vi.clearAllMocks();
	});

	it('returns success on 200 response', async () => {
		mockGET.mockResolvedValue({
			data: undefined,
			error: { error: 'Unauthorized' },
			response: new Response(null, { status: 401 })
		} as never);

		const auth = getAuth();
		await auth.initialize();

		mockPOST.mockResolvedValue({
			data: { message: 'If an account exists, a reset email has been sent.' },
			error: undefined,
			response: new Response(null, { status: 200 })
		} as never);

		const result = await auth.forgotPassword('test@example.com');
		flushSync();

		expect(result.success).toBe(true);
		expect(auth.error).toBeNull();
	});

	it('returns success regardless of whether email exists', async () => {
		mockGET.mockResolvedValue({
			data: undefined,
			error: { error: 'Unauthorized' },
			response: new Response(null, { status: 401 })
		} as never);

		const auth = getAuth();
		await auth.initialize();

		mockPOST.mockResolvedValue({
			data: { message: 'If an account exists, a reset email has been sent.' },
			error: undefined,
			response: new Response(null, { status: 200 })
		} as never);

		const result = await auth.forgotPassword('nonexistent@example.com');
		flushSync();

		expect(result.success).toBe(true);
	});

	it('sets error on rate limit (429)', async () => {
		mockGET.mockResolvedValue({
			data: undefined,
			error: { error: 'Unauthorized' },
			response: new Response(null, { status: 401 })
		} as never);

		const auth = getAuth();
		await auth.initialize();

		mockPOST.mockResolvedValue({
			data: undefined,
			error: { error: 'Too many requests' },
			response: new Response(null, { status: 429 })
		} as never);

		const result = await auth.forgotPassword('test@example.com');
		flushSync();

		expect(result.success).toBe(false);
		expect(auth.error).toBe('Too many requests. Please try again later.');
	});

	it('sets error on network failure', async () => {
		mockGET.mockResolvedValue({
			data: undefined,
			error: { error: 'Unauthorized' },
			response: new Response(null, { status: 401 })
		} as never);

		const auth = getAuth();
		await auth.initialize();

		mockPOST.mockRejectedValue(new Error('Network error'));

		const result = await auth.forgotPassword('test@example.com');
		flushSync();

		expect(result.success).toBe(false);
		expect(auth.error).toBe('An unexpected error occurred');
	});
});

describe('auth store: resetPassword', () => {
	beforeEach(() => {
		vi.clearAllMocks();
	});

	it('returns success on valid token and password', async () => {
		mockGET.mockResolvedValue({
			data: undefined,
			error: { error: 'Unauthorized' },
			response: new Response(null, { status: 401 })
		} as never);

		const auth = getAuth();
		await auth.initialize();

		mockPOST.mockResolvedValue({
			data: { ...mockUser, email_verified: true, onboarding_completed: true },
			error: undefined,
			response: new Response(null, { status: 200 })
		} as never);

		const result = await auth.resetPassword('valid-token', 'newpassword123');
		flushSync();

		expect(result.success).toBe(true);
		expect(auth.error).toBeNull();
		expect(auth.user).toBeNull();
	});

	it('returns expired flag on expired token error', async () => {
		mockGET.mockResolvedValue({
			data: undefined,
			error: { error: 'Unauthorized' },
			response: new Response(null, { status: 401 })
		} as never);

		const auth = getAuth();
		await auth.initialize();

		mockPOST.mockResolvedValue({
			data: undefined,
			error: { error: 'Token expired' },
			response: new Response(null, { status: 400 })
		} as never);

		const result = await auth.resetPassword('expired-token', 'newpassword123');
		flushSync();

		expect(result.success).toBe(false);
		expect(result.expired).toBe(true);
		expect(auth.error).toBe('Token expired');
	});

	it('returns expired flag on invalid token error', async () => {
		mockGET.mockResolvedValue({
			data: undefined,
			error: { error: 'Unauthorized' },
			response: new Response(null, { status: 401 })
		} as never);

		const auth = getAuth();
		await auth.initialize();

		mockPOST.mockResolvedValue({
			data: undefined,
			error: { error: 'Invalid token' },
			response: new Response(null, { status: 400 })
		} as never);

		const result = await auth.resetPassword('bad-token', 'newpassword123');
		flushSync();

		expect(result.success).toBe(false);
		expect(result.expired).toBe(true);
	});

	it('sets error on network failure', async () => {
		mockGET.mockResolvedValue({
			data: undefined,
			error: { error: 'Unauthorized' },
			response: new Response(null, { status: 401 })
		} as never);

		const auth = getAuth();
		await auth.initialize();

		mockPOST.mockRejectedValue(new Error('Network error'));

		const result = await auth.resetPassword('token', 'newpassword123');
		flushSync();

		expect(result.success).toBe(false);
		expect(auth.error).toBe('An unexpected error occurred');
	});
});

describe('auth store: verifyEmail', () => {
	beforeEach(() => {
		vi.clearAllMocks();
	});

	it('returns success and updates the stored user on valid token', async () => {
		mockGET.mockResolvedValue({
			data: mockSessionResponse(),
			error: undefined,
			response: new Response(null, { status: 200 })
		} as never);

		const auth = getAuth();
		await auth.initialize();

		const verifiedUser = { ...mockUser, email_verified: true };
		mockPOST.mockResolvedValue({
			data: verifiedUser,
			error: undefined,
			response: new Response(null, { status: 200 })
		} as never);

		const result = await auth.verifyEmail('valid-token');
		flushSync();

		expect(result.success).toBe(true);
		expect(auth.user?.email_verified).toBe(true);
	});

	it('returns failure on invalid token', async () => {
		mockGET.mockResolvedValue({
			data: mockSessionResponse(),
			error: undefined,
			response: new Response(null, { status: 200 })
		} as never);

		const auth = getAuth();
		await auth.initialize();

		mockPOST.mockResolvedValue({
			data: undefined,
			error: { error: 'Invalid or expired token' },
			response: new Response(null, { status: 400 })
		} as never);

		const result = await auth.verifyEmail('bad-token');
		flushSync();

		expect(result.success).toBe(false);
		expect(auth.error).toBe('Invalid or expired token');
	});

	it('sets error on network failure', async () => {
		mockGET.mockResolvedValue({
			data: mockSessionResponse(),
			error: undefined,
			response: new Response(null, { status: 200 })
		} as never);

		const auth = getAuth();
		await auth.initialize();

		mockPOST.mockRejectedValue(new Error('Network error'));

		const result = await auth.verifyEmail('token');
		flushSync();

		expect(result.success).toBe(false);
		expect(auth.error).toBe('An unexpected error occurred');
	});
});

describe('auth store: resendVerification', () => {
	beforeEach(() => {
		vi.clearAllMocks();
	});

	it('returns success on 200 response', async () => {
		mockGET.mockResolvedValue({
			data: mockSessionResponse(),
			error: undefined,
			response: new Response(null, { status: 200 })
		} as never);

		const auth = getAuth();
		await auth.initialize();

		mockPOST.mockResolvedValue({
			data: { message: 'Verification email sent.' },
			error: undefined,
			response: new Response(null, { status: 200 })
		} as never);

		const result = await auth.resendVerification();
		flushSync();

		expect(result.success).toBe(true);
		expect(auth.error).toBeNull();
	});

	it('sets error on rate limit (429)', async () => {
		mockGET.mockResolvedValue({
			data: mockSessionResponse(),
			error: undefined,
			response: new Response(null, { status: 200 })
		} as never);

		const auth = getAuth();
		await auth.initialize();

		mockPOST.mockResolvedValue({
			data: undefined,
			error: { error: 'Too many requests' },
			response: new Response(null, { status: 429 })
		} as never);

		const result = await auth.resendVerification();
		flushSync();

		expect(result.success).toBe(false);
		expect(auth.error).toBe('Too many requests. Please try again later.');
	});

	it('sets error on network failure', async () => {
		mockGET.mockResolvedValue({
			data: mockSessionResponse(),
			error: undefined,
			response: new Response(null, { status: 200 })
		} as never);

		const auth = getAuth();
		await auth.initialize();

		mockPOST.mockRejectedValue(new Error('Network error'));

		const result = await auth.resendVerification();
		flushSync();

		expect(result.success).toBe(false);
		expect(auth.error).toBe('An unexpected error occurred');
	});
});
