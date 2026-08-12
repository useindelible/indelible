import { fireEvent, render } from '@testing-library/svelte';
import { describe, expect, it, vi } from 'vitest';

import OpmlImportCard from '../../src/routes/(app)/preferences/feed-management/components/OpmlImportCard.svelte';

describe('OpmlImportCard', () => {
	it('keeps the chooser ready for a replacement file after an upload', async () => {
		const onUpload = vi.fn();
		const { container } = render(OpmlImportCard, {
			props: { uploading: false, result: null, error: null, onUpload }
		});
		const input = container.querySelector('input[type="file"]') as HTMLInputElement;
		const malformed = new File(['<opml>'], 'broken.opml', { type: 'text/xml' });
		const replacement = new File(['<opml/>'], 'replacement.opml', { type: 'text/xml' });

		await fireEvent.change(input, { target: { files: [malformed] } });
		expect(input.value).toBe('');
		await fireEvent.change(input, { target: { files: [replacement] } });

		expect(onUpload).toHaveBeenNthCalledWith(1, malformed);
		expect(onUpload).toHaveBeenNthCalledWith(2, replacement);
		expect(input.value).toBe('');
		expect(container.querySelector('input[type="file"]')).toBe(input);
	});
});
