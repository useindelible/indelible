import { describe, it, expect, beforeEach, vi, afterEach } from 'vitest';

vi.mock('$app/environment', () => ({
	browser: true
}));

const storageMap = new Map<string, string>();
const mockLocalStorage = {
	getItem: (key: string) => storageMap.get(key) ?? null,
	setItem: (key: string, value: string) => storageMap.set(key, value),
	removeItem: (key: string) => storageMap.delete(key),
	clear: () => storageMap.clear(),
	get length() {
		return storageMap.size;
	},
	key: (index: number) => [...storageMap.keys()][index] ?? null
};

describe('reader-preferences', () => {
	beforeEach(() => {
		storageMap.clear();
		vi.resetModules();
		vi.stubGlobal('localStorage', mockLocalStorage);
	});

	afterEach(() => {
		vi.unstubAllGlobals();
	});

	async function loadModule() {
		const mod = await import('$lib/stores/reader-preferences.svelte');
		return mod;
	}

	it('returns default values when no localStorage data exists', async () => {
		const { getReaderPreferences } = await loadModule();
		const prefs = getReaderPreferences();
		expect(prefs.theme).toBe('auto');
		expect(prefs.typeface).toBe('sans');
		expect(prefs.fontSize).toBe(18);
		expect(prefs.lineHeight).toBe(1.75);
		expect(prefs.contentWidth).toBe(760);
		expect(prefs.paragraphSpacing).toBe(1.2);
		expect(prefs.textAlign).toBe('left');
	});

	it('persists changes to localStorage', async () => {
		const { getReaderPreferences } = await loadModule();
		const prefs = getReaderPreferences();
		prefs.fontSize = 22;

		const stored = JSON.parse(mockLocalStorage.getItem('indelible:reader-preferences')!);
		expect(stored.fontSize).toBe(22);
	});

	it('restores values from localStorage', async () => {
		mockLocalStorage.setItem(
			'indelible:reader-preferences',
			JSON.stringify({
				theme: 'dark',
				typeface: 'serif',
				fontSize: 24,
				lineHeight: 2.0,
				contentWidth: 720,
				paragraphSpacing: 1.6,
				textAlign: 'justify'
			})
		);

		const { getReaderPreferences } = await loadModule();
		const prefs = getReaderPreferences();
		expect(prefs.theme).toBe('dark');
		expect(prefs.typeface).toBe('serif');
		expect(prefs.fontSize).toBe(24);
		expect(prefs.lineHeight).toBe(2.0);
		expect(prefs.contentWidth).toBe(720);
		expect(prefs.paragraphSpacing).toBe(1.6);
		expect(prefs.textAlign).toBe('justify');
	});

	it('uses defaults for missing localStorage fields', async () => {
		mockLocalStorage.setItem('indelible:reader-preferences', JSON.stringify({ theme: 'sepia' }));

		const { getReaderPreferences } = await loadModule();
		const prefs = getReaderPreferences();
		expect(prefs.theme).toBe('sepia');
		expect(prefs.fontSize).toBe(18);
	});

	it('handles corrupted localStorage gracefully', async () => {
		mockLocalStorage.setItem('indelible:reader-preferences', 'not-json');

		const { getReaderPreferences } = await loadModule();
		const prefs = getReaderPreferences();
		expect(prefs.fontSize).toBe(18);
	});

	it('returns correct fontFamily for each typeface', async () => {
		const { getReaderPreferences } = await loadModule();
		const prefs = getReaderPreferences();

		prefs.typeface = 'serif';
		expect(prefs.fontFamily).toContain('Georgia');

		prefs.typeface = 'sans';
		expect(prefs.fontFamily).toContain('BlinkMacSystemFont');

		prefs.typeface = 'mono';
		expect(prefs.fontFamily).toContain('SF Mono');
	});

	it('reset restores all defaults', async () => {
		const { getReaderPreferences } = await loadModule();
		const prefs = getReaderPreferences();
		prefs.theme = 'dark';
		prefs.fontSize = 28;
		prefs.typeface = 'mono';

		prefs.reset();

		expect(prefs.theme).toBe('auto');
		expect(prefs.fontSize).toBe(18);
		expect(prefs.typeface).toBe('sans');
	});

	it('exposes defaults object', async () => {
		const { getReaderPreferences } = await loadModule();
		const prefs = getReaderPreferences();
		expect(prefs.defaults.fontSize).toBe(18);
		expect(prefs.defaults.contentWidth).toBe(760);
	});
});
