import { locale, setupI18nSync } from '$lib/i18n';
import { beforeEach, describe, expect, it } from 'vitest';

import en from '../i18n/locales/en.json';
import fr from '../i18n/locales/fr.json';
import { relativeTime } from './relative-time';

const NOW = Date.UTC(2026, 0, 1);
const ago = (milliseconds: number) => new Date(NOW - milliseconds).toISOString();

describe('relativeTime', () => {
	beforeEach(() => setupI18nSync({ en, fr }));

	it.each([
		[30_000, 'just now'],
		[5 * 60_000, '5 minutes ago'],
		[60 * 60_000, '1 hour ago'],
		[3 * 24 * 60 * 60_000, '3 days ago'],
		[60 * 24 * 60 * 60_000, '2 months ago'],
		[365 * 24 * 60 * 60_000, '1 year ago']
	])('formats an English duration', (duration, expected) => {
		expect(relativeTime(ago(duration), NOW)).toBe(expected);
	});

	it('formats French relative time', () => {
		locale.set('fr');
		expect(relativeTime(ago(30_000), NOW)).toBe("à l'instant");
		expect(relativeTime(ago(5 * 60_000), NOW)).toBe('il y a 5 minutes');
	});

	it('returns null for absent or invalid timestamps', () => {
		expect(relativeTime(null, NOW)).toBeNull();
		expect(relativeTime('not-a-date', NOW)).toBeNull();
	});
});
