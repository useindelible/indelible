import type { DocumentListEntry } from '$lib/api';

export function entry(id: string, lastRead: string | null = null): DocumentListEntry {
	return {
		max_progress_percent: lastRead ? 60 : null,
		id,
		document_id: id,
		document_type: 'article',
		library_entry_id: `lib_${id}`,
		object: 'library_entry',
		title: id,
		saved_at: '2026-05-18T10:00:00Z',
		created_at: '2026-05-18T10:00:00Z',
		updated_at: '2026-05-18T10:00:00Z',
		source: 'web',
		item_type: 'article',
		triage_state: 'inbox',
		is_favorite: false,
		is_shortlisted: false,
		last_read_at: lastRead
	} as DocumentListEntry;
}
