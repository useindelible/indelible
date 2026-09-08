import { registerShortcuts } from './registry.svelte';

/**
 * Silences app shortcuts while a dialog or popover is open. The layer carries no
 * handlers and no keymap rows target its scope, so every chord resolves to
 * nothing. Dialogs keep their own Escape handling, which never reaches the
 * dispatcher while the backdrop holds focus.
 */
export function suppressShortcutsWhileOpen(isOpen: () => boolean): void {
	registerShortcuts(() => (isOpen() ? { scope: 'modal', handlers: {}, exclusive: true } : null));
}
