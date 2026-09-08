import { afterEach, describe, expect, it, vi } from 'vitest';
import { flushSync } from 'svelte';
import { cleanup, fireEvent, render } from '@testing-library/svelte';

import { activeLayers } from '$lib/shortcuts/registry.svelte';
import TypographyPopover from '$lib/components/reader/TypographyPopover.svelte';

describe('TypographyPopover', () => {
	afterEach(() => {
		cleanup();
		flushSync();
		expect(activeLayers()).toHaveLength(0);
	});

	it('owns the keyboard while open and closes itself on escape', async () => {
		const onClose = vi.fn();
		const anchorEl = document.createElement('button');
		document.body.append(anchorEl);

		render(TypographyPopover, { props: { anchorEl, onClose } });
		flushSync();

		expect(activeLayers()).toEqual([{ scope: 'modal', handlers: {}, exclusive: true }]);

		await fireEvent.keyDown(window, { key: 'Escape' });
		expect(onClose).toHaveBeenCalledOnce();
		anchorEl.remove();
	});
});
