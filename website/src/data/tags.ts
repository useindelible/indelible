/**
 * The tag manager's list.
 *
 * The footer counts TAGS, not documents. `nested` means the row sits under
 * the one above it; `hasChildren` puts a disclosure on the row.
 */
export interface Tag {
	name: string;
	colour: string;
	count: string;
	nested?: boolean;
	hasChildren?: boolean;
}

export const TAGS: readonly Tag[] = [
	{ name: 'artificial intelligence', colour: '#AF52DE', count: '3 items', hasChildren: true },
	{ name: 'machine learning', colour: '#FF3B30', count: '1 item', nested: true },
	{ name: 'neural networks', colour: '#8E8E93', count: '1 item', nested: true },
	{ name: 'books', colour: '#BF5AF2', count: '2 items' },
	{ name: 'creativity', colour: '#34C759', count: '1 item' },
	{ name: 'knowledge management', colour: '#34C759', count: '3 items' },
	{ name: 'long reads', colour: '#FFD60A', count: '2 items' },
	{ name: 'research', colour: '#0A84FF', count: '0 items' },
	{ name: 'systems', colour: '#30D158', count: '1 item' },
	{ name: 'writing', colour: '#0A84FF', count: '1 item' },
];
