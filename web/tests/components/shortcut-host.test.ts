import { afterEach, describe, expect, it, vi } from 'vitest';
import { flushSync } from 'svelte';
import { cleanup, fireEvent, render } from '@testing-library/svelte';

import type { ScopeLayer } from '$lib/shortcuts/dispatch';
import { activeLayers } from '$lib/shortcuts/registry.svelte';
import ShortcutHarness from './ShortcutHarness.svelte';

function mount(extraLayers: ScopeLayer[] = []) {
	const addUrl = vi.fn();
	const archive = vi.fn();
	const layers: ScopeLayer[] = [
		{ scope: 'global', handlers: { add_url: addUrl } },
		{ scope: 'library', handlers: { triage_archive: archive } },
		...extraLayers
	];

	render(ShortcutHarness, { props: { layers } });
	return { addUrl, archive };
}

describe('ShortcutHost', () => {
	// Unmounting must retract the layers, or a stale scope would keep claiming keys.
	afterEach(() => {
		cleanup();
		flushSync();
		expect(activeLayers()).toHaveLength(0);
	});

	it('runs one handler for a chord two scopes could plausibly want', async () => {
		const { addUrl, archive } = mount();

		await fireEvent.keyDown(window, { key: 'a' });

		expect(archive).toHaveBeenCalledOnce();
		expect(addUrl).not.toHaveBeenCalled();
	});

	it('opens the add modal on n without touching the library', async () => {
		const { addUrl, archive } = mount();

		await fireEvent.keyDown(window, { key: 'n' });

		expect(addUrl).toHaveBeenCalledOnce();
		expect(archive).not.toHaveBeenCalled();
	});

	it('lets enter activate a focused button instead of opening the selection', async () => {
		const open = vi.fn();
		mount([{ scope: 'library', handlers: { open_item: open } }]);
		const button = document.createElement('button');
		document.body.append(button);

		await fireEvent.keyDown(button, { key: 'Enter' });
		expect(open).not.toHaveBeenCalled();

		await fireEvent.keyDown(window, { key: 'Enter' });
		expect(open).toHaveBeenCalledOnce();
		button.remove();
	});

	it('shows help on ? and toggles dark mode on d', async () => {
		const help = vi.fn();
		const dark = vi.fn();
		mount([{ scope: 'global', handlers: { show_help: help, toggle_dark: dark } }]);

		await fireEvent.keyDown(window, { key: '?', shiftKey: true });
		await fireEvent.keyDown(window, { key: 'd' });

		expect(help).toHaveBeenCalledOnce();
		expect(dark).toHaveBeenCalledOnce();
	});

	it('leaves cmd+a to the browser', async () => {
		const { addUrl, archive } = mount();

		await fireEvent.keyDown(window, { key: 'a', metaKey: true });

		expect(archive).not.toHaveBeenCalled();
		expect(addUrl).not.toHaveBeenCalled();
	});

	it('stays out of the way while the user types, modifiers included', async () => {
		const { addUrl, archive } = mount();
		const input = document.createElement('input');
		document.body.append(input);
		input.focus();

		await fireEvent.keyDown(input, { key: 'a' });
		await fireEvent.keyDown(input, { key: 'n', metaKey: true });

		expect(archive).not.toHaveBeenCalled();
		expect(addUrl).not.toHaveBeenCalled();

		input.remove();
	});

	it('does not reach the page behind an open dialog', async () => {
		const { addUrl, archive } = mount();
		const dialog = document.createElement('div');
		dialog.setAttribute('role', 'dialog');
		document.body.append(dialog);

		await fireEvent.keyDown(dialog, { key: 'a' });
		await fireEvent.keyDown(dialog, { key: 'n' });

		expect(archive).not.toHaveBeenCalled();
		expect(addUrl).not.toHaveBeenCalled();

		dialog.remove();
	});

	it('still fires while a focusable row inside the library listbox holds focus', async () => {
		const { archive } = mount();
		const list = document.createElement('div');
		list.setAttribute('role', 'listbox');
		const row = document.createElement('div');
		row.setAttribute('role', 'option');
		row.tabIndex = 0;
		list.append(row);
		document.body.append(list);
		row.focus();

		await fireEvent.keyDown(row, { key: 'a' });

		expect(archive).toHaveBeenCalledOnce();

		list.remove();
	});

	it('stays suppressed when an overlay is open but focus sits outside it', async () => {
		const { addUrl, archive } = mount([{ scope: 'modal', handlers: {}, exclusive: true }]);
		const trigger = document.createElement('button');
		document.body.append(trigger);
		trigger.focus();

		await fireEvent.keyDown(trigger, { key: 'a' });
		await fireEvent.keyDown(trigger, { key: 'n' });

		expect(archive).not.toHaveBeenCalled();
		expect(addUrl).not.toHaveBeenCalled();

		trigger.remove();
	});
});
