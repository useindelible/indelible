import * as apiSdk from '$lib/api';
import { setAccessToken as setSdkToken } from '$lib/auth-tokens';

export type AuthUser = {
	id: string;
	email: string;
	display_name: string;
	email_verified: boolean;
	onboarding_completed: boolean;
	avatar_url?: string | null;
	locale?: string;
	theme?: 'light' | 'dark' | 'system';
	timezone?: string;
	ingest_email?: string;
	ingest_library_email?: string;
	created_at?: string;
};

let user = $state<AuthUser | null>(null);
let loading = $state(true);
let error = $state<string | null>(null);
let accessToken = $state<string | null>(null);
let expiresAt = $state<number | null>(null);
let refreshPromise: Promise<void> | null = null;
let refreshTimer: ReturnType<typeof setTimeout> | null = null;
let initialized = false;
let initPromise: Promise<void> | null = null;

type ApiProblem = {
	detail?: string;
	error?: string;
	message?: string;
};

function updateAccessToken(token: string | null) {
	accessToken = token;
	setSdkToken(token);
}

const isAuthenticated = $derived(user !== null);
const needsVerification = $derived(user !== null && !user.email_verified);
const needsOnboarding = $derived(
	user !== null && user.email_verified && !user.onboarding_completed
);

const AUTH_CHANNEL =
	typeof BroadcastChannel !== 'undefined' ? new BroadcastChannel('indelible:auth') : null;

if (AUTH_CHANNEL) {
	AUTH_CHANNEL.onmessage = (event) => {
		const msg = event.data;
		if (msg.type === 'logout') {
			user = null;
			updateAccessToken(null);
			expiresAt = null;
			clearRefreshTimer();
			if (typeof window !== 'undefined' && window.location.pathname !== '/login') {
				window.location.href = '/login';
			}
		} else if (msg.type === 'refreshed' && msg.accessToken) {
			updateAccessToken(msg.accessToken);
			expiresAt = msg.expiresAt;
			scheduleRefresh();
		}
	};
}

function clearRefreshTimer() {
	if (refreshTimer) {
		clearTimeout(refreshTimer);
		refreshTimer = null;
	}
}

function scheduleRefresh() {
	clearRefreshTimer();
	if (!expiresAt) return;
	const msUntilExpiry = expiresAt * 1000 - Date.now();
	const refreshIn = Math.max(msUntilExpiry - 2 * 60 * 1000, 0);
	refreshTimer = setTimeout(() => {
		doRefresh();
	}, refreshIn);
}

async function doRefresh(): Promise<void> {
	if (refreshPromise) return refreshPromise;

	refreshPromise = (async () => {
		try {
			const { data } = await apiSdk.refresh();
			if (data) {
				updateAccessToken(data.access_token);
				expiresAt = data.expires_at;
				scheduleRefresh();
				AUTH_CHANNEL?.postMessage({
					type: 'refreshed',
					accessToken: data.access_token,
					expiresAt: data.expires_at
				});
			} else {
				user = null;
				updateAccessToken(null);
				expiresAt = null;
			}
		} catch {
			user = null;
			updateAccessToken(null);
			expiresAt = null;
		} finally {
			refreshPromise = null;
		}
	})();

	return refreshPromise;
}

export function getAuth() {
	return {
		get user() {
			return user;
		},
		get loading() {
			return loading;
		},
		get error() {
			return error;
		},
		get isAuthenticated() {
			return isAuthenticated;
		},
		get needsVerification() {
			return needsVerification;
		},
		get needsOnboarding() {
			return needsOnboarding;
		},
		get accessToken() {
			return accessToken;
		},
		initialize,
		login,
		register,
		logout,
		refresh,
		forgotPassword,
		resetPassword,
		verifyEmail,
		resendVerification,
		updateProfile,
		changePassword,
		changeEmail,
		deleteAccount
	};
}

function getProblemMessage(problem: unknown, fallback: string): string {
	if (!problem || typeof problem !== 'object') {
		return fallback;
	}

	const candidate = problem as ApiProblem;
	return candidate.detail ?? candidate.message ?? candidate.error ?? fallback;
}

