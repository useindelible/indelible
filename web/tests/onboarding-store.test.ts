import { describe, it, expect, beforeEach, vi } from 'vitest';
import { flushSync } from 'svelte';
import { createApiModuleMock } from './helpers/api-module-mock';

vi.mock('$lib/api', () => createApiModuleMock());

import { api } from '$lib/api';
import { getOnboarding, ONBOARDING_STEPS } from '$lib/stores/onboarding.svelte';

const mockGET = vi.mocked(api.GET);
const mockPOST = vi.mocked(api.POST);

function makeSteps(completedSteps: number[]) {
	return [
		{ step: 1, name: 'Account Setup', completed: completedSteps.includes(1) },
		{ step: 2, name: 'Add Content', completed: completedSteps.includes(2) },
		{ step: 3, name: 'RSS Feeds', completed: completedSteps.includes(3) },
		{ step: 4, name: 'AI Configuration', completed: completedSteps.includes(4) },
		{ step: 5, name: 'Complete', completed: completedSteps.includes(5) }
	];
}

function makeStatusResponse(completedSteps: number[], isCompleted = false) {
	const maxCompleted = completedSteps.length > 0 ? Math.max(...completedSteps) : 0;
	return {
		current_step: maxCompleted,
		completed: isCompleted,
		steps: makeSteps(completedSteps)
	};
}

