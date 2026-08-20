import type { DocumentListEntry } from '$lib/api';
import type { SearchResultResponse } from '$lib/api/generated/types.gen';

export function resultKey(r: SearchResultResponse): string {
	return r.document_id ?? r.delivery_id ?? r.source_entry_id ?? '';
}

export function getDomain(url: string): string {
	try {
		return new URL(url).hostname.replace('www.', '');
	} catch {
		return '';
	}
}

// A search hit carries only what the row renders. The detail panel gets this
// stand-in until the full library entry loads; feed previews keep it for good.
export function toPlaceholderEntry(result: SearchResultResponse): DocumentListEntry {
	const id = resultKey(result);
	return {
		id,
		document_id: id,
		document_type: result.content_type,
		item_type: result.content_type,
		library_entry_id: null,
		object: result.result_kind,
		title: result.title,
		url: result.url ?? null,
		canonical_url: result.url ?? null,
		domain: result.url ? getDomain(result.url) || null : null,
		excerpt: result.snippet.replace(/<\/?mark>/g, ''),
		saved_at: result.saved_at,
		updated_at: result.updated_at,
		created_at: result.saved_at,
		author: null,
		published_at: null,
		word_count: null,
		reading_time_minutes: null,
		language: null,
		summary: null,
		progress_percent: null,
		last_read_at: null,
		sender: result.sender ?? null,
		triage_state: 'later',
		is_favorite: false,
		is_shortlisted: false,
		source: 'document'
	};
}
