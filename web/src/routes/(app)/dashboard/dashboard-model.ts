import type { HomeItemResponse } from '$lib/api';
import type { MessageKey, Translate } from '$lib/i18n';
import { formatReadingTime } from '$lib/utils/format';

export interface DashboardConfigItem {
	id: string;
	labelKey: MessageKey;
	on: boolean;
}

export const DEFAULT_CONFIG_SECTIONS: DashboardConfigItem[] = [
	{ id: 'continue', labelKey: 'dashboard_section_continue', on: true },
	{ id: 'quick', labelKey: 'dashboard_section_quick', on: true },
	{ id: 'long', labelKey: 'dashboard_section_long', on: true },
	{ id: 'review', labelKey: 'dashboard_section_review', on: true },
	{ id: 'recent', labelKey: 'dashboard_section_recent', on: true },
	{ id: 'highlights', labelKey: 'dashboard_section_highlights', on: true },
	{ id: 'stats', labelKey: 'dashboard_section_stats', on: false }
];

export const DEFAULT_CONFIG_TYPES: DashboardConfigItem[] = [
	{ id: 'articles', labelKey: 'dashboard_type_articles', on: true },
	{ id: 'books', labelKey: 'dashboard_type_books', on: true },
	{ id: 'emails', labelKey: 'dashboard_type_emails', on: true },
	{ id: 'pdfs', labelKey: 'dashboard_type_pdfs', on: true },
	{ id: 'tweets', labelKey: 'dashboard_type_tweets', on: true },
	{ id: 'videos', labelKey: 'dashboard_type_videos', on: true },
	{ id: 'feeds', labelKey: 'dashboard_type_feeds', on: true }
];

export const COVER_COLORS = ['blue', 'green', 'purple', 'orange', 'red', 'teal', 'pink'] as const;
export type CoverColor = (typeof COVER_COLORS)[number];

export function cloneConfig(items: DashboardConfigItem[]): DashboardConfigItem[] {
	return items.map((item) => ({ ...item }));
}

export function reorder<T>(arr: T[], from: number, to: number): T[] {
	const next = [...arr];
	const [item] = next.splice(from, 1);
	if (item !== undefined) next.splice(to, 0, item);
	return next;
}

export function greetingKeyForHour(hour: number, named: boolean): MessageKey {
	if (hour < 12) return named ? 'dashboard_greeting_morning_named' : 'dashboard_greeting_morning';
	if (hour < 18) {
		return named ? 'dashboard_greeting_afternoon_named' : 'dashboard_greeting_afternoon';
	}
	return named ? 'dashboard_greeting_evening_named' : 'dashboard_greeting_evening';
}

export function greetingLine(
	translate: Translate,
	displayName: string | null | undefined,
	hour = new Date().getHours()
): string {
	const name = displayName?.trim();
	const key = greetingKeyForHour(hour, Boolean(name));
	return name ? translate(key, { values: { name } }) : translate(key);
}

export function longReadItems(items: HomeItemResponse[]): HomeItemResponse[] {
	return items.filter((item) => (item.reading_time_minutes ?? 0) >= 20);
}

export function coverColor(domain: string | null | undefined): CoverColor {
	const colorIndex = (domain ?? '?').charCodeAt(0) % COVER_COLORS.length;
	return COVER_COLORS[colorIndex] ?? 'blue';
}

export function domainInitial(domain: string | null | undefined): string {
	return (domain ?? '?').charAt(0).toUpperCase();
}

export function readingMeta(translate: Translate, item: HomeItemResponse): string {
	const parts: string[] = [];
	const author = item.author ? item.author.replace(/\s*@\S+$/, '').trim() : '';
	if (author) parts.push(author);
	if (item.reading_time_minutes) {
		parts.push(
			translate('dashboard_reading_meta', {
				values: { duration: formatReadingTime(item.reading_time_minutes) }
			})
		);
	}
	return parts.join(' · ');
}
