export interface ActiveEntityFilter {
	name: string;
	entityType: string;
}

export const FILTER_HINTS = [
	'tag:',
	'collection:',
	'type:',
	'author:',
	'sender:',
	'sender_domain:',
	'list:',
	'subject:',
	'before:',
	'after:',
	'is:',
	'has:',
	'url:',
	'pinned:'
];

export function parseEntityPrefix(
	q: string
): { entityName: string; entityType: string; remainder: string } | null {
	const match = q.match(/^entity:"([^"]+)"(?:\s+(.*))?$/);
	if (match) {
		return { entityName: match[1]!, entityType: '', remainder: match[2]?.trim() ?? '' };
	}
	return null;
}

export function buildSearchQuery(
	userInput: string,
	activeEntityFilter: ActiveEntityFilter | null
): string {
	const trimmed = userInput.trim();
	if (activeEntityFilter) {
		const prefix = `entity:"${activeEntityFilter.name}"`;
		return trimmed ? `${prefix} ${trimmed}` : prefix;
	}
	return trimmed;
}
