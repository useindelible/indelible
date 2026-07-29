import { getAuth } from '$lib/stores/auth.svelte';
import { redirect } from '@sveltejs/kit';
import type { PageLoadEvent } from './$types';

export async function load({ parent }: PageLoadEvent) {
	await parent();

	const auth = getAuth();
	if (!auth.isAuthenticated) {
		redirect(302, '/login');
	}
	if (auth.needsOnboarding) {
		redirect(302, '/onboarding/welcome');
	}
	redirect(302, '/dashboard');
}
