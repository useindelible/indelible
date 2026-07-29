import { describe, it, expect } from 'vitest';
import { render, screen } from '@testing-library/svelte';
import SettingsStub from '$lib/components/settings/SettingsStub.svelte';

describe('SettingsStub', () => {
	it('renders the section title', () => {
		render(SettingsStub, { props: { title: 'Integrations' } });
		expect(screen.getByText('Integrations')).toBeTruthy();
	});

	it('renders the coming soon message', () => {
		render(SettingsStub, { props: { title: 'Notifications' } });
		expect(screen.getByText('This section is coming soon.')).toBeTruthy();
	});

	it('renders different titles for each section', () => {
		const titles = ['Mila & AI', 'API Tokens', 'Import / Export'];
		for (const title of titles) {
			const { unmount } = render(SettingsStub, { props: { title } });
			expect(screen.getByText(title)).toBeTruthy();
			unmount();
		}
	});
});
