import type { DocumentNoteConflictResponse } from '$lib/api';

/** The 409 answer of a note write, or null for any other error the client got back. */
export function noteConflictOf(error: unknown): DocumentNoteConflictResponse | null {
	if (!error || typeof error !== 'object') return null;
	const problem = error as Partial<DocumentNoteConflictResponse>;
	const note = problem.note;
	if (problem.code !== 'note_conflict' || !note || typeof note !== 'object') return null;
	if (typeof note.body !== 'string' || typeof note.updated_at !== 'string') return null;
	return problem as DocumentNoteConflictResponse;
}
