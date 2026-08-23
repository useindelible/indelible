import { getAuth } from '$lib/stores/auth.svelte';
import {
	applyLocale,
	applyProfileLocale,
	readStoredLocale,
	resolveInitialLocale,
	setupI18n,
	waitLocale
} from '$lib/i18n';

export const ssr = false;
export const prerender = false;

export async function load() {
	const provisionalLocale = resolveInitialLocale({
		storedLocale: readStoredLocale(),
		navigatorLanguages: navigator.languages
	});
	setupI18n(provisionalLocale);
	applyLocale(provisionalLocale);
	await waitLocale(provisionalLocale);

	const auth = getAuth();
	await auth.initialize();
	await applyProfileLocale(auth.user ? (auth.user.locale ?? null) : undefined);

	return { auth };
}
