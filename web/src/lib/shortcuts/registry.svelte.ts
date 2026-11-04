import { KEYMAP, type ShortcutId } from './keymap';
import { resolveShortcut, type ScopeLayer } from './dispatch';
import { isActivationTarget, isTypingTarget } from './typing-target';

/**
 * Plain array rather than `$state`: nothing renders from the stack, and the
 * dispatcher reads it at event time. Keeping it untracked avoids an effect that
 * would re-run itself on every push.
 */
const layers: ScopeLayer[] = [];

function warnOnScopeMismatch(layer: ScopeLayer): void {
	const ids = Object.keys(layer.handlers) as ShortcutId[];
	for (const id of ids) {
		const scopes = KEYMAP.filter((def) => def.id === id).map((def) => def.scope);
		if (scopes.length === 0) {
			console.warn(`[shortcuts] handler "${id}" has no keymap entry`);
		} else if (!scopes.includes(layer.scope)) {
			console.warn(
				`[shortcuts] handler "${id}" registered on "${layer.scope}" but the keymap scopes it to ${scopes.join(', ')}`
			);
		}
	}
}

/**
 * Publishes a scope's handlers while the calling component is mounted. Removal
 * is by object identity because SvelteKit mounts an incoming route layout before
 * unmounting the outgoing one: filtering by scope name would drop the new layer.
 */
export function registerShortcuts(build: () => ScopeLayer | null): void {
	$effect(() => {
		const layer = build();
		if (!layer) return;

		if (import.meta.env.DEV) warnOnScopeMismatch(layer);

		layers.push(layer);

		return () => {
			const index = layers.indexOf(layer);
			if (index !== -1) layers.splice(index, 1);
		};
	});
}

export function activeLayers(): readonly ScopeLayer[] {
	return layers;
}

/**
 * Modal roles only. `menu` and `listbox` also dress persistent UI — the library
 * list is a listbox whose rows are focusable — so matching them would silence
 * every shortcut the moment a row took focus. Non-modal overlays declare
 * suppression explicitly instead.
 */
const OVERLAY_SELECTOR = '[role="dialog"], [role="alertdialog"]';

const ACTIVATION_KEYS = new Set(['Enter', ' ']);

/**
 * Keeps app shortcuts from acting on the page behind an open dialog whose own
 * handlers already own the keyboard, including dialogs that never register a
 * suppression layer.
 */
function isInsideOverlay(target: EventTarget | null): boolean {
	return target instanceof HTMLElement && target.closest(OVERLAY_SELECTOR) !== null;
}

export function handleGlobalKeydown(event: KeyboardEvent): void {
	if (isInsideOverlay(event.target)) return;
	if (ACTIVATION_KEYS.has(event.key) && isActivationTarget(event.target)) return;

	const resolved = resolveShortcut({
		event,
		layers,
		typing: isTypingTarget(event.target)
	});

	if (!resolved) return;

	event.preventDefault();
	resolved.handler(event);
}
