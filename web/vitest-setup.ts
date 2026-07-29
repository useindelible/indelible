import '@testing-library/svelte/vitest';

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
