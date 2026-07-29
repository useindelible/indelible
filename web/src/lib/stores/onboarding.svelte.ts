import * as apiSdk from '$lib/api';
import type { OnboardingStepResponse, StepData } from '$lib/api';

type OnboardingStep = OnboardingStepResponse;

export const ONBOARDING_STEPS = [
	{ path: 'welcome', label: 'Welcome', backendStep: null },
	{ path: 'account', label: 'Account Setup', backendStep: 1 },
	{ path: 'add-content', label: 'Add Content', backendStep: 2 },
	{ path: 'feeds', label: 'RSS Feeds', backendStep: 3 },
	{ path: 'ai', label: 'AI Setup', backendStep: 4 },
	{ path: 'ready', label: 'Ready', backendStep: 5 }
] as const;

export type StepPath = (typeof ONBOARDING_STEPS)[number]['path'];

let currentStep = $state(0);
let completed = $state(false);
let steps = $state<OnboardingStep[]>([]);
let loaded = $state(false);
let error = $state<string | null>(null);

export function getOnboarding() {
	return {
		get currentStep() {
			return currentStep;
		},
		get completed() {
			return completed;
		},
		get steps() {
			return steps;
		},
		get loaded() {
			return loaded;
		},
		get error() {
			return error;
		},
		fetchStatus,
		completeStep,
		skipAll,
		getFirstIncompleteStepPath,
		isStepCompleted,
		getStepIndex
	};
}

async function fetchStatus(): Promise<void> {
	try {
		const { data } = await apiSdk.getOnboarding();
		if (data) {
			currentStep = data.current_step;
			completed = data.completed;
			steps = data.steps;
		}
	} catch {
		currentStep = 0;
		completed = false;
		steps = [];
	} finally {
		loaded = true;
	}
}

async function completeStep(backendStep: number, payload?: StepData): Promise<boolean> {
	error = null;
	try {
		const {
			data,
			error: apiError,
			response
		} = await apiSdk.completeStep({
			path: { step: backendStep },
			body: { data: payload ?? {} }
		});
		if (data) {
			currentStep = data.current_step;
			completed = data.completed;
			steps = data.steps;
			return true;
		}
		error = extractErrorMessage(apiError, response, 'Failed to update onboarding');
		return false;
	} catch {
		error = 'An unexpected error occurred';
		return false;
	}
}

async function skipAll(): Promise<boolean> {
	error = null;
	try {
		const { data, error: apiError, response } = await apiSdk.skipOnboarding();
		if (data) {
			currentStep = data.current_step;
			completed = data.completed;
			steps = data.steps;
			return true;
		}
		error = extractErrorMessage(apiError, response, 'Failed to skip onboarding');
		return false;
	} catch {
		error = 'An unexpected error occurred';
		return false;
	}
}

function extractErrorMessage(apiError: unknown, response: Response, fallback: string): string {
	if (apiError && typeof apiError === 'object') {
		const err = apiError as Record<string, unknown>;
		const fieldErrors = err.errors;
		if (Array.isArray(fieldErrors)) {
			const first = fieldErrors[0] as Record<string, unknown> | undefined;
			if (typeof first?.message === 'string') return first.message;
		}
		if (typeof err.detail === 'string') return err.detail;
		if (typeof err.message === 'string') return err.message;
	}
	if (response.status === 422) return 'Please check your onboarding details and try again';
	return fallback;
}

function getFirstIncompleteStepPath(): StepPath {
	for (const step of ONBOARDING_STEPS) {
		if (step.backendStep === null) continue;
		const backendStep = steps.find((s) => s.step === step.backendStep);
		if (!backendStep || !backendStep.completed) {
			return step.path;
		}
	}
	return 'ready';
}

function isStepCompleted(path: StepPath): boolean {
	const stepDef = ONBOARDING_STEPS.find((s) => s.path === path);
	if (!stepDef || stepDef.backendStep === null) return false;
	const backendStep = steps.find((s) => s.step === stepDef.backendStep);
	return backendStep?.completed ?? false;
}

function getStepIndex(path: string): number {
	return ONBOARDING_STEPS.findIndex((s) => s.path === path);
}
