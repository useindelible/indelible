import { describe, expect, it } from 'vitest';

import { load } from '../../src/routes/(app)/library/[[type]]/+page';

describe('library type routing', () => {
	it('redirects the retired podcast route to articles and preserves its query', () => {
		let thrown: unknown;
		try {
			load({
				params: { type: 'podcasts' },
				url: new URL('http://localhost/library/podcasts?smart_list=sl_1&view=compact')
			} as never);
		} catch (error) {
			thrown = error;
		}

		expect(thrown).toMatchObject({
			status: 302,
			location: '/library/articles?smart_list=sl_1&view=compact'
		});
	});
});
