import { getLibrary, triageOptionsForMode, type TriageTab } from './library.svelte';

const lib = getLibrary();

let keyboardDriving = $state(false);
let pointer: { x: number; y: number } | null = null;

/** The rows the list renders; keyboard navigation must walk this, not every fetched item. */
const displayItems = $derived(
	lib.groupBy === 'read_status'
		? lib.readStatusTab === 'unseen'
			? lib.items.filter((item) => !item.last_read_at)
			: lib.items.filter((item) => !!item.last_read_at)
		: lib.items
);

export function nextSelectedId(
	items: readonly { id: string }[],
	selectedId: string | null,
	offset: number
): string | null {
	const index = items.findIndex((item) => item.id === selectedId);
	const next = Math.min(Math.max(index + offset, 0), items.length - 1);
	return items[next]?.id ?? null;
}

/** The row that should take over when `id` leaves the list: the one after it, else the one before. */
export function neighbourId(items: readonly { id: string }[], id: string): string | null {
	const index = items.findIndex((item) => item.id === id);
	if (index === -1) return null;
	return items[index + 1]?.id ?? items[index - 1]?.id ?? null;
}

export function getLibrarySelection() {
	return {
		get displayItems() {
			return displayItems;
		},
		/** The selected row only while it is visible; a row hidden by a view switch is inert. */
		get selectedItem() {
			return displayItems.find((item) => item.id === lib.selectedId) ?? null;
		},
		get keyboardDriving() {
			return keyboardDriving;
		},
		moveSelection(offset: number): void {
			keyboardDriving = true;
			lib.setSelectedId(nextSelectedId(displayItems, lib.selectedId, offset));
		},
		// Manual triage mode has no Later state, so its key does nothing there.
		triageSelected(state: TriageTab): void {
			const item = displayItems.find((row) => row.id === lib.selectedId);
			if (!item || item.triage_state === state) return;
			if (!triageOptionsForMode(lib.triageMode).some((option) => option.value === state)) return;
			const next = lib.triageScopesList() ? neighbourId(displayItems, item.id) : item.id;
			void lib.triageAction(item.id, state);
			lib.setSelectedId(next);
		},
		/**
		 * Whether a pointer event over a row should select it. Scrolling the list
		 * with j/k fires mouseenter on whatever slides under a still pointer, at the
		 * same coordinates as the last real event; only a coordinate change counts.
		 */
		hoverSelects(event: { clientX: number; clientY: number }): boolean {
			const previous = pointer;
			pointer = { x: event.clientX, y: event.clientY };
			if (!keyboardDriving) return true;
			if (previous === null) return false;
			if (previous.x === event.clientX && previous.y === event.clientY) return false;
			keyboardDriving = false;
			return true;
		}
	};
}
