import { beforeEach, describe, expect, it, vi } from 'vitest';
import { fireEvent, render, screen, waitFor } from '@testing-library/svelte';
import type { ObsidianPreviewResponse, ObsidianSettingsDto } from '$lib/api';

const apiMocks = vi.hoisted(() => ({
	loadIntegrationConnections: vi.fn(),
	loadObsidianSettings: vi.fn(),
	previewObsidianSettings: vi.fn(),
	saveObsidianSettings: vi.fn(),
	setupObsidianExportConnection: vi.fn()
}));

vi.mock('$lib/api/integrations', () => apiMocks);

import ObsidianPage from '../src/routes/(app)/preferences/integrations/obsidian/+page.svelte';

function settings(overrides: Partial<ObsidianSettingsDto> = {}): ObsidianSettingsDto {
	return {
		category_folder_templates: {
			articles: 'articles',
			books: 'books',
			podcasts: 'podcasts',
			tweets: 'tweets'
		},
		export_all_reader_documents: false,
		file_name_template: '{{title}}',
		group_files_in_category_folders: true,
		highlight_header_template: '## Highlights',
		highlight_template: '- {{highlight_text}}',
		metadata_template: 'URL: {{url}}',
		page_title_template: '# {{title}}',
		properties_template: '',
		sync_notification_template: '- synced {{document_count}}',
		sync_notifications: false,
		...overrides
	};
}

function preview(): ObsidianPreviewResponse {
	return {
		file_path: 'Indelible/articles/Sample.md',
		full_content: '# Sample\n\nBody',
		full_document_text: null,
		full_document_text_path: null
	};
}

function deferred<T>() {
	let resolve!: (value: T) => void;
	const promise = new Promise<T>((done) => {
		resolve = done;
	});
	return { promise, resolve };
}

function fileNameInput(): HTMLInputElement {
	return screen.getByPlaceholderText('{{title}}') as HTMLInputElement;
}

function saveButton(): HTMLButtonElement {
	return screen.getByRole('button', { name: 'Save' }) as HTMLButtonElement;
}

async function renderLoadedPage() {
	render(ObsidianPage);
	await waitFor(() => expect(apiMocks.previewObsidianSettings).toHaveBeenCalledOnce());
	await waitFor(() => expect(screen.getByText('Indelible/articles/Sample.md')).toBeTruthy());
}

describe('Obsidian preferences validation', () => {
	beforeEach(() => {
		vi.clearAllMocks();
		apiMocks.loadIntegrationConnections.mockResolvedValue({
			success: true,
			data: {
				available_oauth_providers: [],
				connections: [
					{
						config: {
							provider: 'obsidian',
							export_all_reader_documents: false,
							group_files_in_category_folders: true,
							sync_notifications: false
						},
						created_at: '2026-08-01T00:00:00Z',
						id: 'icn_obsidian',
						last_error: null,
						last_sync_at: null,
						pending_jobs: 0,
						provider: 'obsidian',
						status: 'active'
					}
				]
			}
		});
		apiMocks.loadObsidianSettings.mockResolvedValue({ success: true, data: settings() });
		apiMocks.previewObsidianSettings.mockResolvedValue({ success: true, data: preview() });
		apiMocks.saveObsidianSettings.mockResolvedValue({ success: true, data: settings() });
	});

	it('keeps Save disabled until the current corrected draft has a successful preview', async () => {
		await renderLoadedPage();

		await fireEvent.input(fileNameInput(), { target: { value: '{{ title' } });
		expect(saveButton().disabled).toBe(true);

		apiMocks.previewObsidianSettings.mockResolvedValueOnce({
			success: false,
			error: 'failed to render template `file_name`: unexpected end of input'
		});
		await waitFor(() => expect(apiMocks.previewObsidianSettings).toHaveBeenCalledTimes(2), {
			timeout: 1200
		});
		await waitFor(() =>
			expect(screen.getByText(/failed to render template `file_name`/)).toBeTruthy()
		);
		expect(saveButton().disabled).toBe(true);
		expect((screen.getByRole('button', { name: 'Discard' }) as HTMLButtonElement).disabled).toBe(
			false
		);

		await fireEvent.input(fileNameInput(), { target: { value: '{{title}} - corrected' } });
		expect(saveButton().disabled).toBe(true);
		await waitFor(() => expect(apiMocks.previewObsidianSettings).toHaveBeenCalledTimes(3), {
			timeout: 1200
		});
		await waitFor(() => expect(saveButton().disabled).toBe(false));
	});

	it('does not let a stale successful preview validate a newer pending draft', async () => {
		await renderLoadedPage();
		const stale = deferred<{ success: true; data: ObsidianPreviewResponse }>();
		const current = deferred<{ success: true; data: ObsidianPreviewResponse }>();
		apiMocks.previewObsidianSettings
			.mockImplementationOnce(() => stale.promise)
			.mockImplementationOnce(() => current.promise);

		await fireEvent.input(fileNameInput(), { target: { value: 'first draft' } });
		await waitFor(() => expect(apiMocks.previewObsidianSettings).toHaveBeenCalledTimes(2), {
			timeout: 1200
		});
		await fireEvent.input(fileNameInput(), { target: { value: 'newer draft' } });
		await waitFor(() => expect(apiMocks.previewObsidianSettings).toHaveBeenCalledTimes(3), {
			timeout: 1200
		});

		stale.resolve({ success: true, data: preview() });
		await Promise.resolve();
		expect(saveButton().disabled).toBe(true);

		current.resolve({ success: true, data: preview() });
		await waitFor(() => expect(saveButton().disabled).toBe(false));
	});

	it('disables Save while manually re-rendering an already validated draft', async () => {
		await renderLoadedPage();
		await fireEvent.input(fileNameInput(), { target: { value: 'validated draft' } });
		await waitFor(() => expect(apiMocks.previewObsidianSettings).toHaveBeenCalledTimes(2), {
			timeout: 1200
		});
		await waitFor(() => expect(saveButton().disabled).toBe(false));

		const rerender = deferred<{ success: true; data: ObsidianPreviewResponse }>();
		apiMocks.previewObsidianSettings.mockImplementationOnce(() => rerender.promise);
		await fireEvent.click(screen.getByRole('button', { name: /re-render/i }));
		await waitFor(() => expect(apiMocks.previewObsidianSettings).toHaveBeenCalledTimes(3));

		expect(saveButton().disabled).toBe(true);
		rerender.resolve({ success: true, data: preview() });
		await waitFor(() => expect(saveButton().disabled).toBe(false));
	});

	it('retains the dirty draft and baseline after Save is rejected', async () => {
		await renderLoadedPage();
		apiMocks.saveObsidianSettings.mockResolvedValueOnce({
			success: false,
			error: 'settings version conflict'
		});

		await fireEvent.input(fileNameInput(), { target: { value: 'my custom title' } });
		await waitFor(() => expect(apiMocks.previewObsidianSettings).toHaveBeenCalledTimes(2), {
			timeout: 1200
		});
		await waitFor(() => expect(saveButton().disabled).toBe(false));
		await fireEvent.click(saveButton());

		await waitFor(() => expect(screen.getByText('settings version conflict')).toBeTruthy());
		expect(fileNameInput().value).toBe('my custom title');
		expect((screen.getByRole('button', { name: 'Discard' }) as HTMLButtonElement).disabled).toBe(
			false
		);

		await fireEvent.click(screen.getByRole('button', { name: 'Discard' }));
		expect(fileNameInput().value).toBe('{{title}}');
	});
});
