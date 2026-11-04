import { beforeEach, describe, expect, it, vi } from 'vitest';

import { loadPreferencesSettings, savePreferencesSettings } from '$lib/api/settings';
import { isDarkTheme, toggleTheme } from '$lib/styles/theme-toggle';

vi.mock('$lib/api/settings', () => ({
	loadPreferencesSettings: vi.fn(),
	savePreferencesSettings: vi.fn().mockResolvedValue({ success: true })
}));

describe('toggleTheme', () => {
	beforeEach(() => {
		localStorage.clear();
		document.documentElement.dataset.theme = 'light';
		vi.mocked(loadPreferencesSettings).mockResolvedValue({
			success: true,
			data: { theme: 'light', font_size: 'medium' } as never
		});
		vi.mocked(savePreferencesSettings).mockClear();
	});

	it('flips the document theme, persists it, and mirrors it to the account', async () => {
		await toggleTheme();

		expect(isDarkTheme()).toBe(true);
		expect(localStorage.getItem('theme')).toBe('dark');
		expect(savePreferencesSettings).toHaveBeenCalledWith({ theme: 'dark', font_size: 'medium' });

		await toggleTheme();
		expect(isDarkTheme()).toBe(false);
		expect(savePreferencesSettings).toHaveBeenLastCalledWith({
			theme: 'light',
			font_size: 'medium'
		});
	});

	it('waits for a slow save before writing the newer theme, so the last write wins', async () => {
		let releaseFirstSave!: () => void;
		vi.mocked(savePreferencesSettings)
			.mockReturnValueOnce(
				new Promise((resolve) => {
					releaseFirstSave = () => resolve({ success: true } as never);
				})
			)
			.mockResolvedValue({ success: true } as never);

		const first = toggleTheme();
		await vi.waitFor(() => expect(savePreferencesSettings).toHaveBeenCalledOnce());

		const second = toggleTheme();
		expect(isDarkTheme()).toBe(false);
		await Promise.resolve();
		await Promise.resolve();
		expect(savePreferencesSettings).toHaveBeenCalledOnce();

		releaseFirstSave();
		await Promise.all([first, second]);

		expect(savePreferencesSettings).toHaveBeenCalledTimes(2);
		expect(savePreferencesSettings).toHaveBeenLastCalledWith({
			theme: 'light',
			font_size: 'medium'
		});
	});

	it('keeps the local flip when the account preferences cannot be loaded', async () => {
		vi.mocked(loadPreferencesSettings).mockResolvedValue({ success: false, error: 'offline' });

		await toggleTheme();

		expect(isDarkTheme()).toBe(true);
		expect(savePreferencesSettings).not.toHaveBeenCalled();
	});
});
