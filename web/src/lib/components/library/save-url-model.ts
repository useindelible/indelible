export type SaveUrlValidation = '' | 'empty' | 'invalid';

export type DuplicateUrlInfo = {
	id: string;
	title: string;
	domain: string | null;
	savedDate: string | null;
};

export type CollectionSummary = {
	id: string;
	name: string;
};

export function validateSaveUrl(value: string): SaveUrlValidation {
	if (!value.trim()) return 'empty';
	try {
		const parsed = new URL(value.trim());
		if (parsed.protocol !== 'http:' && parsed.protocol !== 'https:') return 'invalid';
	} catch {
		return 'invalid';
	}
	return '';
}

export function messageForUrlValidation(error: SaveUrlValidation): string {
	if (error === 'empty') return get(t)('library_save_url_required');
	if (error === 'invalid') return get(t)('library_save_url_invalid');
	return '';
}

export function normalizeSaveUrlTag(raw: string): string {
	return raw.trim().toLowerCase().replace(/\s+/g, '-');
}

export function addSaveUrlTag(tags: string[], raw: string): string[] {
	const trimmed = normalizeSaveUrlTag(raw);
	if (!trimmed || tags.includes(trimmed)) return tags;
	return [...tags, trimmed];
}

export function removeSaveUrlTag(tags: string[], tag: string): string[] {
	return tags.filter((candidate) => candidate !== tag);
}

export function duplicateFromConflictError(error: unknown): DuplicateUrlInfo | null {
	const body = error as Record<string, unknown> | null | undefined;
	if (!body || typeof body['id'] !== 'string') return null;
	return {
		id: body['id'],
		title:
			typeof body['title'] === 'string' ? body['title'] : get(t)('library_duplicate_already_saved'),
		domain: typeof body['domain'] === 'string' ? body['domain'] : null,
		savedDate: typeof body['created_at'] === 'string' ? body['created_at'] : null
	};
}

export function messageForSaveUrlProblem(error: unknown): string {
	const problem = error as Record<string, unknown> | null | undefined;
	return (
		(typeof problem?.['detail'] === 'string' ? problem['detail'] : undefined) ??
		(typeof problem?.['message'] === 'string' ? problem['message'] : undefined) ??
		get(t)('library_error_save')
	);
}

export function formatDuplicateSavedDate(iso: string | null): string {
	if (!iso) return '';
	try {
		const parsed = new Date(iso);
		if (!Number.isFinite(parsed.getTime())) return '';
		return get(date)(parsed, {
			day: 'numeric',
			month: 'short',
			year: 'numeric'
		});
	} catch {
		return '';
	}
}

export function getSelectedCollectionName(
	collectionId: string | null,
	collections: CollectionSummary[]
): string {
	if (!collectionId) return get(t)('library_triage_inbox');
	return (
		collections.find((collection) => collection.id === collectionId)?.name ??
		get(t)('library_filter_field_collection')
	);
}
import { date, t } from '$lib/i18n';
import { get } from 'svelte/store';
