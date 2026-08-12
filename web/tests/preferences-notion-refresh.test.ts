import { beforeEach, describe, expect, it, vi } from 'vitest';
import { fireEvent, render, screen, waitFor } from '@testing-library/svelte';

const apiMocks = vi.hoisted(() => ({
	disconnectIntegration: vi.fn(),
	dispatchIntegrationSync: vi.fn(),
	loadIntegrationConnections: vi.fn(),
	loadNotionExportItems: vi.fn(),
	loadNotionSettings: vi.fn(),
	refreshNotionDocumentExport: vi.fn(),
	saveNotionExportItems: vi.fn(),
	saveNotionSettings: vi.fn(),
	startIntegrationAuthorization: vi.fn()
}));

vi.mock('$lib/api/integrations', () => apiMocks);

import NotionPage from '../src/routes/(app)/preferences/integrations/notion/+page.svelte';

describe('Notion document replacement', () => {
	beforeEach(() => {
		vi.clearAllMocks();
		vi.stubGlobal(
			'confirm',
			vi.fn(() => true)
		);
		vi.stubGlobal(
			'IntersectionObserver',
			class {
				observe() {}
				disconnect() {}
			}
		);
		apiMocks.loadIntegrationConnections.mockResolvedValue({
			success: true,
			data: {
				available_oauth_providers: ['notion'],
				connections: [
					{
						id: 'icn_notion',
						provider: 'notion',
						status: 'active',
						config: {},
						pending_jobs: 0,
						last_sync_at: null,
						last_error: null,
						created_at: '2026-08-12T00:00:00Z'
					}
				]
			}
		});
		apiMocks.loadNotionSettings.mockResolvedValue({
			success: true,
			data: {
				export_automatically: true,
				include_highlight_locations: true,
				compact_layout: true,
				selection_enabled: true
			}
		});
		apiMocks.loadNotionExportItems.mockResolvedValue({
			success: true,
			data: {
				items: [
					{
						library_entry_id: 'lib_article',
						title: 'Research article',
						item_type: 'article',
						selected: false,
						exported_page_id: 'page-old'
					}
				],
				total_count: 1,
				filtered_count: 1
			}
		});
		apiMocks.refreshNotionDocumentExport.mockResolvedValue({
			success: true,
			data: {
				library_entry_id: 'lib_article',
				job_id: 'job_replacement',
				archived_page_url: 'https://www.notion.so/Old-page-old'
			}
		});
	});

	it('archives and queues replacement in one action without the old manual ritual', async () => {
		render(NotionPage);
		await waitFor(() => expect(screen.getByText('Research article')).toBeTruthy());

		await fireEvent.click(screen.getByRole('button', { name: 'Refresh' }));

		expect(confirm).toHaveBeenCalledWith(
			'Archive the current Notion page for "Research article" and queue its replacement?'
		);
		expect(String((confirm as ReturnType<typeof vi.fn>).mock.calls[0]?.[0])).not.toMatch(
			/delete.*first|start export/i
		);
		await waitFor(() =>
			expect(apiMocks.refreshNotionDocumentExport).toHaveBeenCalledWith('icn_notion', 'lib_article')
		);
		expect(apiMocks.loadIntegrationConnections).toHaveBeenCalledTimes(2);
		expect(apiMocks.dispatchIntegrationSync).not.toHaveBeenCalled();
		const rollback = await screen.findByRole('link', { name: 'Open archived page in Notion' });
		expect(rollback.getAttribute('href')).toBe('https://www.notion.so/Old-page-old');
	});
});
