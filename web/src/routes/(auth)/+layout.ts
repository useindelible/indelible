import { getPublicRouteRedirect } from '$lib/auth-routing';
import { redirect } from '@sveltejs/kit';
import type { LayoutLoadEvent } from './$types';
import { getAuth } from '$lib/stores/auth.svelte';
import { getInstanceStatus } from '$lib/api/instance';

export async function load({ url, parent }: LayoutLoadEvent) {
	await parent();
	const auth = getAuth();
	const { setupRequired } = await getInstanceStatus();

	const redirectPath = getPublicRouteRedirect({
		pathname: url.pathname,
		isAuthenticated: auth.isAuthenticated,
		needsOnboarding: auth.needsOnboarding,
		needsVerification: auth.needsVerification,
		setupRequired
	});

	if (redirectPath) {
		redirect(302, redirectPath);
	}
}
