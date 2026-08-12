import { describe, expect, it } from 'vitest';
import { render } from '@testing-library/svelte';
import NotionHeroTile from '../../src/routes/(app)/preferences/integrations/notion/components/NotionHeroTile.svelte';

describe('NotionHeroTile', () => {
	it('labels sample records without obscuring live sync metrics', () => {
		const { container } = render(NotionHeroTile, {
			props: {
				workspaceIcon: null,
				workspaceName: null,
				databaseLabel: 'Indelible Library',
				connectionState: 'disconnected',
				heroStatus: 'Not connected',
				formattedHeroLastSync: 'Never',
				pendingJobs: 0
			}
		});

		expect(container.textContent).toContain('Example preview');
		expect(container.textContent).toContain('On the difficulty of writing');
		expect(container.textContent).toContain('Last edited Never');
		expect(container.textContent).toContain('0 pending');
	});
});
