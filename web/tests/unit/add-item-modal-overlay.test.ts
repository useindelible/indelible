import { afterEach, describe, expect, it } from 'vitest';

import { getModalStore } from '$lib/stores/addItemModal.svelte';

const modal = getModalStore();

describe('modal store overlayOpen', () => {
	afterEach(() => {
		modal.close();
		modal.closePopover();
	});

	it('is closed when nothing is showing', () => {
		expect(modal.overlayOpen).toBe(false);
	});

	it('reports an open modal', () => {
		modal.open('url');
		expect(modal.overlayOpen).toBe(true);
	});

	it('reports an open popover, whose trigger keeps focus outside the menu', () => {
		modal.togglePopover();

		expect(modal.popoverOpen).toBe(true);
		expect(modal.overlayOpen).toBe(true);
	});

	it('closes again once the popover is dismissed', () => {
		modal.togglePopover();
		modal.closePopover();

		expect(modal.overlayOpen).toBe(false);
	});
});