describe('onboarding store', () => {
	beforeEach(() => {
		vi.clearAllMocks();
	});

	it('has 6 visible steps', () => {
		expect(ONBOARDING_STEPS).toHaveLength(6);
	});

	it('welcome step has no backend step number', () => {
		const welcome = ONBOARDING_STEPS.find((s) => s.path === 'welcome');
		expect(welcome?.backendStep).toBeNull();
	});

	it('steps after welcome map to five backend steps', () => {
		const stepsWithBackend = ONBOARDING_STEPS.filter((s) => s.backendStep !== null);
		expect(stepsWithBackend).toHaveLength(5);
		expect(stepsWithBackend.map((s) => s.backendStep)).toEqual([1, 2, 3, 4, 5]);
	});

	it('fetchStatus populates step completion data', async () => {
		mockGET.mockResolvedValue({
			data: makeStatusResponse([1, 2]),
			error: undefined,
			response: new Response(null, { status: 200 })
		} as never);

		const onboarding = getOnboarding();
		await onboarding.fetchStatus();
		flushSync();

		expect(onboarding.steps.filter((s) => s.completed).map((s) => s.step)).toEqual([1, 2]);
		expect(onboarding.completed).toBe(false);
		expect(onboarding.loaded).toBe(true);
	});

	it('fetchStatus handles completed onboarding', async () => {
		mockGET.mockResolvedValue({
			data: makeStatusResponse([1, 2, 3, 4, 5], true),
			error: undefined,
			response: new Response(null, { status: 200 })
		} as never);

		const onboarding = getOnboarding();
		await onboarding.fetchStatus();
		flushSync();

		expect(onboarding.completed).toBe(true);
		expect(onboarding.steps.filter((s) => s.completed).map((s) => s.step)).toEqual([1, 2, 3, 4, 5]);
	});

	it('fetchStatus handles network errors gracefully', async () => {
		mockGET.mockRejectedValue(new Error('Network error'));

		const onboarding = getOnboarding();
		await onboarding.fetchStatus();
		flushSync();

		expect(onboarding.steps).toEqual([]);
		expect(onboarding.completed).toBe(false);
		expect(onboarding.loaded).toBe(true);
	});

	it('completeStep adds to completed steps on success', async () => {
		mockGET.mockResolvedValue({
			data: makeStatusResponse([]),
			error: undefined,
			response: new Response(null, { status: 200 })
		} as never);

		const onboarding = getOnboarding();
		await onboarding.fetchStatus();

		mockPOST.mockResolvedValue({
			data: makeStatusResponse([1]),
			error: undefined,
			response: new Response(null, { status: 200 })
		} as never);

		const result = await onboarding.completeStep(1, { display_name: 'Test' });
		flushSync();

		expect(result).toBe(true);
		expect(onboarding.steps.find((s) => s.step === 1)?.completed).toBe(true);
	});

	it('completeStep sends payload with request', async () => {
		mockGET.mockResolvedValue({
			data: makeStatusResponse([]),
			error: undefined,
			response: new Response(null, { status: 200 })
		} as never);

		const onboarding = getOnboarding();
		await onboarding.fetchStatus();

		mockPOST.mockResolvedValue({
			data: makeStatusResponse([1]),
			error: undefined,
			response: new Response(null, { status: 200 })
		} as never);

		await onboarding.completeStep(1, { display_name: 'User', theme: 'dark' });

		expect(mockPOST).toHaveBeenCalledWith('/api/v1/onboarding/steps/{step}/complete', {
			path: { step: 1 },
			body: { data: { display_name: 'User', theme: 'dark' } }
		});
	});

	it('completeStep returns false on failure', async () => {
		mockGET.mockResolvedValue({
			data: makeStatusResponse([]),
			error: undefined,
			response: new Response(null, { status: 200 })
		} as never);

		const onboarding = getOnboarding();
		await onboarding.fetchStatus();

		mockPOST.mockRejectedValue(new Error('Server error'));

		const result = await onboarding.completeStep(1);
		expect(result).toBe(false);
	});

	it('completeStep does not add duplicate step numbers', async () => {
		mockGET.mockResolvedValue({
			data: makeStatusResponse([1]),
			error: undefined,
			response: new Response(null, { status: 200 })
		} as never);

		const onboarding = getOnboarding();
		await onboarding.fetchStatus();

		mockPOST.mockResolvedValue({
			data: makeStatusResponse([1]),
			error: undefined,
			response: new Response(null, { status: 200 })
		} as never);

		await onboarding.completeStep(1);
		flushSync();

		expect(onboarding.steps.filter((s) => s.step === 1)).toHaveLength(1);
	});

	it('skipAll calls the skip endpoint and updates state', async () => {
		mockGET.mockResolvedValue({
			data: makeStatusResponse([]),
			error: undefined,
			response: new Response(null, { status: 200 })
		} as never);

		const onboarding = getOnboarding();
		await onboarding.fetchStatus();

		mockPOST.mockResolvedValue({
			data: makeStatusResponse([1, 2, 3, 4, 5], true),
			error: undefined,
			response: new Response(null, { status: 200 })
		} as never);

		const result = await onboarding.skipAll();
		flushSync();

		expect(result).toBe(true);
		expect(onboarding.completed).toBe(true);
		expect(mockPOST).toHaveBeenCalledWith('/api/v1/onboarding/skip');
	});

	it('skipAll returns false on error', async () => {
		mockGET.mockResolvedValue({
			data: makeStatusResponse([]),
			error: undefined,
			response: new Response(null, { status: 200 })
		} as never);

		const onboarding = getOnboarding();
		await onboarding.fetchStatus();

		mockPOST.mockRejectedValue(new Error('Error'));

		const result = await onboarding.skipAll();
		expect(result).toBe(false);
	});

	it('getFirstIncompleteStepPath returns first incomplete step', async () => {
		mockGET.mockResolvedValue({
			data: makeStatusResponse([1, 2]),
			error: undefined,
			response: new Response(null, { status: 200 })
		} as never);

		const onboarding = getOnboarding();
		await onboarding.fetchStatus();
		flushSync();

		expect(onboarding.getFirstIncompleteStepPath()).toBe('feeds');
	});

	it('getFirstIncompleteStepPath returns account when nothing completed', async () => {
		mockGET.mockResolvedValue({
			data: makeStatusResponse([]),
			error: undefined,
			response: new Response(null, { status: 200 })
		} as never);

		const onboarding = getOnboarding();
		await onboarding.fetchStatus();
		flushSync();

		expect(onboarding.getFirstIncompleteStepPath()).toBe('account');
	});

	it('getFirstIncompleteStepPath returns ready when all backend steps completed', async () => {
		mockGET.mockResolvedValue({
			data: makeStatusResponse([1, 2, 3, 4, 5]),
			error: undefined,
			response: new Response(null, { status: 200 })
		} as never);

		const onboarding = getOnboarding();
		await onboarding.fetchStatus();
		flushSync();

		expect(onboarding.getFirstIncompleteStepPath()).toBe('ready');
	});

	it('isStepCompleted returns correct values', async () => {
		mockGET.mockResolvedValue({
			data: makeStatusResponse([1, 2]),
			error: undefined,
			response: new Response(null, { status: 200 })
		} as never);

		const onboarding = getOnboarding();
		await onboarding.fetchStatus();
		flushSync();

		expect(onboarding.isStepCompleted('account')).toBe(true);
		expect(onboarding.isStepCompleted('add-content')).toBe(true);
		expect(onboarding.isStepCompleted('welcome')).toBe(false);
	});

	it('getStepIndex returns correct indices', () => {
		const onboarding = getOnboarding();
		expect(onboarding.getStepIndex('welcome')).toBe(0);
		expect(onboarding.getStepIndex('account')).toBe(1);
		expect(onboarding.getStepIndex('add-content')).toBe(2);
		expect(onboarding.getStepIndex('feeds')).toBe(3);
		expect(onboarding.getStepIndex('ai')).toBe(4);
		expect(onboarding.getStepIndex('ready')).toBe(5);
		expect(onboarding.getStepIndex('nonexistent')).toBe(-1);
	});
});
