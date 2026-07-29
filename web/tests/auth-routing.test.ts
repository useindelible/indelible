import { describe, expect, it } from 'vitest';
import { getProtectedRouteRedirect, getPublicRouteRedirect } from '$lib/auth-routing';

describe('getProtectedRouteRedirect', () => {
	it('keeps authenticated users on protected routes once bootstrap succeeds', () => {
		expect(
			getProtectedRouteRedirect({
				pathname: '/dashboard',
				isAuthenticated: true,
				needsOnboarding: false
			})
		).toBeNull();
	});

	it('redirects unauthenticated users on protected routes to /login', () => {
		expect(
			getProtectedRouteRedirect({
				pathname: '/preferences/profile',
				isAuthenticated: false,
				needsOnboarding: false
			})
		).toBe('/login');
	});

	it('redirects authenticated users who still need onboarding to the onboarding flow', () => {
		expect(
			getProtectedRouteRedirect({
				pathname: '/dashboard',
				isAuthenticated: true,
				needsOnboarding: true
			})
		).toBe('/onboarding/welcome');
	});

	it('allows users who still need onboarding to stay on onboarding routes', () => {
		expect(
			getProtectedRouteRedirect({
				pathname: '/onboarding/account',
				isAuthenticated: true,
				needsOnboarding: true
			})
		).toBeNull();
	});

	it('redirects onboarded users away from onboarding routes', () => {
		expect(
			getProtectedRouteRedirect({
				pathname: '/onboarding/welcome',
				isAuthenticated: true,
				needsOnboarding: false
			})
		).toBe('/');
	});

	it('redirects to /register during first-run setup', () => {
		expect(
			getProtectedRouteRedirect({
				pathname: '/dashboard',
				isAuthenticated: false,
				needsVerification: false,
				needsOnboarding: false,
				setupRequired: true
			})
		).toBe('/register');
	});

	it('does not let stale setup status override authenticated verification state', () => {
		expect(
			getProtectedRouteRedirect({
				pathname: '/dashboard',
				isAuthenticated: true,
				needsVerification: true,
				needsOnboarding: false,
				setupRequired: true
			})
		).toBe('/verify-email');
	});
});

describe('getPublicRouteRedirect', () => {
	it('leaves public auth routes alone for unauthenticated users', () => {
		expect(
			getPublicRouteRedirect({
				pathname: '/login',
				isAuthenticated: false,
				needsOnboarding: false,
				needsVerification: false
			})
		).toBeNull();
	});

	it('allows authenticated users who need verification to stay on /verify-email', () => {
		expect(
			getPublicRouteRedirect({
				pathname: '/verify-email',
				isAuthenticated: true,
				needsOnboarding: false,
				needsVerification: true
			})
		).toBeNull();
	});

	it('redirects authenticated users who need onboarding from public auth routes', () => {
		expect(
			getPublicRouteRedirect({
				pathname: '/login',
				isAuthenticated: true,
				needsOnboarding: true,
				needsVerification: false
			})
		).toBe('/onboarding/welcome');
	});

	it('redirects fully authenticated users away from public auth routes', () => {
		expect(
			getPublicRouteRedirect({
				pathname: '/login',
				isAuthenticated: true,
				needsOnboarding: false,
				needsVerification: false
			})
		).toBe('/');
	});

	it('redirects unauthenticated visitors on /login to /register during setup', () => {
		expect(
			getPublicRouteRedirect({
				pathname: '/login',
				isAuthenticated: false,
				needsOnboarding: false,
				needsVerification: false,
				setupRequired: true
			})
		).toBe('/register');
	});

	it('does not loop on /register during setup', () => {
		expect(
			getPublicRouteRedirect({
				pathname: '/register',
				isAuthenticated: false,
				needsOnboarding: false,
				needsVerification: false,
				setupRequired: true
			})
		).toBeNull();
	});

	it('does not let stale setup status redirect authenticated users away from verification', () => {
		expect(
			getPublicRouteRedirect({
				pathname: '/verify-email',
				isAuthenticated: true,
				needsOnboarding: false,
				needsVerification: true,
				setupRequired: true
			})
		).toBeNull();
	});
});
