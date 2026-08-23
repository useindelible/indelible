import { fireEvent, render, screen, waitFor } from '@testing-library/svelte';
import { beforeEach, describe, expect, it, vi } from 'vitest';
import { setupI18nSync } from '$lib/i18n';
import en from '$lib/i18n/locales/en.json';
import fr from '$lib/i18n/locales/fr.json';
import type { PreferencesSettingsResponse } from '$lib/api';

const settings = vi.hoisted(() => ({
	load: vi.fn(),
	save: vi.fn()
}));

const auth = vi.hoisted(() => ({
	user: { locale: 'en' as string | null },
	updateProfile: vi.fn()
}));

vi.mock('$lib/api/settings', () => ({
	loadPreferencesSettings: settings.load,
	savePreferencesSettings: settings.save
}));

vi.mock('$lib/stores/auth.svelte', () => ({ getAuth: () => auth }));
vi.mock('$lib/stores/library.svelte', () => ({
	getLibrary: () => ({ applyPreferences: vi.fn() })
}));
vi.mock('$lib/stores/app-preferences.svelte', () => ({
	getAppPreferences: () => ({ setDefaultView: vi.fn() })
}));
vi.mock('$lib/styles/theme', () => ({ applyTheme: vi.fn(), saveTheme: vi.fn() }));

import ReadingAppearancePage from '../../src/routes/(app)/preferences/reading-appearance/+page.svelte';

function preferences(theme: PreferencesSettingsResponse['theme'] = 'system') {
	return {
		theme,
		appearance: { accent_color: 'blue' },
		layout: {
			sidebar_mode: 'expanded',
			default_view: 'library',
			list_density: 'compact',
			side_panel: 'open'
		},
		workflow: { triage_mode: 'focus', auto_advance: true },
		reader: {
			font_family: 'serif',
			font_size: 'medium',
			line_height: 'relaxed',
			email_open_mode: 'reader'
		},
		ai: { mila_enabled: true, custom_prompt: null }
	} satisfies PreferencesSettingsResponse;
}

describe('reading appearance save', () => {
	beforeEach(() => {
		vi.clearAllMocks();
		localStorage.clear();
		setupI18nSync({ en, fr }, 'en');
		auth.user.locale = 'en';
		settings.load.mockResolvedValue({ success: true, data: preferences() });
		settings.save.mockImplementation(async (body) => ({ success: true, data: body }));
	});

	it('keeps persisted preferences when the locale update fails', async () => {
		auth.updateProfile.mockResolvedValue({ success: false, error: 'Locale unavailable' });
		render(ReadingAppearancePage);

		const language = await screen.findByRole('combobox', { name: 'Language' });
		await fireEvent.click(screen.getByRole('tab', { name: 'Dark' }));
		await fireEvent.change(language, { target: { value: 'fr' } });
		await fireEvent.click(screen.getByRole('button', { name: 'Save' }));
		await screen.findByText('Locale unavailable');
		await fireEvent.click(screen.getByRole('button', { name: 'Discard' }));

		expect(screen.getByRole('tab', { name: 'Dark' }).getAttribute('aria-selected')).toBe('true');
		expect((language as HTMLSelectElement).value).toBe('en');
	});

	it('applies and remembers a successfully saved locale', async () => {
		auth.updateProfile.mockImplementation(async ({ locale }) => {
			auth.user.locale = locale;
			return { success: true };
		});
		render(ReadingAppearancePage);

		const language = await screen.findByRole('combobox', { name: 'Language' });
		await fireEvent.change(language, { target: { value: 'fr' } });
		await fireEvent.click(screen.getByRole('button', { name: 'Save' }));
		await waitFor(() => expect(document.documentElement.lang).toBe('fr'));

		expect((language as HTMLSelectElement).value).toBe('fr');
		expect(localStorage.getItem('ind.locale')).toBe('fr');
	});
});
