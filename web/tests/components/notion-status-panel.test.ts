import { afterEach, describe, it, expect, vi } from 'vitest';
import { fireEvent, render, screen } from '@testing-library/svelte';
import NotionStatusPanel from '$lib/components/integrations/NotionStatusPanel.svelte';
import type { IntegrationConnectionDto, NotionSettingsDto } from '$lib/api';

afterEach(() => {
	vi.unstubAllGlobals();
});

const settings: NotionSettingsDto = {
	export_automatically: true,
	include_highlight_locations: true,
	compact_layout: true,
	selection_enabled: false
};

function panelHandlers() {
	return {
		onSync: () => {},
		onReauthorize: () => {},
		onChangeAccount: () => {},
		onDisconnect: () => {},
		onSettingChange: () => {},
		onItemsSearch: () => {},
		onItemsLoadMore: () => {},
		onItemSelection: () => {},
		onRefreshItem: () => {}
	};
}

function makeConnection(
	overrides: Partial<IntegrationConnectionDto> = {}
): IntegrationConnectionDto {
	return {
		id: 'int_abc',
		provider: 'notion',
		status: 'active',
		last_sync_at: '2026-04-25T12:00:00Z',
		last_error: null,
		config: {
			provider: 'notion',
			workspace_id: 'wsp_1',
			workspace_name: "Maya's Reading",
			database_id: 'db_1a2b',
			data_source_id: null,
			workspace_icon: null,
			export_automatically: true,
			include_highlight_locations: true,
			compact_layout: true,
			selection_enabled: false
		},
		pending_jobs: 0,
		created_at: '2026-04-17T12:00:00Z',
		...overrides
	} as IntegrationConnectionDto;
}

