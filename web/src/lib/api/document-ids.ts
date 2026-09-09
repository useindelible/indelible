// Callers address a document by either id; the two maps keep the pairing learned from responses.
export const libraryEntryByDocument = new Map<string, string>();
export const documentByLibraryEntry = new Map<string, string>();

export function resolveDocumentId(id: string): string {
	if (id.startsWith('doc_')) return id;
	return documentByLibraryEntry.get(id) ?? id;
}