function applyProfile(data: apiSdk.ProfileResponse) {
	user = {
		id: data.id,
		email: data.email,
		display_name: data.display_name,
		email_verified: data.email_verified,
		onboarding_completed: data.onboarding_completed,
		avatar_url: data.avatar_url,
		locale: data.locale,
		theme: data.theme as 'light' | 'dark' | 'system' | undefined,
		timezone: data.timezone,
		ingest_email: data.ingest_email ?? undefined,
		ingest_library_email: data.ingest_library_email ?? undefined,
		created_at: data.created_at
	};
}

async function fetchAndApplyProfile(): Promise<boolean> {
	try {
		const { data } = await apiSdk.getProfile();
		if (!data) return false;
		applyProfile(data);
		return true;
	} catch {
		// non-fatal — caller has already set minimal user state
		return false;
	}
}

async function initialize() {
	if (initialized) return;
	if (initPromise) return initPromise;

	initPromise = (async () => {
		loading = true;
		error = null;
		try {
			await doRefresh();
			if (accessToken) {
				const { data } = await apiSdk.getProfile();
				if (data) {
					applyProfile(data);
				}
			}
		} catch {
			user = null;
			updateAccessToken(null);
			expiresAt = null;
		} finally {
			loading = false;
			initialized = true;
			initPromise = null;
		}
	})();

	return initPromise;
}

async function login(
	email: string,
	password: string
): Promise<{ success: boolean; rateLimited?: boolean; retryAfter?: number }> {
	error = null;
	try {
		const {
			data,
			error: apiError,
			response
		} = await apiSdk.login({
			body: { email, password }
		});
		if (data) {
			updateAccessToken(data.access_token ?? null);
			expiresAt = data.expires_at ?? null;
			scheduleRefresh();
			user = {
				id: data.id,
				email: data.email,
				display_name: data.display_name,
				email_verified: data.email_verified,
				onboarding_completed: data.onboarding_completed,
				theme: undefined,
				locale: undefined,
				timezone: undefined
			};
			await fetchAndApplyProfile();
			AUTH_CHANNEL?.postMessage({ type: 'login' });
			return { success: true };
		}
		if (response?.status === 429) {
			const retryAfter = parseInt(response.headers.get('Retry-After') ?? '30', 10);
			error = 'Too many login attempts. Please try again later.';
			return { success: false, rateLimited: true, retryAfter };
		}
		error =
			response?.status === 401
				? 'Email or password is incorrect.'
				: getProblemMessage(apiError, 'Login failed');
		return { success: false };
	} catch {
		error = 'An unexpected error occurred';
		return { success: false };
	}
}

async function register(
	email: string,
	password: string,
	displayName: string
): Promise<{ success: boolean }> {
	error = null;
	try {
		const { data, error: apiError } = await apiSdk.register({
			body: { email, password, display_name: displayName }
		});
		if (data) {
			updateAccessToken(data.access_token ?? null);
			expiresAt = data.expires_at ?? null;
			scheduleRefresh();
			user = {
				id: data.id,
				email: data.email,
				display_name: data.display_name,
				email_verified: data.email_verified,
				onboarding_completed: data.onboarding_completed,
				theme: undefined,
				locale: undefined,
				timezone: undefined
			};
			await fetchAndApplyProfile();
			AUTH_CHANNEL?.postMessage({ type: 'login' });
			return { success: true };
		}
		error = getProblemMessage(apiError, 'Registration failed');
		return { success: false };
	} catch {
		error = 'An unexpected error occurred';
		return { success: false };
	}
}

async function logout(): Promise<void> {
	try {
		await apiSdk.logout();
	} finally {
		user = null;
		updateAccessToken(null);
		expiresAt = null;
		clearRefreshTimer();
		AUTH_CHANNEL?.postMessage({ type: 'logout' });
	}
}

async function refresh(): Promise<boolean> {
	error = null;
	await doRefresh();
	if (!accessToken) {
		error = 'Unable to refresh your session. Please try again.';
		return false;
	}
	if (!(await fetchAndApplyProfile())) {
		error = 'Unable to refresh your profile. Please try again.';
		return false;
	}
	return true;
}

