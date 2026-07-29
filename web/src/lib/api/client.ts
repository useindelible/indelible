import { client } from './generated/client.gen';
import { getAccessToken } from '$lib/auth-tokens';

export const AUTH_REDIRECT_SUPPRESSION_HEADER = 'X-Suppress-Auth-Redirect';
const DEFAULT_DEV_API_PORT = '38473';

export const api = client;

const AUTH_PATHS = [
	'/api/v1/auth/refresh',
	'/api/v1/auth/login',
	'/api/v1/auth/register',
	'/api/v1/auth/password/',
	'/api/v1/auth/email/',
	'/api/v1/me'
];

function isAuthPath(url: string): boolean {
	return AUTH_PATHS.some((p) => url.includes(p));
}

export function shouldRedirectToLogin(request: Request, response: Response): boolean {
	if (response.status !== 401) {
		return false;
	}

	if (isAuthPath(request.url)) {
		return false;
	}

	if (request.headers.get(AUTH_REDIRECT_SUPPRESSION_HEADER) === '1') {
		return false;
	}

	if (typeof window === 'undefined') {
		return false;
	}

	const url = new URL(window.location.href);
	return url.pathname !== '/login';
}

function resolveApiBaseUrl(): string {
	const configuredBaseUrl = import.meta.env.VITE_API_BASE_URL?.trim();
	if (configuredBaseUrl) {
		return configuredBaseUrl;
	}

	if (import.meta.env.DEV) {
		if (typeof window !== 'undefined') {
			return `http://${window.location.hostname}:${DEFAULT_DEV_API_PORT}`;
		}

		return `http://localhost:${DEFAULT_DEV_API_PORT}`;
	}

	return '';
}

export function getApiBaseUrl(): string {
	return resolveApiBaseUrl();
}

api.setConfig({
	baseUrl: resolveApiBaseUrl(),
	credentials: 'include'
});

api.interceptors.request.use(async (request) => {
	const token = getAccessToken();
	if (token) {
		request.headers.set('Authorization', `Bearer ${token}`);
	}
	return request;
});

api.interceptors.response.use(async (response, request) => {
	if (shouldRedirectToLogin(request, response)) {
		window.location.href = '/login';
	}
	return response;
});
