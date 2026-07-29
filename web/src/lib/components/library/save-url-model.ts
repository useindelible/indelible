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
	if (error === 'empty') return 'Please paste a URL.';
	if (error === 'invalid') return 'That does not look like a valid URL.';
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
		title: typeof body['title'] === 'string' ? body['title'] : 'Already saved',
		domain: typeof body['domain'] === 'string' ? body['domain'] : null,
		savedDate: typeof body['created_at'] === 'string' ? body['created_at'] : null
	};
}

export function messageForSaveUrlProblem(error: unknown): string {
	const problem = error as Record<string, unknown> | null | undefined;
	return (
		(typeof problem?.['detail'] === 'string' ? problem['detail'] : undefined) ??
		(typeof problem?.['message'] === 'string' ? problem['message'] : undefined) ??
		'Failed to save. Please try again.'
	);
}

export function formatDuplicateSavedDate(iso: string | null): string {
	if (!iso) return '';
	try {
		const date = new Date(iso);
		if (!Number.isFinite(date.getTime())) return '';
		return new Intl.DateTimeFormat('en-US', {
			day: 'numeric',
			month: 'short',
			year: 'numeric'
		}).format(date);
	} catch {
		return '';
	}
}

export function getSelectedCollectionName(
	collectionId: string | null,
	collections: CollectionSummary[]
): string {
	if (!collectionId) return 'Inbox';
	return collections.find((collection) => collection.id === collectionId)?.name ?? 'Collection';
}
