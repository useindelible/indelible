import { describe, expect, it } from 'vitest';
import type { MilaSourceRef } from '$lib/api/generated/types.gen';
import { renderMilaMessageMarkdown } from '$lib/utils/mila-citations';

describe('renderMilaMessageMarkdown', () => {
	it('replaces source labels with inline reader chips', () => {
		const refs: MilaSourceRef[] = [
			{
				source_label: 'S1',
				document_id: 'doc_1',
				item_title:
					'I built a database in France because the Cloud Act makes EU data sovereignty impossible'
			}
		];

		const html = renderMilaMessageMarkdown(
			'FISA 702 applies broadly [S1].',
			refs,
			(documentId) => `/reader/${documentId}`
		);

		expect(html).toContain('href="/reader/doc_1"');
		expect(html).toContain('class="chat-inline-source"');
		expect(html).toContain('I built a database in France');
		expect(html).not.toContain('[S1]');
	});

	it('leaves unknown source labels as text', () => {
		const html = renderMilaMessageMarkdown(
			'No ref for this [S9].',
			[],
			(documentId) => `/reader/${documentId}`
		);

		expect(html).toContain('[S9]');
	});

	it('collapses repeated adjacent labels before rendering chips', () => {
		const refs: MilaSourceRef[] = [
			{
				source_label: 'S1',
				document_id: 'doc_1',
				item_title: 'Cloud Act Article'
			}
		];

		const html = renderMilaMessageMarkdown(
			'Supported [S1][S1][S1].',
			refs,
			(documentId) => `/reader/${documentId}`
		);

		expect(html.match(/chat-inline-source/g)?.length).toBe(1);
	});
});
