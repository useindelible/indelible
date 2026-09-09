import { SvelteMap, SvelteSet } from 'svelte/reactivity';

import * as apiSdk from '$lib/api';
import type { DocumentListEntry, LibraryTriageRequest } from '$lib/api';

type TriageTab = LibraryTriageRequest['state'];

/** The server-backed slice of library state a mutation reads and commits into. */
export type MutationContext = {
	base(): DocumentListEntry[];
	setBase(next: DocumentListEntry[]): void;
	selectedId(): string | null;
	setSelectedId(id: string | null): void;
	triageScopesList(): boolean;
	/** Whether the built-in view the user is looking at now would list this row. */
	viewShows(item: DocumentListEntry): boolean;
	/** Asks the backend to re-evaluate membership when a filter it owns is active. */
	refetchIfBackendOwnsFilter(): void;
	refreshTrashCount(): Promise<void>;
};

type Overlay = { fields: Partial<DocumentListEntry>; removed: boolean };

/**
 * Optimistic mutations never edit the server rows directly. Each in-flight request holds an
 * overlay that hides or patches its row in the visible list; success commits the overlay into
 * the rows, failure drops it. A fetch can replace the rows at any time and the overlays still
 * apply, so a request settling late can neither resurrect, duplicate, nor stale-restore a row.
 */
export function createLibraryMutations(ctx: MutationContext) {
	const overlays = new SvelteMap<string, Overlay[]>();

	function overlay(rows: DocumentListEntry[]): DocumentListEntry[] {
		if (overlays.size === 0) return rows;
		const out: DocumentListEntry[] = [];
		for (const row of rows) {
			const pending = overlays.get(row.id);
			if (!pending) {
				out.push(row);
				continue;
			}
			if (pending.some((op) => op.removed)) continue;
			out.push(pending.reduce((acc, op) => ({ ...acc, ...op.fields }), row));
		}
		return out;
	}

	function begin(itemId: string, op: Overlay): Overlay {
		overlays.set(itemId, [...(overlays.get(itemId) ?? []), op]);
		if (op.removed && ctx.selectedId() === itemId) ctx.setSelectedId(null);
		return op;
	}

	// A committed value outranks anything an older request on the same row is still waiting
	// on, so its keys leave those overlays; a dropped overlay leaves the older ones untouched.
	function end(itemId: string, op: Overlay, committed: Partial<DocumentListEntry> | null): void {
		const pending = overlays.get(itemId) ?? [];
		const index = pending.indexOf(op);
		const rest = pending.filter((entry) => entry !== op);
		if (committed) {
			for (const older of rest.slice(0, index)) {
				older.fields = Object.fromEntries(
					Object.entries(older.fields).filter(([key]) => !(key in committed))
				);
			}
		}
		if (rest.length === 0) overlays.delete(itemId);
		else overlays.set(itemId, rest);
	}

	function commitFields(itemId: string, fields: Partial<DocumentListEntry>): void {
		ctx.setBase(ctx.base().map((row) => (row.id === itemId ? { ...row, ...fields } : row)));
	}

	function commitRemoval(itemId: string): void {
		ctx.setBase(ctx.base().filter((row) => row.id !== itemId));
	}

	function visible(itemId: string): DocumentListEntry | undefined {
		return overlay(ctx.base()).find((row) => row.id === itemId);
	}

	async function triageAction(itemId: string, state: TriageTab): Promise<void> {
		const before = visible(itemId);
		if (!before) return;
		const committed = { ...before, triage_state: state };
		const op = begin(itemId, { fields: { triage_state: state }, removed: ctx.triageScopesList() });

		try {
			await apiSdk.triageLibraryEntry({ path: { document_id: itemId }, body: { state } });
			if (op.removed && !ctx.viewShows(committed)) commitRemoval(itemId);
			else commitFields(itemId, op.fields);
			end(itemId, op, op.fields);
			ctx.refetchIfBackendOwnsFilter();
		} catch {
			end(itemId, op, null);
		}
	}

	async function deleteAction(itemId: string): Promise<void> {
		if (!visible(itemId)) return;
		const op = begin(itemId, { fields: {}, removed: true });

		try {
			await apiSdk.deleteLibraryEntry({ path: { document_id: itemId } });
			commitRemoval(itemId);
			end(itemId, op, op.fields);
			await ctx.refreshTrashCount();
		} catch {
			end(itemId, op, null);
		}
	}

	const unreadInFlight = new SvelteSet<string>();

	/**
	 * Clears the row's read state in place; views that hide unread rows re-derive from it.
	 * One request per row at a time. Resolves to whether the backend accepted it.
	 */
	async function markUnread(itemId: string): Promise<boolean> {
		if (!visible(itemId) || unreadInFlight.has(itemId)) return false;
		unreadInFlight.add(itemId);
		const op = begin(itemId, {
			fields: { last_read_at: null, progress_percent: null, max_progress_percent: null },
			removed: false
		});

		try {
			await apiSdk.markDocumentUnread({ path: { document_id: itemId } });
			commitFields(itemId, op.fields);
			end(itemId, op, op.fields);
			ctx.refetchIfBackendOwnsFilter();
			return true;
		} catch {
			end(itemId, op, null);
			return false;
		} finally {
			unreadInFlight.delete(itemId);
		}
	}

	return { overlay, triageAction, deleteAction, markUnread };
}
