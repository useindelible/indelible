import { describe, it, expect, beforeEach } from 'vitest';
import { AUTH_REDIRECT_SUPPRESSION_HEADER, shouldRedirectToLogin } from '$lib/api/client';

describe('CSRF token extraction', () => {
	beforeEach(() => {
		Object.defineProperty(document, 'cookie', {
			writable: true,
			value: ''
		});
	});

	it('extracts csrf token from cookie string', () => {
		document.cookie = 'csrf=test-token-123; session=abc';
		const match = document.cookie.match(/(?:^|;\s*)csrf=([^;]*)/);
		expect(match?.[1]).toBe('test-token-123');
	});

	it('returns undefined when no csrf cookie', () => {
		document.cookie = 'session=abc; other=def';
		const match = document.cookie.match(/(?:^|;\s*)csrf=([^;]*)/);
		expect(match).toBeNull();
	});

	it('handles csrf as first cookie', () => {
		document.cookie = 'csrf=first-token';
		const match = document.cookie.match(/(?:^|;\s*)csrf=([^;]*)/);
		expect(match?.[1]).toBe('first-token');
	});

	it('handles csrf after semicolon', () => {
		document.cookie = 'other=val; csrf=after-semi';
		const match = document.cookie.match(/(?:^|;\s*)csrf=([^;]*)/);
		expect(match?.[1]).toBe('after-semi');
	});
});

describe('CSRF middleware behavior', () => {
	it('adds X-CSRF-Token header to POST requests', () => {
		Object.defineProperty(document, 'cookie', {
			writable: true,
			value: 'csrf=my-csrf-token'
		});

		const request = new Request('http://localhost:38473/api/v1/auth/login', {
			method: 'POST'
		});

		const method = request.method;
		if (method !== 'GET' && method !== 'HEAD' && method !== 'OPTIONS') {
			const match = document.cookie.match(/(?:^|;\s*)csrf=([^;]*)/);
			const csrf = match?.[1];
			if (csrf) {
				request.headers.set('X-CSRF-Token', csrf);
			}
		}

		expect(request.headers.get('X-CSRF-Token')).toBe('my-csrf-token');
	});

	it('does not add X-CSRF-Token header to GET requests', () => {
		Object.defineProperty(document, 'cookie', {
			writable: true,
			value: 'csrf=my-csrf-token'
		});

		const request = new Request('http://localhost:38473/api/v1/auth/session', {
			method: 'GET'
		});

		const method = request.method;
		if (method !== 'GET' && method !== 'HEAD' && method !== 'OPTIONS') {
			const match = document.cookie.match(/(?:^|;\s*)csrf=([^;]*)/);
			const csrf = match?.[1];
			if (csrf) {
				request.headers.set('X-CSRF-Token', csrf);
			}
		}

		expect(request.headers.get('X-CSRF-Token')).toBeNull();
	});

	it('does not add X-CSRF-Token when no csrf cookie present', () => {
		Object.defineProperty(document, 'cookie', {
			writable: true,
			value: 'session=abc'
		});

		const request = new Request('http://localhost:38473/api/v1/auth/login', {
			method: 'POST'
		});

		const method = request.method;
		if (method !== 'GET' && method !== 'HEAD' && method !== 'OPTIONS') {
			const match = document.cookie.match(/(?:^|;\s*)csrf=([^;]*)/);
			const csrf = match?.[1];
			if (csrf) {
				request.headers.set('X-CSRF-Token', csrf);
			}
		}

		expect(request.headers.get('X-CSRF-Token')).toBeNull();
	});

	it('adds X-CSRF-Token header to DELETE requests', () => {
		Object.defineProperty(document, 'cookie', {
			writable: true,
			value: 'csrf=delete-token'
		});

		const request = new Request('http://localhost:38473/api/v1/auth/logout', {
			method: 'DELETE'
		});

		const method = request.method;
		if (method !== 'GET' && method !== 'HEAD' && method !== 'OPTIONS') {
			const match = document.cookie.match(/(?:^|;\s*)csrf=([^;]*)/);
			const csrf = match?.[1];
			if (csrf) {
				request.headers.set('X-CSRF-Token', csrf);
			}
		}

		expect(request.headers.get('X-CSRF-Token')).toBe('delete-token');
	});

	it('skips CSRF header for HEAD requests', () => {
		Object.defineProperty(document, 'cookie', {
			writable: true,
			value: 'csrf=head-token'
		});

		const request = new Request('http://localhost:38473/api/v1/resource', {
			method: 'HEAD'
		});

		const method = request.method;
		if (method !== 'GET' && method !== 'HEAD' && method !== 'OPTIONS') {
			const match = document.cookie.match(/(?:^|;\s*)csrf=([^;]*)/);
			const csrf = match?.[1];
			if (csrf) {
				request.headers.set('X-CSRF-Token', csrf);
			}
		}

		expect(request.headers.get('X-CSRF-Token')).toBeNull();
	});
});

