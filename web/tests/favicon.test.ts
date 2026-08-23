import { existsSync, readFileSync } from 'node:fs';
import { resolve } from 'node:path';

import { describe, expect, it } from 'vitest';

describe('application favicon', () => {
	it('uses the shipped Indelible brand mark', () => {
		const appHtml = readFileSync(resolve('src/app.html'), 'utf8');
		const faviconPath = 'brand/indelible-mark-favicon.svg';

		expect(appHtml).toContain(`href="%sveltekit.assets%/${faviconPath}"`);
		expect(existsSync(resolve('static', faviconPath))).toBe(true);
	});

	it('fills the favicon canvas', () => {
		const svg = readFileSync(resolve('static/brand/indelible-mark-favicon.svg'), 'utf8');
		const viewBox = svg
			.match(/viewBox="([\d.\s]+)"/)?.[1]
			.split(/\s+/)
			.map(Number);
		const tileWidth = Number(svg.match(/<rect[^>]*width="([\d.]+)"/)?.[1]);

		expect(viewBox).toHaveLength(4);
		expect(tileWidth / viewBox![2]).toBeGreaterThanOrEqual(0.9);
	});
});
