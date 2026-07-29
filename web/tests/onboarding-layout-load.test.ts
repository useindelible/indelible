import { beforeEach, describe, expect, it, vi } from 'vitest';

const fetchStatus = vi.fn<() => Promise<void>>();
const getStepIndex = vi.fn<(path: string) => number>();
const getFirstIncompleteStepPath = vi.fn<() => string>();

let completed = false;

vi.mock('$lib/stores/onboarding.svelte', () => ({
	getOnboarding: () => ({
		get completed() {
			return completed;
		},
		fetchStatus,
		getStepIndex,
		getFirstIncompleteStepPath
	})
}));

import { load } from '../src/routes/(app)/onboarding/+layout';

describe('onboarding layout load', () => {
	beforeEach(() => {
		completed = false;
		fetchStatus.mockReset();
		getStepIndex.mockReset();
		getFirstIncompleteStepPath.mockReset();

		fetchStatus.mockResolvedValue();
		getStepIndex.mockReturnValue(0);
		getFirstIncompleteStepPath.mockReturnValue('ready');
	});

	it('waits for parent auth bootstrap before fetching onboarding status', async () => {
		const callOrder: string[] = [];
		let releaseParent!: () => void;

		const parent = vi.fn(
			() =>
				new Promise((resolve) => {
					callOrder.push('parent-called');
					releaseParent = () => {
						callOrder.push('parent-resolved');
						resolve({});
					};
				})
		);

		fetchStatus.mockImplementation(async () => {
			callOrder.push('fetch-status');
		});

		const loadPromise = load({
			parent,
			url: new URL('http://localhost/onboarding/ready')
		} as never);

		await Promise.resolve();
		expect(callOrder).toEqual(['parent-called']);

		releaseParent();
		await loadPromise;

		expect(callOrder).toEqual(['parent-called', 'parent-resolved', 'fetch-status']);
	});

	it('redirects completed onboarding to the app root', async () => {
		completed = true;

		await expect(
			load({
				parent: async () => ({}),
				url: new URL('http://localhost/onboarding/ready')
			} as never)
		).rejects.toMatchObject({ status: 302, location: '/' });
	});
});
