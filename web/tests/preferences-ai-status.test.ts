import { fireEvent, render, screen, waitFor } from '@testing-library/svelte';
import { beforeEach, describe, expect, it, vi } from 'vitest';

const api = vi.hoisted(() => ({
	createPromptPreset: vi.fn(),
	deletePromptPreset: vi.fn(),
	getConfig: vi.fn(),
	getStatus: vi.fn(),
	listPromptPresets: vi.fn(),
	reindexConfig: vi.fn(),
	testConfig: vi.fn(),
	updatePromptPreset: vi.fn(),
	upsertConfig: vi.fn()
}));

vi.mock('$lib/api', () => api);

import AiPreferencesPage from '../src/routes/(app)/preferences/ai/+page.svelte';

const config = {
	byo_enabled: true,
	chat_api_base: 'http://localhost:18086/v1',
	chat_context_pct: 70,
	chat_model: 'qa-chat',
	cross_item_max_per_item: 3,
	cross_item_top_k: 10,
	embedding_api_base: 'http://localhost:18086/v1',
	embedding_dim: 768,
	embedding_model: 'qa-embed',
	enabled: true,
	has_chat_api_key: true,
	has_embedding_api_key: false,
	model_context_window: 12000,
	supports_reasoning_effort: false,
	supports_structured_output: true,
	top_k: 5
};

function status(overrides: Record<string, unknown> = {}) {
	return {
		enabled: true,
		eligible_items: 8,
		indexed_items: 6,
		is_indexing: true,
		progress_percent: 75,
		reindex_required: true,
		stale_items: 2,
		...overrides
	};
}

