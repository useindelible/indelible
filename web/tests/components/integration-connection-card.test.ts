import { render, screen } from '@testing-library/svelte';
import { describe, expect, it } from 'vitest';

import IntegrationConnectionCard from '$lib/components/integrations/IntegrationConnectionCard.svelte';
import IntegrationStatusPill from '$lib/components/integrations/IntegrationStatusPill.svelte';

describe('IntegrationConnectionCard', () => {
	it('renders a current-design card shell with status and errors', () => {
		render(IntegrationConnectionCard, {
			props: {
				title: 'Notion',
				tagline: 'Sync highlights into a Notion database.',
				statusLabel: 'Needs attention',
				statusVariant: 'attention',
				errorMessage: 'Managed Notion database no longer exists.'
			}
		});

		expect(screen.getByTestId('integration-connection-card')).toBeTruthy();
		expect(screen.getByText('Notion')).toBeTruthy();
		expect(screen.getByText('Sync highlights into a Notion database.')).toBeTruthy();
		expect(screen.getByText('Needs attention')).toBeTruthy();
		expect(screen.getByRole('alert').textContent).toContain('Managed Notion database');
	});
});

describe('IntegrationStatusPill', () => {
	it('marks syncing status with a stable variant and pulse indicator', () => {
		render(IntegrationStatusPill, {
			props: {
				variant: 'syncing',
				label: 'Syncing',
				pulse: true
			}
		});

		const pill = screen.getByTestId('integration-status-pill');
		expect(pill.getAttribute('data-variant')).toBe('syncing');
		expect(pill.textContent).toContain('Syncing');
		expect(pill.querySelector('.pulse-dot')).toBeTruthy();
	});
});
