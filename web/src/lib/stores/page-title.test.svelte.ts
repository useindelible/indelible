import { createRawSnippet, tick } from 'svelte';
import { SvelteURL } from 'svelte/reactivity';
import { render, waitFor } from '@testing-library/svelte';
import { afterEach, beforeAll, beforeEach, describe, expect, it, vi } from 'vitest';

import { locale, setupI18nSync } from '$lib/i18n';
import en from '$lib/i18n/locales/en.json';
import fr from '$lib/i18n/locales/fr.json';

vi.mock('$app/state', async () => await import('./page-title.test-page.svelte'));
vi.mock('$lib/styles/theme', () => ({ initTheme: vi.fn() }));

import { page as mockPage } from './page-title.test-page.svelte';
import TitleProvider from './page-title.test-provider.svelte';
import RootLayout from '../../routes/+layout.svelte';

const children = createRawSnippet(() => ({ render: () => '<div></div>' }));

function renderLayout() {
	return render(RootLayout, { props: { children } });
}

// The global setup registers English only; the French assertion needs both catalogs.
beforeAll(() => setupI18nSync({ en, fr }, 'en'));

beforeEach(() => {
	mockPage.url = new SvelteURL('http://localhost/dashboard');
	mockPage.error = null;
	mockPage.status = 200;
});

afterEach(async () => {
	await locale.set('en');
});

describe('document title', () => {
	it('titles the current route', async () => {
		renderLayout();
		await tick();
		expect(document.title).toBe('Home');
	});

	it('retitles on client-side navigation without a remount', async () => {
		renderLayout();
		await tick();
		expect(document.title).toBe('Home');

		mockPage.url = new SvelteURL('http://localhost/library/books');
		await waitFor(() => expect(document.title).toBe('Books'));

		mockPage.url = new SvelteURL('http://localhost/trash');
		await waitFor(() => expect(document.title).toBe('Trash'));
	});

	it('retitles when the locale changes', async () => {
		renderLayout();
		await tick();
		expect(document.title).toBe('Home');

		await locale.set('fr');
		await waitFor(() => expect(document.title).toBe('Accueil'));
	});

	it('shows the error title when the page errored', async () => {
		renderLayout();
		await tick();

		mockPage.error = { message: 'nope' };
		mockPage.status = 404;
		await waitFor(() => expect(document.title).toBe('Page not found'));
	});
});

describe('setDocumentTitle', () => {
	it('names the document once an async value arrives', async () => {
		mockPage.url = new SvelteURL('http://localhost/reader/doc_1');
		renderLayout();

		const provider = render(TitleProvider, { props: { value: null } });
		await tick();
		expect(document.title).toBe('Reader');

		await provider.rerender({ value: 'How to Read a Book' });
		await waitFor(() => expect(document.title).toBe('How to Read a Book'));
	});

	it('drops a stale document name when the route parameter changes', async () => {
		mockPage.url = new SvelteURL('http://localhost/reader/doc_1');
		renderLayout();

		const provider = render(TitleProvider, { props: { value: 'Article A' } });
		await waitFor(() => expect(document.title).toBe('Article A'));

		// Navigating reader-to-reader: the page's route-identity gate yields null until the
		// new record arrives, so the previous document must not title the new URL.
		mockPage.url = new SvelteURL('http://localhost/reader/doc_2');
		await provider.rerender({ value: null });
		await waitFor(() => expect(document.title).toBe('Reader'));
	});

	it('restores the route title when the provider unmounts', async () => {
		mockPage.url = new SvelteURL('http://localhost/reader/doc_1');
		renderLayout();

		const provider = render(TitleProvider, { props: { value: 'An Article' } });
		await waitFor(() => expect(document.title).toBe('An Article'));

		provider.unmount();
		await waitFor(() => expect(document.title).toBe('Reader'));
	});

	it('keeps the newest provider when an older one unmounts', async () => {
		mockPage.url = new SvelteURL('http://localhost/reader/doc_1');
		renderLayout();

		const first = render(TitleProvider, { props: { value: 'First' } });
		await waitFor(() => expect(document.title).toBe('First'));

		render(TitleProvider, { props: { value: 'Second' } });
		await waitFor(() => expect(document.title).toBe('Second'));

		first.unmount();
		await tick();
		expect(document.title).toBe('Second');
	});
});