describe('NotionStatusPanel', () => {
	it('renders the empty state with a Connect Notion CTA', () => {
		render(NotionStatusPanel, {
			props: {
				connection: undefined,
				settings,
				...panelHandlers()
			}
		});
		const panel = screen.getByTestId('notion-status-panel');
		expect(panel.dataset.state).toBe('empty');
		expect(screen.getByText('Connect Notion')).toBeTruthy();
	});

	it('renders the workspace name and database id when connected', () => {
		render(NotionStatusPanel, {
			props: {
				connection: makeConnection(),
				settings,
				...panelHandlers()
			}
		});
		expect(screen.getAllByText("Maya's Reading").length).toBeGreaterThan(0);
		expect(screen.getByTestId('notion-database-id').textContent?.trim()).toBe('db_1a2b');
	});

	it('renders a pending pill when pending_jobs > 0', () => {
		render(NotionStatusPanel, {
			props: {
				connection: makeConnection({ pending_jobs: 3 }),
				settings,
				...panelHandlers()
			}
		});
		expect(screen.getByTestId('notion-pending-pill').textContent).toContain('3');
	});

	it('renders zero pending jobs honestly', () => {
		render(NotionStatusPanel, {
			props: {
				connection: makeConnection({ pending_jobs: 0 }),
				settings,
				...panelHandlers()
			}
		});
		expect(screen.getByTestId('notion-pending-pill').textContent).toContain('0');
	});

	it('renders the rate-limit callout when last_error indicates 429', () => {
		render(NotionStatusPanel, {
			props: {
				connection: makeConnection({ last_error: '429 Too Many Requests' }),
				settings,
				...panelHandlers()
			}
		});
		expect(screen.getByTestId('notion-rate-limit')).toBeTruthy();
	});

	it('renders the auth-failure callout when last_error indicates 401', () => {
		render(NotionStatusPanel, {
			props: {
				connection: makeConnection({
					last_error: '401 Unauthorized: token revoked'
				}),
				settings,
				...panelHandlers()
			}
		});
		expect(screen.getByTestId('notion-auth-failure')).toBeTruthy();
		expect(screen.getAllByText(/Re-authorize/i).length).toBeGreaterThan(0);
	});

	it('renders an empty workspace placeholder when config is missing fields', () => {
		render(NotionStatusPanel, {
			props: {
				connection: makeConnection({
					config: {
						provider: 'notion',
						workspace_id: undefined,
						workspace_name: undefined,
						database_id: undefined,
						data_source_id: undefined,
						workspace_icon: undefined,
						export_automatically: true,
						include_highlight_locations: true,
						compact_layout: true,
						selection_enabled: false
					}
				}),
				settings,
				...panelHandlers()
			}
		});
		const dbCell = screen.getByTestId('notion-database-id');
		expect(dbCell.textContent?.trim()).toBe('Provisioned on first export');
	});

	it('does not render a hardcoded managed schema or deleted-database reprovisioning claim', () => {
		render(NotionStatusPanel, {
			props: {
				connection: makeConnection(),
				settings,
				...panelHandlers()
			}
		});
		expect(screen.queryByText(/Managed properties/)).toBeNull();
		expect(screen.queryByText(/Start Export provisions/)).toBeNull();
	});

	it('renders Readwise-style settings and item selection controls', async () => {
		const calls: Array<string> = [];
		render(NotionStatusPanel, {
			props: {
				connection: makeConnection(),
				settings: {
					...settings,
					selection_enabled: true
				},
				items: [
					{
						library_entry_id: 'lib_123',
						title: 'Exported essay',
						item_type: 'article',
						url: 'https://example.com',
						selected: true,
						exported_page_id: 'page_123',
						last_synced_at: null,
						last_error: null
					}
				],
				itemsTotal: 1,
				itemsFilteredCount: 1,
				...panelHandlers(),
				onSettingChange: (key, value) => calls.push(`${key}:${value}`),
				onItemsSearch: (query) => calls.push(`search:${query}`),
				onItemSelection: (libraryEntryId, selected) => calls.push(`${libraryEntryId}:${selected}`),
				onRefreshItem: (item) => calls.push(`refresh:${item.library_entry_id}`)
			}
		});

		expect(screen.getByText('Export automatically')).toBeTruthy();
		expect(screen.getByText('Include highlight locations')).toBeTruthy();
		expect(screen.getByText('Compact layout')).toBeTruthy();
		expect(screen.getByText('Select items to export')).toBeTruthy();
		expect(screen.getByText('Exported essay')).toBeTruthy();
		expect(screen.getByTestId('notion-items-meta').textContent).toContain(
			'1 selected · 1 of 1 documents'
		);

		const switches = screen.getAllByRole('switch');
		await fireEvent.click(switches[0]);
		await fireEvent.input(screen.getByPlaceholderText('Search documents'), {
			target: { value: 'essay' }
		});
		await fireEvent.click(screen.getByLabelText('Deselect'));
		await fireEvent.click(screen.getAllByText('Refresh').at(-1)!);

		expect(calls).toContain('export_automatically:false');
		expect(calls).toContain('search:essay');
		expect(calls).toContain('lib_123:false');
		expect(calls).toContain('refresh:lib_123');
	});

	it('calls load-more when the pager button is clicked', async () => {
		const calls: string[] = [];
		vi.stubGlobal(
			'IntersectionObserver',
			class {
				observe() {}
				disconnect() {}
			}
		);

		render(NotionStatusPanel, {
			props: {
				connection: makeConnection(),
				settings: {
					...settings,
					selection_enabled: true
				},
				items: [
					{
						library_entry_id: 'lib_123',
						title: 'Exported essay',
						item_type: 'article',
						url: 'https://example.com',
						selected: true,
						exported_page_id: 'page_123',
						last_synced_at: null,
						last_error: null
					}
				],
				itemsTotal: 2,
				itemsFilteredCount: 2,
				itemsHasNext: true,
				...panelHandlers(),
				onItemsLoadMore: () => calls.push('load-more')
			}
		});

		await fireEvent.click(screen.getByRole('button', { name: 'Load more' }));

		expect(calls).toEqual(['load-more']);
	});
});
