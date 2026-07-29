import { beforeEach, describe, expect, it, vi } from 'vitest';

let isAuthenticated = false;
let needsOnboarding = false;

vi.mock('$lib/stores/auth.svelte', () => ({
	getAuth: () => ({
		get isAuthenticated() {
			return isAuthenticated;
		},
		get needsOnboarding() {
			return needsOnboarding;
		}
	})
}));

import { load } from '../src/routes/+page';

describe('root page load', () => {
	beforeEach(() => {
		isAuthenticated = false;
		needsOnboarding = false;
	});

	it('waits for auth bootstrap before choosing a destination', async () => {
		let releaseParent!: () => void;
		const parent = vi.fn(
			() =>
				new Promise<Record<string, never>>((resolve) => {
					releaseParent = () => resolve({});
				})
		);

		const loadPromise = load({ parent } as never);
		await Promise.resolve();

		expect(parent).toHaveBeenCalledOnce();
		releaseParent();
		await expect(loadPromise).rejects.toMatchObject({ status: 302, location: '/login' });
	});

	it('redirects unauthenticated visitors to login', async () => {
		await expect(load({ parent: async () => ({}) } as never)).rejects.toMatchObject({
			status: 302,
			location: '/login'
		});
	});

	it('redirects new accounts directly to onboarding', async () => {
		isAuthenticated = true;
		needsOnboarding = true;

		await expect(load({ parent: async () => ({}) } as never)).rejects.toMatchObject({
			status: 302,
			location: '/onboarding/welcome'
		});
	});

	it('redirects onboarded accounts to the dashboard', async () => {
		isAuthenticated = true;

		await expect(load({ parent: async () => ({}) } as never)).rejects.toMatchObject({
			status: 302,
			location: '/dashboard'
		});
	});
});