async function forgotPassword(email: string): Promise<{ success: boolean }> {
	error = null;
	try {
		const { data, response } = await apiSdk.forgotPassword({
			body: { email }
		});
		if (data) {
			return { success: true };
		}
		if (response?.status === 429) {
			error = 'Too many requests. Please try again later.';
			return { success: false };
		}
		return { success: true };
	} catch {
		error = 'An unexpected error occurred';
		return { success: false };
	}
}

async function resetPassword(
	token: string,
	password: string
): Promise<{ success: boolean; expired?: boolean }> {
	error = null;
	try {
		const { data, error: apiError } = await apiSdk.resetPassword({
			body: { token, new_password: password }
		});
		if (data) {
			user = null;
			updateAccessToken(null);
			expiresAt = null;
			return { success: true };
		}
		const msg = getProblemMessage(apiError, 'Password reset failed');
		error = msg;
		const isExpired =
			msg.toLowerCase().includes('expired') || msg.toLowerCase().includes('invalid');
		return { success: false, expired: isExpired };
	} catch {
		error = 'An unexpected error occurred';
		return { success: false };
	}
}

async function verifyEmail(token: string): Promise<{ success: boolean }> {
	error = null;
	try {
		const { data, error: apiError } = await apiSdk.verifyEmail({
			body: { token }
		});
		if (data) {
			user = {
				id: data.id,
				email: data.email,
				display_name: data.display_name,
				email_verified: data.email_verified,
				onboarding_completed: data.onboarding_completed
			};
			return { success: true };
		}
		error = getProblemMessage(apiError, 'Email verification failed');
		return { success: false };
	} catch {
		error = 'An unexpected error occurred';
		return { success: false };
	}
}

async function resendVerification(): Promise<{ success: boolean }> {
	error = null;
	try {
		const { data, response } = await apiSdk.resendVerification();
		if (data) {
			return { success: true };
		}
		if (response?.status === 429) {
			error = 'Too many requests. Please try again later.';
			return { success: false };
		}
		return { success: true };
	} catch {
		error = 'An unexpected error occurred';
		return { success: false };
	}
}

async function updateProfile(body: {
	display_name?: string;
	avatar_url?: string | null;
	locale?: string;
	theme?: 'light' | 'dark' | 'system';
	timezone?: string;
}): Promise<{ success: boolean; error?: string }> {
	try {
		const { data, error: apiError } = await apiSdk.updateProfile({ body });
		if (data) {
			applyProfile(data);
			return { success: true };
		}
		return { success: false, error: getProblemMessage(apiError, 'Update failed') };
	} catch {
		return { success: false, error: 'An unexpected error occurred' };
	}
}

async function changePassword(
	currentPassword: string,
	newPassword: string
): Promise<{ success: boolean; error?: string }> {
	try {
		const { data, error: apiError } = await apiSdk.changePassword({
			body: { current_password: currentPassword, new_password: newPassword }
		});
		if (data) {
			return { success: true };
		}
		return { success: false, error: getProblemMessage(apiError, 'Password change failed') };
	} catch {
		return { success: false, error: 'An unexpected error occurred' };
	}
}

async function changeEmail(
	newEmail: string,
	password: string
): Promise<{ success: boolean; error?: string }> {
	try {
		const { data, error: apiError } = await apiSdk.changeEmail({
			body: { new_email: newEmail, password }
		});
		if (data) {
			user = null;
			updateAccessToken(null);
			expiresAt = null;
			return { success: true };
		}
		return { success: false, error: getProblemMessage(apiError, 'Email change failed') };
	} catch {
		return { success: false, error: 'An unexpected error occurred' };
	}
}

async function deleteAccount(confirmation: string): Promise<{ success: boolean; error?: string }> {
	try {
		const { error: apiError } = await apiSdk.deleteAccount({
			body: { confirmation }
		});
		if (!apiError) {
			user = null;
			updateAccessToken(null);
			expiresAt = null;
			return { success: true };
		}
		return { success: false, error: getProblemMessage(apiError, 'Account deletion failed') };
	} catch {
		return { success: false, error: 'An unexpected error occurred' };
	}
}
