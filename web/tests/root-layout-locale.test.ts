import { get } from 'svelte/store';
import { beforeEach, describe, expect, it, vi } from 'vitest';

const authState = vi.hoisted(() => ({
	initialize: vi.fn(),
	user: null as { locale: string | null } | null
}));

vi.mock('$lib/stores/auth.svelte', () => ({
	getAuth: () => ({
		initialize: authState.initialize,
		get user() {
			return authState.user;
		}
	})
}));

function setSystemLanguages(languages: string[]): void {
	Object.defineProperty(window.navigator, 'languages', { configurable: true, value: languages });
}

import { locale } from '$lib/i18n';
import { load } from '../src/routes/+layout';

describe('root layout locale bootstrap', () => {
	beforeEach(() => {
		authState.initialize.mockReset();
		authState.initialize.mockResolvedValue(undefined);
		authState.user = null;
		window.localStorage.clear();
		setSystemLanguages(['fr-CA']);
	});

	it('uses the system locale for an anonymous visitor without persisting it', async () => {
		await load();

		expect(get(locale)).toBe('fr');
		expect(document.documentElement.lang).toBe('fr');
		expect(window.localStorage.length).toBe(0);
	});

	it('prefers a stored locale while auth initializes', async () => {
		window.localStorage.setItem('ind.locale', 'en');

		await load();

		expect(get(locale)).toBe('en');
	});

	it('lets an authenticated profile override the provisional locale', async () => {
		window.localStorage.setItem('ind.locale', 'en');
		authState.user = { locale: 'fr' };

		await load();

		expect(get(locale)).toBe('fr');
		expect(window.localStorage.getItem('ind.locale')).toBe('fr');
	});

	it('clears a stored locale when the profile follows the system', async () => {
		window.localStorage.setItem('ind.locale', 'en');
		authState.user = { locale: null };

		await load();

		expect(get(locale)).toBe('fr');
		expect(window.localStorage.length).toBe(0);
	});
});
