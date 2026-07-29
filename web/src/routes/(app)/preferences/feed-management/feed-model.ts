import type { FeedSubscriptionResponse } from '$lib/api';

export type FeedStatus = 'active' | 'error' | 'paused';
export type FilterChip = 'all' | FeedStatus;

export interface Feed {
	id: string;
	name: string;
	domain: string;
	initials: string;
	iconKey: string;
	lastFetched: string;
	status: FeedStatus;
	enabled: boolean;
	autoSave: boolean;
	errorMessage?: string;
	autoSaveCollectionId: string | null;
	pollIntervalOverride: number | null;
	titleOverride: string | null;
	inputUrl: string;
}

export interface FeedStats {
	total: number;
	active: number;
	paused: number;
	error: number;
}

export interface FeedSnapshotEntry {
	id: string;
	autoSave: boolean;
	autoSaveCollectionId: string | null;
}

export interface EditComposerState {
	feedId: string;
	title: string;
	autoSaveCollectionId: string | null;
	pollInterval: string;
	autoSave: boolean;
}

const ICON_KEYS = ['blue', 'green', 'orange', 'rose', 'purple', 'cyan', 'red', 'teal'] as const;

export function deriveInitials(name: string): string {
	const words = name.trim().split(/\s+/).filter(Boolean);
	const first = words[0];
	const second = words[1];
	if (first && second) {
		return (first.charAt(0) + second.charAt(0)).toUpperCase();
	}
	return name.substring(0, 2).toUpperCase();
}

export function hashIconKey(name: string): string {
	let hash = 0;
	for (let i = 0; i < name.length; i++) {
		hash = (hash * 31 + name.charCodeAt(i)) | 0;
	}
	return ICON_KEYS[Math.abs(hash) % ICON_KEYS.length]!;
}

export function formatRelativeTime(iso: string | null | undefined): string {
	if (!iso) return 'Never';
	const diff = Math.max(0, Date.now() - new Date(iso).getTime());
	const minutes = Math.floor(diff / 60_000);
	if (minutes < 1) return 'just now';
	if (minutes < 60) return `${minutes}m ago`;
	const hours = Math.floor(minutes / 60);
	if (hours < 24) return hours === 1 ? '1h ago' : `${hours}h ago`;
	const days = Math.floor(hours / 24);
	if (days < 7) return days === 1 ? '1d ago' : `${days}d ago`;
	const weeks = Math.floor(days / 7);
	return weeks === 1 ? '1w ago' : `${weeks}w ago`;
}

export function isFresh(iso: string | null | undefined): boolean {
	if (!iso) return false;
	const diff = Date.now() - new Date(iso).getTime();
	return diff < 10 * 60_000;
}

export function extractDomain(url: string): string {
	try {
		return new URL(url).hostname.replace(/^www\./, '');
	} catch {
		return url;
	}
}

export function formatSchedule(minutes: number | null | undefined): string {
	if (!minutes) return 'Default';
	if (minutes < 60) return `Every ${minutes}m`;
	const hours = Math.round(minutes / 60);
	if (hours < 24) return hours === 1 ? 'Every 1h' : `Every ${hours}h`;
	const days = Math.round(hours / 24);
	return days === 1 ? 'Daily' : `Every ${days}d`;
}

export function mapStatus(sub: FeedSubscriptionResponse): FeedStatus {
	if (sub.status === 'paused') return 'paused';
	if (sub.source.consecutive_failures > 0 || sub.source.last_error) return 'error';
	return 'active';
}

export function mapSubscription(
	sub: FeedSubscriptionResponse,
	lastPolledIso?: string | null
): Feed {
	const name = sub.title_override ?? sub.source.name;
	const status = mapStatus(sub);
	const polledAt = lastPolledIso !== undefined ? lastPolledIso : sub.source.last_polled_at;
	return {
		id: sub.id,
		name,
		domain: sub.source.domain ?? extractDomain(sub.input_url),
		initials: deriveInitials(name),
		iconKey: hashIconKey(name),
		lastFetched: formatRelativeTime(polledAt),
		status,
		enabled: sub.status !== 'paused',
		autoSave: sub.auto_save,
		errorMessage: sub.source.last_error ?? undefined,
		autoSaveCollectionId: sub.auto_save_collection_id ?? null,
		pollIntervalOverride: sub.poll_interval_override_minutes ?? null,
		titleOverride: sub.title_override ?? null,
		inputUrl: sub.input_url
	};
}

export function calculateFeedStats(feeds: Feed[]): FeedStats {
	return {
		total: feeds.length,
		active: feeds.filter((feed) => feed.status === 'active').length,
		paused: feeds.filter((feed) => feed.status === 'paused').length,
		error: feeds.filter((feed) => feed.status === 'error').length
	};
}

export function filterFeeds(feeds: Feed[], activeFilter: FilterChip, searchQuery: string): Feed[] {
	const query = searchQuery.trim().toLowerCase();
	return feeds.filter((feed) => {
		if (activeFilter !== 'all' && feed.status !== activeFilter) return false;
		if (
			query &&
			!feed.name.toLowerCase().includes(query) &&
			!feed.domain.toLowerCase().includes(query)
		) {
			return false;
		}
		return true;
	});
}

export function snapshotFeeds(feeds: Feed[]): string {
	return JSON.stringify(
		feeds.map((feed) => ({
			id: feed.id,
			autoSave: feed.autoSave,
			autoSaveCollectionId: feed.autoSaveCollectionId
		}))
	);
}

export function parseFeedSnapshot(snapshot: string): FeedSnapshotEntry[] {
	return JSON.parse(snapshot) as FeedSnapshotEntry[];
}

export function changedSnapshotFeeds(feeds: Feed[], saved: FeedSnapshotEntry[]): Feed[] {
	return feeds.filter((feed) => {
		const previous = saved.find((entry) => entry.id === feed.id);
		return (
			previous &&
			(previous.autoSave !== feed.autoSave ||
				previous.autoSaveCollectionId !== feed.autoSaveCollectionId)
		);
	});
}

export function pollIntervalToMinutes(value: string): number | null {
	if (value === 'default') return null;
	return parseInt(value, 10);
}
