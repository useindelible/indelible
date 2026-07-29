import { beforeEach, describe, expect, it, vi } from 'vitest';

const { mockListProviders } = vi.hoisted(() => ({
	mockListProviders: vi.fn()
}));

vi.mock('$lib/api', () => ({
	listProviders: mockListProviders
}));

import { getInstanceStatus, resetInstanceStatusCache } from '../src/lib/api/instance';

describe('getInstanceStatus', () => {
	beforeEach(() => {
		vi.clearAllMocks();
		resetInstanceStatusCache();
	});

	it('fails closed when provider status cannot be loaded', async () => {
		mockListProviders.mockRejectedValueOnce(new Error('offline'));

		await expect(getInstanceStatus()).resolves.toEqual({
			signupsEnabled: false,
			setupRequired: false
		});
	});

	it('does not reuse stale setup status after a previous fetch', async () => {
		mockListProviders
			.mockResolvedValueOnce({
				data: { providers: [], signups_enabled: true, setup_required: true }
			})
			.mockResolvedValueOnce({
				data: { providers: [], signups_enabled: false, setup_required: false }
			});

		await expect(getInstanceStatus()).resolves.toEqual({
			signupsEnabled: true,
			setupRequired: true
		});
		await expect(getInstanceStatus()).resolves.toEqual({
			signupsEnabled: false,
			setupRequired: false
		});
	});
});
