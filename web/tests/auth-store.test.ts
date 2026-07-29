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

	it('handles login failure', async () => {
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
		expect(auth.error).toBe('Invalid credentials');
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

	it('refresh does not clear authenticated user', async () => {
		const { getAuth, mockPOST } = await load();

		mockPOST.mockResolvedValueOnce({
			data: mockLoginData,
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

		await auth.refresh();
		flushSync();

		expect(auth.isAuthenticated).toBe(true);
		expect(auth.user?.email).toBe('test@example.com');
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
