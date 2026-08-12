export type LibraryFilterFieldDef = {
	key: string;
	label: string;
	section: 'content' | 'attributes' | 'dates';
	ops: { value: string; label: string }[];
	valueType: 'select' | 'multi-select' | 'text' | 'number' | 'date' | 'boolean';
	options?: { value: string; label: string }[];
	booleanLabels?: { true: string; false: string };
	scope?: 'email';
};

const LIBRARY_FILTER_FIELDS: LibraryFilterFieldDef[] = [
	{
		key: 'tag',
		label: 'Tag',
		section: 'content',
		ops: [
			{ value: 'contains', label: 'contains' },
			{ value: 'eq', label: 'is exactly' },
			{ value: 'neq', label: 'is not' }
		],
		valueType: 'text'
	},
	{
		key: 'item_type',
		label: 'Content type',
		section: 'content',
		ops: [
			{ value: 'eq', label: 'is' },
			{ value: 'neq', label: 'is not' },
			{ value: 'in', label: 'is any of' }
		],
		valueType: 'select',
		options: [
			{ value: 'article', label: 'Article' },
			{ value: 'book', label: 'Book' },
			{ value: 'email', label: 'Email' },
			{ value: 'pdf', label: 'PDF' },
			{ value: 'tweet', label: 'Tweet' },
			{ value: 'video', label: 'Video' }
		]
	},
	{
		key: 'domain',
		label: 'Domain',
		section: 'content',
		ops: [
			{ value: 'eq', label: 'is' },
			{ value: 'neq', label: 'is not' },
			{ value: 'in', label: 'is any of' }
		],
		valueType: 'text'
	},
	{
		key: 'sender',
		label: 'Sender',
		section: 'content',
		scope: 'email',
		ops: [
			{ value: 'contains', label: 'contains' },
			{ value: 'eq', label: 'is exactly' },
			{ value: 'neq', label: 'is not' },
			{ value: 'in', label: 'is any of' }
		],
		valueType: 'text'
	},
	{
		key: 'sender_domain',
		label: 'Sender domain',
		section: 'content',
		scope: 'email',
		ops: [
			{ value: 'contains', label: 'contains' },
			{ value: 'eq', label: 'is exactly' },
			{ value: 'neq', label: 'is not' },
			{ value: 'in', label: 'is any of' }
		],
		valueType: 'text'
	},
	{
		key: 'list_id',
		label: 'List-ID',
		section: 'content',
		scope: 'email',
		ops: [
			{ value: 'contains', label: 'contains' },
			{ value: 'eq', label: 'is exactly' },
			{ value: 'neq', label: 'is not' },
			{ value: 'in', label: 'is any of' }
		],
		valueType: 'text'
	},
	{
		key: 'subject',
		label: 'Subject',
		section: 'content',
		scope: 'email',
		ops: [
			{ value: 'contains', label: 'contains' },
			{ value: 'eq', label: 'is exactly' },
			{ value: 'neq', label: 'is not' },
			{ value: 'in', label: 'is any of' }
		],
		valueType: 'text'
	},
	{
		key: 'collection',
		label: 'Collection',
		section: 'content',
		ops: [
			{ value: 'eq', label: 'is' },
			{ value: 'contains', label: 'contains' }
		],
		valueType: 'text'
	},
	{
		key: 'is_favorite',
		label: 'Favorited',
		section: 'attributes',
		ops: [],
		valueType: 'boolean',
		booleanLabels: { true: 'is favorited', false: 'is not favorited' }
	},
	{
		key: 'has_unsubscribe',
		label: 'Has unsubscribe',
		section: 'attributes',
		scope: 'email',
		ops: [],
		valueType: 'boolean',
		booleanLabels: { true: 'has unsubscribe', false: 'no unsubscribe' }
	},
	{
		key: 'sender_blocked',
		label: 'Sender blocked',
		section: 'attributes',
		scope: 'email',
		ops: [],
		valueType: 'boolean',
		booleanLabels: { true: 'sender blocked', false: 'sender not blocked' }
	},
	{
		key: 'triage_state',
		label: 'Read Status',
		section: 'attributes',
		ops: [
			{ value: 'eq', label: 'is' },
			{ value: 'neq', label: 'is not' },
			{ value: 'in', label: 'is any of' }
		],
		valueType: 'select',
		options: [
			{ value: 'inbox', label: 'Inbox' },
			{ value: 'later', label: 'Later' },
			{ value: 'archive', label: 'Archive' }
		]
	},
	{
		key: 'saved_at',
		label: 'Saved date',
		section: 'dates',
		ops: [
			{ value: 'gt', label: 'after' },
			{ value: 'lt', label: 'before' },
			{ value: 'eq', label: 'on' },
			{ value: 'gte', label: 'on or after' },
			{ value: 'lte', label: 'on or before' }
		],
		valueType: 'date'
	},
	{
		key: 'published_at',
		label: 'Published date',
		section: 'dates',
		ops: [
			{ value: 'gt', label: 'after' },
			{ value: 'lt', label: 'before' },
			{ value: 'eq', label: 'on' },
			{ value: 'gte', label: 'on or after' },
			{ value: 'lte', label: 'on or before' }
		],
		valueType: 'date'
	}
];

export function getLibraryFilterFieldDef(key: string): LibraryFilterFieldDef {
	return LIBRARY_FILTER_FIELDS.find((field) => field.key === key) ?? LIBRARY_FILTER_FIELDS[0]!;
}

export function getVisibleLibraryFilterFields(activeType?: string | null): LibraryFilterFieldDef[] {
	const isEmailSection = activeType === 'emails';
	return LIBRARY_FILTER_FIELDS.filter((field) => !field.scope || isEmailSection);
}
