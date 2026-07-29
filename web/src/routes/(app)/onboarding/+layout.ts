import { redirect } from '@sveltejs/kit';
import { getOnboarding } from '$lib/stores/onboarding.svelte';
import type { LayoutLoadEvent } from './$types';

export async function load({ parent, url }: LayoutLoadEvent) {
	await parent();

	const onboarding = getOnboarding();
	await onboarding.fetchStatus();

	if (onboarding.completed) {
		redirect(302, '/');
	}

	const currentPath = url.pathname.split('/').pop() ?? '';
	const stepIndex = onboarding.getStepIndex(currentPath);
	const firstIncompletePath = onboarding.getFirstIncompleteStepPath();

	if (
		currentPath &&
		currentPath !== firstIncompletePath &&
		!(currentPath === 'welcome' && onboarding.currentStep === 0)
	) {
		redirect(302, `/onboarding/${firstIncompletePath}`);
	}

	return {
		stepIndex: stepIndex >= 0 ? stepIndex : 0,
		firstIncompletePath
	};
}
