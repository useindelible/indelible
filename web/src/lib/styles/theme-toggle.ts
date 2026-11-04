import { loadPreferencesSettings, savePreferencesSettings } from '$lib/api/settings';
import { saveTheme, type ThemePreference } from './theme';

export function isDarkTheme(): boolean {
	return document.documentElement.dataset.theme === 'dark';
}

let requested: ThemePreference | null = null;
let persisting: Promise<void> | null = null;

/**
 * Flips the theme locally at once. Account persistence runs one write at a
 * time and always carries the newest request, so rapid toggles cannot land out of order.
 */
export function toggleTheme(): Promise<void> {
	const next = isDarkTheme() ? 'light' : 'dark';
	saveTheme(next);
	requested = next;
	persisting ??= persistRequested();
	return persisting;
}

async function persistRequested(): Promise<void> {
	try {
		while (requested) {
			const result = await loadPreferencesSettings();
			const theme = requested;
			requested = null;
			if (!theme) break;
			if (result.success) await savePreferencesSettings({ ...result.data, theme });
		}
	} finally {
		persisting = null;
	}
}
