import { KEYMAP, matchChord, type ShortcutId, type ShortcutScope } from './keymap';

export type ShortcutHandler = (event: KeyboardEvent) => void;

export type ShortcutHandlers = Partial<Record<ShortcutId, ShortcutHandler>>;

export type ScopeLayer = {
	scope: ShortcutScope;
	handlers: ShortcutHandlers;
	/** Swallows every shortcut below it, so an open overlay owns the keyboard. */
	exclusive?: boolean;
};

export type ResolvableEvent = Pick<
	KeyboardEvent,
	'key' | 'metaKey' | 'ctrlKey' | 'altKey' | 'shiftKey'
> & {
	isComposing?: boolean;
	keyCode?: number;
};

export type ResolvedShortcut = {
	id: ShortcutId;
	handler: ShortcutHandler;
};

const IME_KEY_CODE = 229;

/**
 * Picks the single handler that owns a keystroke. Layers are ordered bottom-up,
 * so the last one pushed wins. Taking the stack as an argument keeps resolution
 * free of DOM and global state, which is what lets the keymap tests assert that
 * exactly one handler can ever claim a chord.
 */
export function resolveShortcut(input: {
	event: ResolvableEvent;
	layers: readonly ScopeLayer[];
	typing: boolean;
}): ResolvedShortcut | null {
	const { event, layers, typing } = input;

	if (event.isComposing || event.keyCode === IME_KEY_CODE) return null;
	if (typing) return null;

	const lastExclusive = layers.reduce(
		(found, layer, index) => (layer.exclusive ? index : found),
		-1
	);
	const candidates = lastExclusive >= 0 ? [layers[lastExclusive]!] : [...layers].reverse();

	for (const layer of candidates) {
		for (const def of KEYMAP) {
			if (def.scope !== layer.scope) continue;
			if (!matchChord(event, def.chord)) continue;

			const handler = layer.handlers[def.id];
			if (handler) return { id: def.id, handler };
		}
	}

	return null;
}
