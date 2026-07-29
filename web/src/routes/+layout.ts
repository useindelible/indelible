import { getAuth } from '$lib/stores/auth.svelte';

export const ssr = false;
export const prerender = false;

export async function load() {
	const auth = getAuth();
	await auth.initialize();
	return { auth };
}
