export type ThemePreference = 'light' | 'dark' | 'system';

const STORAGE_KEY = 'theme';

export function applyTheme(preference: ThemePreference): void {
	const root = document.documentElement;
	if (preference === 'system') {
		const isDark = window.matchMedia('(prefers-color-scheme: dark)').matches;
		root.dataset.theme = isDark ? 'dark' : 'light';
	} else {
		root.dataset.theme = preference;
	}
}

export function saveTheme(preference: ThemePreference): void {
	localStorage.setItem(STORAGE_KEY, preference);
	applyTheme(preference);
}

export function getSavedTheme(): ThemePreference {
	const saved = localStorage.getItem(STORAGE_KEY);
	if (saved === 'light' || saved === 'dark' || saved === 'system') {
		return saved;
	}
	return 'system';
}

export function initTheme(): void {
	applyTheme(getSavedTheme());

	window.matchMedia('(prefers-color-scheme: dark)').addEventListener('change', () => {
		const current = getSavedTheme();
		if (current === 'system') {
			applyTheme('system');
		}
	});
}