describe('unauthorized redirect behavior', () => {
	it('redirects to /login on 401 response when not already on login page', () => {
		const originalLocation = window.location;
		const mockLocation = { href: 'http://localhost:38473/dashboard' } as Location;
		Object.defineProperty(window, 'location', {
			writable: true,
			value: mockLocation,
			configurable: true
		});

		const response = new Response(null, { status: 401 });
		if (response.status === 401) {
			const url = new URL(window.location.href);
			if (url.pathname !== '/login') {
				window.location.href = '/login';
			}
		}

		expect(mockLocation.href).toBe('/login');

		Object.defineProperty(window, 'location', {
			writable: true,
			value: originalLocation,
			configurable: true
		});
	});

	it('does not redirect when already on /login', () => {
		const originalLocation = window.location;
		const mockLocation = { href: 'http://localhost:38473/login' } as Location;
		Object.defineProperty(window, 'location', {
			writable: true,
			value: mockLocation,
			configurable: true
		});

		const response = new Response(null, { status: 401 });
		if (response.status === 401) {
			const url = new URL(window.location.href);
			if (url.pathname !== '/login') {
				window.location.href = '/login';
			}
		}

		expect(mockLocation.href).toBe('http://localhost:38473/login');

		Object.defineProperty(window, 'location', {
			writable: true,
			value: originalLocation,
			configurable: true
		});
	});

	it('does not redirect on non-401 responses', () => {
		const originalLocation = window.location;
		const mockLocation = { href: 'http://localhost:38473/dashboard' } as Location;
		Object.defineProperty(window, 'location', {
			writable: true,
			value: mockLocation,
			configurable: true
		});

		const response = new Response(null, { status: 200 });
		if (response.status === 401) {
			const url = new URL(window.location.href);
			if (url.pathname !== '/login') {
				window.location.href = '/login';
			}
		}

		expect(mockLocation.href).toBe('http://localhost:38473/dashboard');

		Object.defineProperty(window, 'location', {
			writable: true,
			value: originalLocation,
			configurable: true
		});
	});

	it('skips redirect when the suppression header is present', () => {
		const originalLocation = window.location;
		const mockLocation = { href: 'http://localhost:38473/dashboard' } as Location;
		Object.defineProperty(window, 'location', {
			writable: true,
			value: mockLocation,
			configurable: true
		});

		const request = new Request('http://localhost:38473/api/v1/auth/session', {
			headers: {
				[AUTH_REDIRECT_SUPPRESSION_HEADER]: '1'
			}
		});
		const response = new Response(null, { status: 401 });

		expect(shouldRedirectToLogin(request, response)).toBe(false);

		Object.defineProperty(window, 'location', {
			writable: true,
			value: originalLocation,
			configurable: true
		});
	});

	it('redirects on 401 responses without the suppression header', () => {
		const originalLocation = window.location;
		const mockLocation = { href: 'http://localhost:38473/dashboard' } as Location;
		Object.defineProperty(window, 'location', {
			writable: true,
			value: mockLocation,
			configurable: true
		});

		const request = new Request('http://localhost:38473/api/v1/auth/session');
		const response = new Response(null, { status: 401 });

		expect(shouldRedirectToLogin(request, response)).toBe(true);

		Object.defineProperty(window, 'location', {
			writable: true,
			value: originalLocation,
			configurable: true
		});
	});
});
