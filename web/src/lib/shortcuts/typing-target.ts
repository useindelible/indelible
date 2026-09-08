const TYPING_TAGS = new Set(['INPUT', 'TEXTAREA', 'SELECT']);
const TYPING_ROLES = new Set(['textbox', 'combobox', 'searchbox', 'spinbutton']);

/**
 * True when keystrokes belong to the target rather than to an app shortcut.
 * Covers native fields, ARIA widgets that accept text, and any descendant of a
 * contenteditable region.
 */
export function isTypingTarget(target: EventTarget | null): boolean {
	if (!(target instanceof HTMLElement)) return false;
	if (TYPING_TAGS.has(target.tagName)) return true;
	if (target.isContentEditable) return true;

	const role = target.getAttribute('role');
	if (role !== null && TYPING_ROLES.has(role)) return true;

	return target.closest('[contenteditable="true"]') !== null;
}

const ACTIVATION_SELECTOR = [
	'button',
	'a[href]',
	'summary',
	'[role="button"]',
	'[role="link"]',
	'[role="option"]',
	'[role="menuitem"]',
	'[role="tab"]',
	'[role="checkbox"]',
	'[role="switch"]',
	'[role="radio"]'
].join(', ');

/** True when Enter or Space on the target activates it, so a shortcut must not also fire. */
export function isActivationTarget(target: EventTarget | null): boolean {
	return target instanceof HTMLElement && target.closest(ACTIVATION_SELECTOR) !== null;
}
