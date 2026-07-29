import { beforeEach, describe, expect, it, vi } from 'vitest';

const { getInstanceStatus } = vi.hoisted(() => ({ getInstanceStatus: vi.fn() }));

let isAuthenticated = false;
let needsOnboarding = false;

vi.mock('$lib/api/instance', () => ({ getInstanceStatus }));

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

import { load } from '../src/routes/(app)/+layout';

describe('app layout load', () => {
	beforeEach(() => {
		isAuthenticated = false;
		needsOnboarding = false;
		getInstanceStatus.mockReset();
		getInstanceStatus.mockResolvedValue({ signupsEnabled: false, setupRequired: false });
	});

	it('does not fetch public instance status for authenticated users', async () => {
		isAuthenticated = true;
		const parentData = { auth: 'ready' };

		await expect(
			load({
				parent: async () => parentData,
				url: new URL('http://localhost/dashboard')
			} as never)
		).resolves.toEqual(parentData);
		expect(getInstanceStatus).not.toHaveBeenCalled();
	});

	it('redirects authenticated new accounts without fetching instance status', async () => {
		isAuthenticated = true;
		needsOnboarding = true;

		await expect(
			load({
				parent: async () => ({}),
				url: new URL('http://localhost/dashboard')
			} as never)
		).rejects.toMatchObject({ status: 302, location: '/onboarding/welcome' });
		expect(getInstanceStatus).not.toHaveBeenCalled();
	});

	it('fetches setup status when routing unauthenticated visitors', async () => {
		getInstanceStatus.mockResolvedValue({ signupsEnabled: true, setupRequired: true });

		await expect(
			load({
				parent: async () => ({}),
				url: new URL('http://localhost/dashboard')
			} as never)
		).rejects.toMatchObject({ status: 302, location: '/register' });
		expect(getInstanceStatus).toHaveBeenCalledOnce();
	});
});
