import { afterEach, describe, expect, it, vi } from 'vitest';
import { flushSync, tick } from 'svelte';
import { cleanup, fireEvent, render, screen } from '@testing-library/svelte';

import { activeLayers } from '$lib/shortcuts/registry.svelte';
import ShortcutHelpOverlay from '$lib/components/shortcuts/ShortcutHelpOverlay.svelte';
import ShortcutHost from '$lib/components/shortcuts/ShortcutHost.svelte';

describe('ShortcutHelpOverlay', () => {
	afterEach(() => {
		cleanup();
		flushSync();
		expect(activeLayers()).toHaveLength(0);
	});

	it('lists the documented bindings and owns the keyboard while open', async () => {
		const onClose = vi.fn();
		render(ShortcutHelpOverlay, { props: { onClose } });
		flushSync();

		expect(activeLayers()).toEqual([
			{ scope: 'modal', handlers: { dismiss: onClose }, exclusive: true }
		]);
		const dialog = screen.getByRole('dialog', { name: 'Keyboard shortcuts' });
		expect(dialog.textContent).toContain('Show keyboard shortcuts');
		expect(dialog.textContent).toContain('Toggle dark mode');
		expect(dialog.textContent).toContain('Archive selected');

		await fireEvent.keyDown(dialog, { key: 'Escape' });
		expect(onClose).toHaveBeenCalledOnce();
	});

	it('closes on the close button and on a backdrop click, not on a dialog click', async () => {
		const onClose = vi.fn();
		render(ShortcutHelpOverlay, { props: { onClose } });

		await fireEvent.click(screen.getByRole('dialog'));
		expect(onClose).not.toHaveBeenCalled();

		await fireEvent.click(screen.getByRole('button', { name: 'Close' }));
		expect(onClose).toHaveBeenCalledOnce();

		const backdrop = screen.getByRole('dialog').parentElement!;
		await fireEvent.click(backdrop);
		expect(onClose).toHaveBeenCalledTimes(2);
	});

	it('takes focus on open, keeps tab inside, and gives focus back on close', async () => {
		const opener = document.createElement('button');
		document.body.append(opener);
		opener.focus();

		const { unmount } = render(ShortcutHelpOverlay, { props: { onClose: vi.fn() } });
		await tick();
		const dialog = screen.getByRole('dialog');
		const close = screen.getByRole('button', { name: 'Close' });
		expect(document.activeElement).toBe(dialog);

		await fireEvent.keyDown(dialog, { key: 'Tab' });
		expect(document.activeElement).toBe(close);

		await fireEvent.keyDown(close, { key: 'Tab' });
		expect(document.activeElement).toBe(close);

		await fireEvent.keyDown(close, { key: 'Tab', shiftKey: true });
		expect(document.activeElement).toBe(close);

		unmount();
		flushSync();
		expect(document.activeElement).toBe(opener);
		opener.remove();
	});

	it('still closes on escape once focus has left the dialog', async () => {
		const onClose = vi.fn();
		render(ShortcutHost);
		render(ShortcutHelpOverlay, { props: { onClose } });
		flushSync();
		const outside = document.createElement('button');
		document.body.append(outside);
		outside.focus();

		await fireEvent.keyDown(outside, { key: 'Escape' });
		expect(onClose).toHaveBeenCalledOnce();

		await fireEvent.keyDown(outside, { key: 'n' });
		expect(onClose).toHaveBeenCalledOnce();
		outside.remove();
	});
});
