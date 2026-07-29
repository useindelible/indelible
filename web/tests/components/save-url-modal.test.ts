import { describe, expect, it, vi } from 'vitest';
import { fireEvent, render, screen } from '@testing-library/svelte';

import SaveUrlInputZone from '$lib/components/library/SaveUrlInputZone.svelte';

describe('SaveUrl input zone', () => {
	it('submits on Enter and closes on Escape', async () => {
		const onSave = vi.fn();
		const onClose = vi.fn();

		render(SaveUrlInputZone, {
			props: {
				url: 'https://example.com',
				hasDuplicate: false,
				onSave,
				onClose
			}
		});

		const input = screen.getByLabelText('URL');
		await fireEvent.keyDown(input, { key: 'Enter' });
		await fireEvent.keyDown(input, { key: 'Escape' });

		expect(onSave).toHaveBeenCalledOnce();
		expect(onClose).toHaveBeenCalledOnce();
	});
});