describe('Mila indexing status', () => {
	beforeEach(() => {
		vi.clearAllMocks();
		api.getConfig.mockResolvedValue({ data: config });
		api.listPromptPresets.mockResolvedValue({ data: { groups: [] } });
		api.getStatus.mockResolvedValue({ data: status() });
	});

	it('shows live indexed, eligible, stale and active progress', async () => {
		render(AiPreferencesPage);

		const region = await screen.findByRole('status', { name: 'Mila indexing status' });
		expect(region.textContent).toContain('Indexing your library');
		expect(region.textContent).toContain('6 of 8 items indexed');
		expect(region.textContent).toContain('75%');
		expect(region.textContent).toContain('2 stale');
		expect(region.textContent).toContain('qa-embed');
		expect(region.querySelector('progress')?.getAttribute('value')).toBe('75');
	});

	it('offers retry after indexing stops with stale items and refreshes the status', async () => {
		api.getStatus
			.mockResolvedValueOnce({ data: status({ is_indexing: false, progress_percent: 75 }) })
			.mockResolvedValueOnce({
				data: status({
					indexed_items: 8,
					stale_items: 0,
					is_indexing: false,
					progress_percent: 100,
					reindex_required: false
				})
			});
		api.reindexConfig.mockResolvedValue({ data: config });
		render(AiPreferencesPage);

		await fireEvent.click(await screen.findByRole('button', { name: 'Retry indexing' }));

		await waitFor(() => expect(api.reindexConfig).toHaveBeenCalledOnce());
		expect(api.reindexConfig.mock.calls[0][0].body.embedding_model).toBe('qa-embed');
		await waitFor(() =>
			expect(screen.getByRole('status', { name: 'Mila indexing status' }).textContent).toContain(
				'8 of 8 items indexed'
			)
		);
		expect(screen.getByText('Your library is ready')).toBeTruthy();
	});

	it('polls while indexing and stops after completion', async () => {
		vi.useFakeTimers();
		api.getStatus.mockResolvedValueOnce({ data: status() }).mockResolvedValueOnce({
			data: status({
				indexed_items: 8,
				stale_items: 0,
				is_indexing: false,
				progress_percent: 100,
				reindex_required: false
			})
		});
		render(AiPreferencesPage);
		await vi.advanceTimersByTimeAsync(2000);
		await vi.runAllTicks();

		expect(api.getStatus).toHaveBeenCalledTimes(2);
		expect(screen.getByText('Your library is ready')).toBeTruthy();
		await vi.advanceTimersByTimeAsync(4000);
		expect(api.getStatus).toHaveBeenCalledTimes(2);
		vi.useRealTimers();
	});

	it('keeps provider settings usable when status is temporarily unavailable', async () => {
		api.getStatus.mockRejectedValueOnce(new Error('offline'));
		render(AiPreferencesPage);

		expect(await screen.findByText('Use my own AI provider')).toBeTruthy();
		const alert = screen.getByRole('alert');
		expect(alert.textContent).toContain('Can’t reach the index');
		expect(alert.textContent).toContain('The status service did not respond.');
		expect(screen.getByRole('button', { name: 'Try again' })).toBeTruthy();
	});

	it('explains reasoning compatibility without inventing an effort level', async () => {
		render(AiPreferencesPage);

		const capability = await screen.findByRole('checkbox', {
			name: 'Reasoning model compatibility'
		});
		expect((capability as HTMLInputElement).checked).toBe(false);
		const helpId = capability.getAttribute('aria-describedby');
		const help = helpId ? document.getElementById(helpId) : null;
		expect(help?.textContent).toMatch(/per-task sampling controls.*temperature.*top_p/i);
		expect(help?.textContent).toContain('LM Studio 0.4.8 or newer');
		expect(screen.queryByText(/Provider supports reasoning_effort/)).toBeNull();

		await fireEvent.click(capability);
		expect(help?.textContent).toMatch(
			/No reasoning_effort value is sent.*provider uses its default reasoning level/i
		);
	});

	it('shows indexing as paused without offering retry while Mila is disabled', async () => {
		api.getStatus.mockResolvedValue({
			data: status({ enabled: false, is_indexing: false, progress_percent: 75 })
		});
		render(AiPreferencesPage);

		expect(await screen.findByText('Indexing is paused')).toBeTruthy();
		expect(screen.queryByRole('button', { name: 'Retry indexing' })).toBeNull();
	});

	it('always saves provider changes through the normal config endpoint', async () => {
		api.upsertConfig.mockResolvedValue({
			data: { ...config, embedding_model: 'embedding-gemma' }
		});
		render(AiPreferencesPage);

		const model = await screen.findByRole('textbox', { name: 'Embedding model ID' });
		await fireEvent.input(model, { target: { value: 'embedding-gemma' } });
		await fireEvent.click(screen.getByRole('button', { name: 'Save' }));

		await waitFor(() => expect(api.upsertConfig).toHaveBeenCalledOnce());
		expect(api.reindexConfig).not.toHaveBeenCalled();
		expect(api.upsertConfig.mock.calls[0][0].body.embedding_model).toBe('embedding-gemma');
	});

	it('keeps persisted status authoritative while provider edits are unsaved', async () => {
		render(AiPreferencesPage);
		const region = await screen.findByRole('status', { name: 'Mila indexing status' });

		const model = screen.getByRole('textbox', { name: 'Embedding model ID' });
		await fireEvent.input(model, { target: { value: 'embedding-gemma' } });

		expect(region.textContent).toContain('6 of 8 items indexed');
		expect(region.textContent).toContain('qa-embed');
		expect(region.textContent).not.toContain('embedding-gemma');
	});

	it('does not label platform indexing with the inactive stored BYO model', async () => {
		api.getConfig.mockResolvedValue({ data: { ...config, byo_enabled: false } });
		render(AiPreferencesPage);

		const region = await screen.findByRole('status', { name: 'Mila indexing status' });
		expect(region.textContent).toContain('Platform default');
		expect(region.textContent).not.toContain('qa-embed');
	});

	it('does not declare readiness while the server still requires reindexing', async () => {
		api.getStatus.mockResolvedValue({
			data: status({
				indexed_items: 8,
				stale_items: 0,
				is_indexing: false,
				progress_percent: 100,
				reindex_required: true
			})
		});
		render(AiPreferencesPage);

		expect(await screen.findByText('Indexing stopped early')).toBeTruthy();
		expect(screen.queryByText('Your library is ready')).toBeNull();
	});
});
