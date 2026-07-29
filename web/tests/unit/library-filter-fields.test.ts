import { describe, expect, it } from 'vitest';

import { getVisibleLibraryFilterFields } from '../../src/lib/utils/library-filter-fields';

const fieldKeys = (activeType?: string) =>
	getVisibleLibraryFilterFields(activeType).map((field) => field.key);

describe('getVisibleLibraryFilterFields', () => {
	it('hides email-only fields outside the email library section', () => {
		expect(fieldKeys()).not.toContain('sender');
		expect(fieldKeys('articles')).not.toContain('sender');
		expect(fieldKeys('articles')).not.toContain('sender_domain');
		expect(fieldKeys('articles')).not.toContain('list_id');
		expect(fieldKeys('articles')).not.toContain('subject');
		expect(fieldKeys('articles')).not.toContain('has_unsubscribe');
		expect(fieldKeys('articles')).not.toContain('sender_blocked');
	});

	it('shows email-only fields inside the email library section', () => {
		expect(fieldKeys('emails')).toEqual(
			expect.arrayContaining([
				'sender',
				'sender_domain',
				'list_id',
				'subject',
				'has_unsubscribe',
				'sender_blocked'
			])
		);
	});

	it('keeps common fields visible in every library section', () => {
		expect(fieldKeys('articles')).toEqual(
			expect.arrayContaining(['tag', 'item_type', 'domain', 'collection'])
		);
		expect(fieldKeys('emails')).toEqual(
			expect.arrayContaining(['tag', 'item_type', 'domain', 'collection'])
		);
	});
});
