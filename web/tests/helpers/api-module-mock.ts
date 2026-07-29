import { vi } from 'vitest';

export function createApiModuleMock() {
	const api = {
		GET: vi.fn(),
		POST: vi.fn(),
		PUT: vi.fn(),
		DELETE: vi.fn(),
		getConfig: vi.fn(() => ({ baseUrl: 'http://localhost:38473' })),
		PATCH: vi.fn(),
		HEAD: vi.fn(),
		OPTIONS: vi.fn(),
		TRACE: vi.fn(),
		request: vi.fn(),
		use: vi.fn(),
		eject: vi.fn()
	};

	const completeStep = vi.fn((options: unknown) =>
		api.POST('/api/v1/onboarding/steps/{step}/complete', options)
	);
	const testConfig = vi.fn((options: unknown) => api.POST('/api/v1/mila/config/test', options));
	const subscribe = vi.fn((options: unknown) => api.POST('/api/v1/feeds/subscriptions', options));

	return {
		AUTH_REDIRECT_SUPPRESSION_HEADER: 'X-Suppress-Auth-Redirect',
		api,
		changeEmail: (options: unknown) => api.POST('/api/v1/me/email', options),
		changePassword: (options: unknown) => api.POST('/api/v1/me/password', options),
		completeStep,
		createToken: (options: unknown) => api.POST('/api/v1/tokens', options),
		deleteAccount: (options: unknown) => api.DELETE('/api/v1/me', options),
		extensionStatus: (options?: unknown) =>
			options === undefined
				? api.GET('/api/v1/extension/status')
				: api.GET('/api/v1/extension/status', options),
		forgotPassword: (options: unknown) => api.POST('/api/v1/auth/password/forgot', options),
		getOnboarding: (options?: unknown) =>
			options === undefined
				? api.GET('/api/v1/onboarding')
				: api.GET('/api/v1/onboarding', options),
		getProfile: (options?: unknown) =>
			options === undefined ? api.GET('/api/v1/me') : api.GET('/api/v1/me', options),
		getSession: (options?: unknown) =>
			options === undefined
				? api.GET('/api/v1/auth/session')
				: api.GET('/api/v1/auth/session', options),
		listProviders: (options?: unknown) =>
			options === undefined
				? api.GET('/api/v1/auth/providers')
				: api.GET('/api/v1/auth/providers', options),
		listTokens: (options?: unknown) =>
			options === undefined ? api.GET('/api/v1/tokens') : api.GET('/api/v1/tokens', options),
		login: (options: unknown) => api.POST('/api/v1/auth/login', options),
		logout: (options?: unknown) =>
			options === undefined
				? api.POST('/api/v1/auth/logout')
				: api.POST('/api/v1/auth/logout', options),
		register: (options: unknown) => api.POST('/api/v1/auth/register', options),
		refresh: (options?: unknown) =>
			options === undefined
				? api.POST('/api/v1/auth/refresh')
				: api.POST('/api/v1/auth/refresh', options),
		resendVerification: (options?: unknown) =>
			options === undefined
				? api.POST('/api/v1/auth/email/resend')
				: api.POST('/api/v1/auth/email/resend', options),
		resetPassword: (options: unknown) => api.POST('/api/v1/auth/password/reset', options),
		revokeToken: (options: unknown) => api.DELETE('/api/v1/tokens/{token_id}', options),
		skipOnboarding: (options?: unknown) =>
			options === undefined
				? api.POST('/api/v1/onboarding/skip')
				: api.POST('/api/v1/onboarding/skip', options),
		subscribe,
		testConfig,
		updateProfile: (options: unknown) => api.PATCH('/api/v1/me', options),
		verifyEmail: (options: unknown) => api.POST('/api/v1/auth/email/verify', options)
	};
}
