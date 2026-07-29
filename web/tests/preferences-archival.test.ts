import { describe, it, expect, vi, beforeEach } from 'vitest';
import { fireEvent, render, screen, waitFor } from '@testing-library/svelte';

import type { ArchivalSettingsResponse } from '$lib/api/generated/types.gen';

const loadedSettings = (): ArchivalSettingsResponse => ({
	archive_formats: {
		readable_html: true,
		monolith: true,
		pdf: false,
		screenshot: true,
		warc: false
	},
	duplicate_detection: {
		enabled: true,
		sensitivity: 'medium',
		on_duplicate: 'notify_me'
	},
	processing: {
		browser_timeout_secs: 90,
		max_concurrent_archives: 2,
		ai_auto_processing: true
	},
	proxy: {
		url: undefined,
		all_requests: false
	}
});

const { loadArchivalSettings, saveArchivalSettings } = vi.hoisted(() => ({
	loadArchivalSettings: vi.fn(),
	saveArchivalSettings: vi.fn()
}));

vi.mock('$lib/api/settings', () => ({
	loadArchivalSettings,
	saveArchivalSettings
}));

import ArchivalPage from '../src/routes/(app)/preferences/archival/+page.svelte';

describe('Archival preferences page', () => {
	beforeEach(() => {
		vi.clearAllMocks();
		loadArchivalSettings.mockResolvedValue({ success: true, data: loadedSettings() });
		saveArchivalSettings.mockImplementation(async (body: ArchivalSettingsResponse) => ({
			success: true,
			data: body
		}));
	});

	it('saves the toggled pdf archive flag through the API', async () => {
		render(ArchivalPage);

		await waitFor(() => expect(loadArchivalSettings).toHaveBeenCalledOnce());

		await fireEvent.click(screen.getByRole('switch', { name: 'PDF Snapshot' }));
		await fireEvent.click(screen.getByRole('button', { name: 'Save' }));

		await waitFor(() => expect(saveArchivalSettings).toHaveBeenCalledOnce());
		expect(saveArchivalSettings.mock.calls[0]?.[0].archive_formats.pdf).toBe(true);
	});

	it('does not send all-request proxy routing without a proxy URL', async () => {
		loadArchivalSettings.mockResolvedValue({
			success: true,
			data: {
				...loadedSettings(),
				proxy: {
					url: 'https://proxy.example.test',
					all_requests: true
				}
			}
		});

		render(ArchivalPage);

		await waitFor(() => expect(loadArchivalSettings).toHaveBeenCalledOnce());

		const proxyInput = screen.getByPlaceholderText('socks5://127.0.0.1:1080');
		await fireEvent.input(proxyInput, { target: { value: '' } });
		await fireEvent.click(screen.getByRole('button', { name: 'Save' }));

		await waitFor(() => expect(saveArchivalSettings).toHaveBeenCalledOnce());
		expect(saveArchivalSettings.mock.calls[0]?.[0].proxy).toEqual({
			url: null,
			all_requests: false
		});
	});
});
