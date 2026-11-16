import { describe, expect, it } from 'vitest';
import { noteConflictOf } from './note-conflict';

const conflict = {
	type: 'https://indelible.app/problems/note-conflict',
	title: 'Conflict',
	status: 409,
	detail: 'the note changed since base_updated_at',
	code: 'note_conflict',
	note: {
		id: 'inote_1',
		body: 'theirs',
		created_at: '2026-11-12T22:20:24Z',
		updated_at: '2026-11-12T22:22:32Z'
	}
};

describe('noteConflictOf', () => {
	it('returns the conflict answer of a note write', () => {
		expect(noteConflictOf(conflict)).toEqual(conflict);
	});

	it('ignores another problem', () => {
		expect(noteConflictOf({ ...conflict, code: 'validation_error' })).toBeNull();
	});

	it('ignores a problem without the stored note', () => {
		expect(noteConflictOf({ ...conflict, note: undefined })).toBeNull();
	});

	it('ignores nothing at all', () => {
		expect(noteConflictOf(undefined)).toBeNull();
	});
});
