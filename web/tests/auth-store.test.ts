import { describe, it, expect, beforeEach, vi } from 'vitest';
import { flushSync } from 'svelte';
import { createApiModuleMock } from './helpers/api-module-mock';

vi.mock('$lib/api', () => createApiModuleMock());

const mockLoginData = {
	id: 'usr_01924b6e-5c3a-7d4f-8e6b-a1b2c3d4e5f6',
	email: 'test@example.com',
	display_name: 'Test User',
	avatar_url: null,
	email_verified: true,
	onboarding_completed: true,
	access_token: 'test-access-token',
	expires_at: 9999999999
};

const mockProfile = {
	id: 'usr_01924b6e-5c3a-7d4f-8e6b-a1b2c3d4e5f6',
	email: 'test@example.com',
	display_name: 'Test User',
	avatar_url: null,
	email_verified: true,
	onboarding_completed: true
};

describe('auth store', () => {
	beforeEach(() => {
		vi.resetModules();
	});

	async function load() {
		const apiModule = await import('$lib/api');
		const authModule = await import('$lib/stores/auth.svelte');
		return {
			getAuth: authModule.getAuth,
			mockGET: vi.mocked(apiModule.api.GET),
			mockPOST: vi.mocked(apiModule.api.POST)
		};
	}

	it('starts with loading true before initialization', async () => {
		const { getAuth } = await load();
		expect(getAuth().loading).toBe(true);
	});

	it('isAuthenticated is false when no user', async () => {
		const { getAuth } = await load();
		const auth = getAuth();
		await auth.initialize();
		flushSync();
		expect(auth.isAuthenticated).toBe(false);
		expect(auth.user).toBeNull();
		expect(auth.loading).toBe(false);
	});

	it('populates user and isAuthenticated on successful initialize', async () => {
		const { getAuth, mockPOST, mockGET } = await load();
		mockPOST.mockResolvedValueOnce({
			data: { access_token: 'test-token', expires_at: 9999999999 },
			error: undefined,
			response: new Response(null, { status: 200 })
		} as never);
		mockGET.mockResolvedValueOnce({
			data: mockProfile,
			error: undefined,
			response: new Response(null, { status: 200 })
		} as never);

		const auth = getAuth();
		await auth.initialize();
		flushSync();

		expect(auth.isAuthenticated).toBe(true);
		expect(auth.user?.email).toBe('test@example.com');
		expect(auth.user?.display_name).toBe('Test User');
		expect(auth.loading).toBe(false);
	});

	it('handles login success', async () => {
		const { getAuth, mockPOST } = await load();

		mockPOST.mockResolvedValueOnce({
			data: mockLoginData,
			error: undefined,
			response: new Response(null, { status: 200 })
		} as never);

		const auth = getAuth();
		const result = await auth.login('test@example.com', 'password123');
		flushSync();

		expect(result.success).toBe(true);
		expect(auth.isAuthenticated).toBe(true);
		expect(auth.user?.email).toBe('test@example.com');
	});

	it('replaces invalid-credential details with user-facing login copy', async () => {
		const { getAuth, mockPOST } = await load();

		mockPOST.mockResolvedValueOnce({
			data: undefined,
			error: { detail: 'Invalid credentials' },
			response: new Response(null, { status: 401 })
		} as never);

		const auth = getAuth();
		const result = await auth.login('test@example.com', 'wrong');
		flushSync();

		expect(result.success).toBe(false);
		expect(auth.isAuthenticated).toBe(false);
		expect(auth.error).toBe('Email or password is incorrect.');
	});

	it('preserves non-credential login problem details', async () => {
		const { getAuth, mockPOST } = await load();

		mockPOST.mockResolvedValueOnce({
			data: undefined,
			error: { detail: 'Account disabled' },
			response: new Response(null, { status: 403 })
		} as never);

		const auth = getAuth();
		const result = await auth.login('test@example.com', 'password123');
		flushSync();

		expect(result.success).toBe(false);
		expect(auth.error).toBe('Account disabled');
	});

	it('preserves rate-limit cooldown metadata and copy', async () => {
		const { getAuth, mockPOST } = await load();

		mockPOST.mockResolvedValueOnce({
			data: undefined,
			error: { detail: 'rate limited' },
			response: new Response(null, {
				status: 429,
				headers: { 'Retry-After': '45' }
			})
		} as never);

		const auth = getAuth();
		const result = await auth.login('test@example.com', 'password123');
		flushSync();

		expect(result).toEqual({ success: false, rateLimited: true, retryAfter: 45 });
		expect(auth.error).toBe('Too many login attempts. Please try again later.');
	});

	it('handles register success', async () => {
		const { getAuth, mockPOST } = await load();

		const newUser = { ...mockLoginData, email_verified: false };
		mockPOST.mockResolvedValueOnce({
			data: newUser,
			error: undefined,
			response: new Response(null, { status: 201 })
		} as never);

		const auth = getAuth();
		const result = await auth.register('new@example.com', 'password123', 'New User');
		flushSync();

		expect(result.success).toBe(true);
		expect(auth.isAuthenticated).toBe(true);
	});

	it('handles register failure', async () => {
		const { getAuth, mockPOST } = await load();

		mockPOST.mockResolvedValueOnce({
			data: undefined,
			error: { detail: 'Email already registered' },
			response: new Response(null, { status: 409 })
		} as never);

		const auth = getAuth();
		const result = await auth.register('taken@example.com', 'password123', 'Taken');
		flushSync();

		expect(result.success).toBe(false);
		expect(auth.error).toBe('Email already registered');
	});

	it('clears user on logout', async () => {
		const { getAuth, mockPOST } = await load();

		mockPOST.mockResolvedValueOnce({
			data: mockLoginData,
			error: undefined,
			response: new Response(null, { status: 200 })
		} as never);

		const auth = getAuth();
		await auth.login('test@example.com', 'password123');
		expect(auth.isAuthenticated).toBe(true);

		mockPOST.mockResolvedValueOnce({
			data: undefined,
			error: undefined,
			response: new Response(null, { status: 204 })
		} as never);

		await auth.logout();
		flushSync();

		expect(auth.isAuthenticated).toBe(false);
		expect(auth.user).toBeNull();
	});

	it('needsVerification is true when user exists with unverified email', async () => {
		const { getAuth, mockPOST } = await load();

		mockPOST.mockResolvedValueOnce({
			data: { ...mockLoginData, email_verified: false },
			error: undefined,
			response: new Response(null, { status: 200 })
		} as never);

		const auth = getAuth();
		await auth.login('test@example.com', 'password123');
		flushSync();

		expect(auth.needsVerification).toBe(true);
	});

	it('needsVerification is false when user has verified email', async () => {
		const { getAuth, mockPOST } = await load();

		mockPOST.mockResolvedValueOnce({
			data: mockLoginData,
			error: undefined,
			response: new Response(null, { status: 200 })
		} as never);

		const auth = getAuth();
		await auth.login('test@example.com', 'password123');
		flushSync();

		expect(auth.needsVerification).toBe(false);
	});

	it('needsOnboarding is true when verified but onboarding not completed', async () => {
		const { getAuth, mockPOST } = await load();

		mockPOST.mockResolvedValueOnce({
			data: { ...mockLoginData, email_verified: true, onboarding_completed: false },
			error: undefined,
			response: new Response(null, { status: 200 })
		} as never);

		const auth = getAuth();
		await auth.login('test@example.com', 'password123');
		flushSync();

		expect(auth.needsOnboarding).toBe(true);
	});

	it('needsOnboarding is false when onboarding is completed', async () => {
		const { getAuth, mockPOST } = await load();

		mockPOST.mockResolvedValueOnce({
			data: mockLoginData,
			error: undefined,
			response: new Response(null, { status: 200 })
		} as never);

		const auth = getAuth();
		await auth.login('test@example.com', 'password123');
		flushSync();

		expect(auth.needsOnboarding).toBe(false);
	});

	it('refresh reconciles profile-derived auth state', async () => {
		const { getAuth, mockGET, mockPOST } = await load();

		mockPOST.mockResolvedValueOnce({
			data: { ...mockLoginData, onboarding_completed: false },
			error: undefined,
			response: new Response(null, { status: 200 })
		} as never);

		const auth = getAuth();
		await auth.login('test@example.com', 'password123');

		mockPOST.mockResolvedValueOnce({
			data: { access_token: 'new-token', expires_at: 9999999999 },
			error: undefined,
			response: new Response(null, { status: 200 })
		} as never);
		mockGET.mockResolvedValueOnce({
			data: { ...mockProfile, display_name: 'Updated User', onboarding_completed: true },
			error: undefined,
			response: new Response(null, { status: 200 })
		} as never);

		const refreshed = await auth.refresh();
		flushSync();

		expect(refreshed).toBe(true);
		expect(auth.isAuthenticated).toBe(true);
		expect(auth.user?.email).toBe('test@example.com');
		expect(auth.user?.display_name).toBe('Updated User');
		expect(auth.needsOnboarding).toBe(false);
	});

	it('refresh reports a profile reconciliation failure', async () => {
		const { getAuth, mockGET, mockPOST } = await load();

		mockPOST.mockResolvedValueOnce({
			data: { ...mockLoginData, onboarding_completed: false },
			error: undefined,
			response: new Response(null, { status: 200 })
		} as never);

		const auth = getAuth();
		await auth.login('test@example.com', 'password123');

		mockPOST.mockResolvedValueOnce({
			data: { access_token: 'new-token', expires_at: 9999999999 },
			error: undefined,
			response: new Response(null, { status: 200 })
		} as never);
		mockGET.mockResolvedValueOnce({
			data: undefined,
			error: { detail: 'Unavailable' },
			response: new Response(null, { status: 503 })
		} as never);

		const refreshed = await auth.refresh();
		flushSync();

		expect(refreshed).toBe(false);
		expect(auth.needsOnboarding).toBe(true);
		expect(auth.error).toBe('Unable to refresh your profile. Please try again.');
	});

	it('handles network errors during initialize gracefully', async () => {
		const { getAuth, mockPOST } = await load();

		mockPOST.mockRejectedValueOnce(new Error('Network error'));

		const auth = getAuth();
		await auth.initialize();
		flushSync();

		expect(auth.isAuthenticated).toBe(false);
		expect(auth.user).toBeNull();
		expect(auth.loading).toBe(false);
	});

	it('handles network errors during login gracefully', async () => {
		const { getAuth, mockPOST } = await load();

		mockPOST.mockRejectedValueOnce(new Error('Network error'));

		const auth = getAuth();
		const result = await auth.login('test@example.com', 'password123');
		flushSync();

		expect(result.success).toBe(false);
		expect(auth.error).toBe('An unexpected error occurred');
	});
});
