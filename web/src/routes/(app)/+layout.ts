import { getProtectedRouteRedirect } from '$lib/auth-routing';
import { getAuth } from '$lib/stores/auth.svelte';
import { redirect } from '@sveltejs/kit';
import type { LayoutLoadEvent } from './$types';
import { getInstanceStatus } from '$lib/api/instance';

export async function load({ parent, url }: LayoutLoadEvent) {
	const parentData = await parent();

	const auth = getAuth();
	const { setupRequired } = await getInstanceStatus();
	const redirectPath = getProtectedRouteRedirect({
		pathname: url.pathname,
		isAuthenticated: auth.isAuthenticated,
		needsOnboarding: auth.needsOnboarding,
		setupRequired
	});

	if (redirectPath) {
		redirect(302, redirectPath);
	}

	return parentData;
}
