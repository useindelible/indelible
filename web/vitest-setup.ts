import '@testing-library/svelte/vitest';
import en from './src/lib/i18n/locales/en.json';
import { setupI18nSync } from './src/lib/i18n';

setupI18nSync({ en });

if (typeof window !== 'undefined' && !window.localStorage) {
	const values = new Map<string, string>();
	Object.defineProperty(window, 'localStorage', {
		configurable: true,
		value: {
			clear: () => values.clear(),
			getItem: (key: string) => values.get(key) ?? null,
			key: (index: number) => [...values.keys()][index] ?? null,
			get length() {
				return values.size;
			},
			removeItem: (key: string) => values.delete(key),
			setItem: (key: string, value: string) => values.set(key, String(value))
		} satisfies Storage
	});
}

// jsdom ships <dialog> without the modal methods components call on open.
if (typeof HTMLDialogElement !== 'undefined' && !HTMLDialogElement.prototype.showModal) {
	HTMLDialogElement.prototype.showModal = function showModal(this: HTMLDialogElement) {
		this.open = true;
	};
}

if (typeof HTMLDialogElement !== 'undefined' && !HTMLDialogElement.prototype.close) {
	HTMLDialogElement.prototype.close = function close(this: HTMLDialogElement) {
		this.open = false;
		this.dispatchEvent(new Event('close'));
	};
}

if (typeof window !== 'undefined' && !window.matchMedia) {
	Object.defineProperty(window, 'matchMedia', {
		writable: true,
		value: (query: string): MediaQueryList => ({
			matches: false,
			media: query,
			onchange: null,
			addListener: () => {},
			removeListener: () => {},
			addEventListener: () => {},
			removeEventListener: () => {},
			dispatchEvent: () => false
		})
	});
}
