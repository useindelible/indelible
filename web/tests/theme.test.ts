import { describe, it, expect, beforeEach, afterEach, vi } from 'vitest';

const store = new Map<string, string>();
const localStorageMock: Storage = {
	getItem: (key: string) => store.get(key) ?? null,
	setItem: (key: string, value: string) => {
		store.set(key, value);
	},
	removeItem: (key: string) => {
		store.delete(key);
	},
	clear: () => {
		store.clear();
	},
	get length() {
		return store.size;
	},
	key: (index: number) => [...store.keys()][index] ?? null
};

function stubLocalStorage() {
	vi.stubGlobal('localStorage', localStorageMock);
}

stubLocalStorage();

import { applyTheme, getSavedTheme, saveTheme, initTheme } from '$lib/styles/theme';

describe('applyTheme', () => {
	beforeEach(() => {
		stubLocalStorage();
		document.documentElement.removeAttribute('data-theme');
	});

	it('sets data-theme to light for light preference', () => {
		applyTheme('light');
		expect(document.documentElement.dataset.theme).toBe('light');
	});

	it('sets data-theme to dark for dark preference', () => {
		applyTheme('dark');
		expect(document.documentElement.dataset.theme).toBe('dark');
	});

	it('reads prefers-color-scheme for system preference', () => {
		const matchMediaMock = vi.fn().mockReturnValue({ matches: true } as MediaQueryList);
		vi.stubGlobal('matchMedia', matchMediaMock);

		applyTheme('system');
		expect(document.documentElement.dataset.theme).toBe('dark');

		matchMediaMock.mockReturnValue({ matches: false } as MediaQueryList);
		applyTheme('system');
		expect(document.documentElement.dataset.theme).toBe('light');

		vi.unstubAllGlobals();
	});
});

describe('saveTheme / getSavedTheme', () => {
	beforeEach(() => {
		stubLocalStorage();
		store.clear();
		document.documentElement.removeAttribute('data-theme');
	});

	it('persists preference to localStorage', () => {
		saveTheme('dark');
		expect(localStorage.getItem('theme')).toBe('dark');
	});

	it('applies theme when saving', () => {
		saveTheme('dark');
		expect(document.documentElement.dataset.theme).toBe('dark');
	});

	it('returns system when no saved preference', () => {
		expect(getSavedTheme()).toBe('system');
	});

	it('returns saved preference when present', () => {
		localStorage.setItem('theme', 'light');
		expect(getSavedTheme()).toBe('light');
	});

	it('returns system for invalid stored values', () => {
		localStorage.setItem('theme', 'invalid');
		expect(getSavedTheme()).toBe('system');
	});
});

describe('initTheme', () => {
	beforeEach(() => {
		stubLocalStorage();
		store.clear();
		document.documentElement.removeAttribute('data-theme');
	});

	afterEach(() => {
		vi.unstubAllGlobals();
	});

	it('applies saved theme on init', () => {
		const matchMediaMock = vi.fn().mockReturnValue({
			matches: false,
			addEventListener: vi.fn()
		} as unknown as MediaQueryList);
		vi.stubGlobal('matchMedia', matchMediaMock);

		localStorage.setItem('theme', 'dark');
		initTheme();
		expect(document.documentElement.dataset.theme).toBe('dark');
	});

	it('applies system theme when no preference saved', () => {
		const listeners: Array<() => void> = [];
		const matchMediaMock = vi.fn().mockReturnValue({
			matches: false,
			addEventListener: (_event: string, cb: () => void) => listeners.push(cb)
		} as unknown as MediaQueryList);
		vi.stubGlobal('matchMedia', matchMediaMock);

		initTheme();
		expect(document.documentElement.dataset.theme).toBe('light');
	});

	it('listens for system color scheme changes', () => {
		const listeners: Array<() => void> = [];
		const matchMediaMock = vi.fn().mockReturnValue({
			matches: false,
			addEventListener: (_event: string, cb: () => void) => listeners.push(cb)
		} as unknown as MediaQueryList);
		vi.stubGlobal('matchMedia', matchMediaMock);

		initTheme();
		expect(listeners.length).toBe(1);

		matchMediaMock.mockReturnValue({
			matches: true,
			addEventListener: (_event: string, cb: () => void) => listeners.push(cb)
		} as unknown as MediaQueryList);

		expect(listeners[0]).toBeDefined();
		listeners[0]!();
		expect(document.documentElement.dataset.theme).toBe('dark');
	});

	it('does not react to system changes when preference is explicit', () => {
		const listeners: Array<() => void> = [];
		const matchMediaMock = vi.fn().mockReturnValue({
			matches: false,
			addEventListener: (_event: string, cb: () => void) => listeners.push(cb)
		} as unknown as MediaQueryList);
		vi.stubGlobal('matchMedia', matchMediaMock);

		localStorage.setItem('theme', 'light');
		initTheme();
		expect(document.documentElement.dataset.theme).toBe('light');

		matchMediaMock.mockReturnValue({
			matches: true,
			addEventListener: (_event: string, cb: () => void) => listeners.push(cb)
		} as unknown as MediaQueryList);

		expect(listeners[0]).toBeDefined();
		listeners[0]!();
		expect(document.documentElement.dataset.theme).toBe('light');
	});
});
