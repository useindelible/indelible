import { describe, expect, test } from 'vitest';

import {
	hasScrollableOverflow,
	scrollProgressPercent
} from '$lib/components/reader/progress-geometry';

describe('reader progress geometry', () => {
	test('treats content that fits the viewport as fully visible', () => {
		expect(scrollProgressPercent({ scrollTop: 0, scrollHeight: 420, clientHeight: 840 })).toBe(100);
	});

	test('treats tiny layout rounding overflow as unscrollable', () => {
		expect(scrollProgressPercent({ scrollTop: 0, scrollHeight: 840.5, clientHeight: 840 })).toBe(
			100
		);
		expect(hasScrollableOverflow({ scrollHeight: 840.5, clientHeight: 840 })).toBe(false);
	});

	test('calculates progress for scrollable content', () => {
		expect(scrollProgressPercent({ scrollTop: 250, scrollHeight: 1500, clientHeight: 500 })).toBe(
			25
		);
	});

	test('clamps scroll progress into the valid percent range', () => {
		expect(scrollProgressPercent({ scrollTop: -100, scrollHeight: 1500, clientHeight: 500 })).toBe(
			0
		);
		expect(scrollProgressPercent({ scrollTop: 1200, scrollHeight: 1500, clientHeight: 500 })).toBe(
			100
		);
	});
});
