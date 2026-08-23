import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen } from '@testing-library/svelte';
import { locale, setupI18nSync } from '$lib/i18n';
import fr from '$lib/i18n/locales/fr.json';

vi.mock('$app/state', () => ({
	page: {
		url: new URL('http://localhost/preferences/account')
	}
}));

vi.mock('$app/paths', () => ({
	resolve: (path: string) => path
}));

import SettingsNav from '$lib/components/settings/SettingsNav.svelte';

describe('SettingsNav', () => {
	beforeEach(() => {
		vi.clearAllMocks();
		locale.set('en');
	});

	it('renders the reading and appearance link in French', () => {
		setupI18nSync({ fr }, 'fr');
		render(SettingsNav);

		expect(screen.getByText('Apparence et lecture')).toBeTruthy();
	});

	it('renders all settings sections', () => {
		render(SettingsNav);

		const expectedLabels = [
			'Account',
			'Reading & Appearance',
			'Integrations',
			'Feed Management',
			'Archival',
			'Mila & AI',
			'Developer'
		];

		for (const label of expectedLabels) {
			expect(screen.getByText(label)).toBeTruthy();
		}
	});

	it('renders nav items as links with correct URLs', () => {
		render(SettingsNav);

		const expectedLinks: Record<string, string> = {
			Account: '/preferences/account',
			'Reading & Appearance': '/preferences/reading-appearance',
			Integrations: '/preferences/integrations',
			'Feed Management': '/preferences/feed-management',
			Archival: '/preferences/archival',
			'Mila & AI': '/preferences/ai',
			Developer: '/preferences/developer'
		};

		for (const [label, href] of Object.entries(expectedLinks)) {
			const link = screen.getByText(label).closest('a');
			expect(link).toBeTruthy();
			expect(link?.getAttribute('href')).toBe(href);
		}
	});

	it('marks the active nav item with aria-current', () => {
		render(SettingsNav);

		const accountLink = screen.getByText('Account').closest('a');
		expect(accountLink?.getAttribute('aria-current')).toBe('page');

		const developerLink = screen.getByText('Developer').closest('a');
		expect(developerLink?.getAttribute('aria-current')).toBeNull();
	});

	it('applies active class to current route nav item', () => {
		render(SettingsNav);

		const accountLink = screen.getByText('Account').closest('a');
		expect(accountLink?.classList.contains('active')).toBe(true);

		const developerLink = screen.getByText('Developer').closest('a');
		expect(developerLink?.classList.contains('active')).toBe(false);
	});

	it('has navigation landmark with accessible label', () => {
		render(SettingsNav);
		const nav = screen.getByRole('navigation', { name: 'Settings navigation' });
		expect(nav).toBeTruthy();
	});
});
