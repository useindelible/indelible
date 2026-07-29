export const LIBRARY_DOMAIN_EVENT_TYPES = [
	'library_entry.saved',
	'library_entry.triaged',
	'library_entry.archived',
	'library_entry.favorited',
	'library_entry.trashed',
	'library_entry.restored',
	'library_entry.permanently_deleted',
	'library_entry.tagged',
	'library_entry.untagged',
	'document.highlighted',
	'highlight.updated',
	'highlight.deleted',
	'highlight.noted'
] as const;

export const READER_HIGHLIGHT_DOMAIN_EVENT_TYPES = [
	'document.highlighted',
	'highlight.updated',
	'highlight.deleted',
	'highlight.noted'
] as const;

export const READER_AI_DOMAIN_EVENT_TYPES = ['ai.output.completed', 'ai.output.failed'] as const;

export const WEB_DOMAIN_EVENT_TYPES = [
	...new Set([
		...LIBRARY_DOMAIN_EVENT_TYPES,
		...READER_HIGHLIGHT_DOMAIN_EVENT_TYPES,
		...READER_AI_DOMAIN_EVENT_TYPES
	])
] as const;
