import type { HomeItemResponse } from '$lib/api';
import { formatReadingTime } from '$lib/utils/format';

export interface DashboardConfigItem {
	id: string;
	label: string;
	on: boolean;
}

export const DEFAULT_CONFIG_SECTIONS: DashboardConfigItem[] = [
	{ id: 'continue', label: 'Continue Reading', on: true },
	{ id: 'quick', label: 'Quick Reads', on: true },
	{ id: 'long', label: 'Long Reads', on: true },
	{ id: 'review', label: 'Daily Review', on: true },
	{ id: 'recent', label: 'Recently Added', on: true },
	{ id: 'highlights', label: 'Recently Highlighted', on: true },
	{ id: 'stats', label: 'Reading Stats', on: false }
];

export const DEFAULT_CONFIG_TYPES: DashboardConfigItem[] = [
	{ id: 'articles', label: 'Articles', on: true },
	{ id: 'books', label: 'Books', on: true },
	{ id: 'emails', label: 'Emails', on: true },
	{ id: 'pdfs', label: 'PDFs', on: true },
	{ id: 'tweets', label: 'Tweets', on: true },
	{ id: 'videos', label: 'Videos', on: true },
	{ id: 'podcasts', label: 'Podcasts', on: true },
	{ id: 'feeds', label: 'Feeds', on: true }
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

export function greetingForHour(hour: number): string {
	if (hour < 12) return 'Good morning';
	if (hour < 18) return 'Good afternoon';
	return 'Good evening';
}

export function greetingLine(
	displayName: string | null | undefined,
	hour = new Date().getHours()
): string {
	const greeting = greetingForHour(hour);
	const name = displayName?.trim();
	return name ? `${greeting}, ${name}.` : `${greeting}.`;
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

export function readingMeta(item: HomeItemResponse): string {
	const parts: string[] = [];
	const author = item.author ? item.author.replace(/\s*@\S+$/, '').trim() : '';
	if (author) parts.push(author);
	if (item.reading_time_minutes) parts.push(`${formatReadingTime(item.reading_time_minutes)} read`);
	return parts.join(' · ');
}
