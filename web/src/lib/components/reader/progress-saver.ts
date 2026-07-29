import type { UpdateDocumentProgressBody } from '$lib/api/generated/types.gen';

export type ProgressSaveInput = {
	progress_percent: number;
	chapter_locator?: string | null;
	chapter_offset?: number | null;
};

type SaveFn = (body: UpdateDocumentProgressBody) => Promise<unknown>;

const IDLE_SAVE_MS = 800;
const ACTIVE_SAVE_MS = 5000;

function roundPercent(value: number): number {
	return Math.round(Math.min(100, Math.max(0, value)) * 100) / 100;
}

function normalize(input: ProgressSaveInput): UpdateDocumentProgressBody {
	return {
		progress_percent: roundPercent(input.progress_percent),
		chapter_locator: input.chapter_locator ?? null,
		chapter_offset: input.chapter_offset ?? null
	};
}

function sameBody(a: UpdateDocumentProgressBody | null, b: UpdateDocumentProgressBody): boolean {
	return (
		a?.progress_percent === b.progress_percent &&
		(a.chapter_locator ?? null) === (b.chapter_locator ?? null) &&
		(a.chapter_offset ?? null) === (b.chapter_offset ?? null)
	);
}

export function createProgressSaver(save: SaveFn) {
	let idleTimer: ReturnType<typeof setTimeout> | undefined;
	let pending: UpdateDocumentProgressBody | null = null;
	let lastSaved: UpdateDocumentProgressBody | null = null;
	let activeWindowStartedAt: number | null = null;
	let inFlight = false;
	let flushAgain = false;
	let disposed = false;

	function clearIdleTimer() {
		if (idleTimer) {
			clearTimeout(idleTimer);
			idleTimer = undefined;
		}
	}

	async function flush(): Promise<void> {
		clearIdleTimer();
		if (inFlight) {
			flushAgain = true;
			return;
		}
		if (!pending || sameBody(lastSaved, pending)) return;

		const body = pending;
		pending = null;
		inFlight = true;
		flushAgain = false;
		activeWindowStartedAt = null;
		try {
			await save(body);
			lastSaved = body;
		} finally {
			inFlight = false;
			if ((flushAgain || pending) && pending && !sameBody(lastSaved, pending)) {
				void flush();
			}
		}
	}

	function update(input: ProgressSaveInput): void {
		if (disposed) return;
		const next = normalize(input);
		if (sameBody(lastSaved, next) && !inFlight) {
			pending = null;
			activeWindowStartedAt = null;
			clearIdleTimer();
			return;
		}

		pending = next;
		const now = Date.now();
		if (activeWindowStartedAt === null) {
			activeWindowStartedAt = now;
		}
		if (now - activeWindowStartedAt >= ACTIVE_SAVE_MS) {
			void flush();
			return;
		}

		clearIdleTimer();
		idleTimer = setTimeout(() => {
			void flush();
		}, IDLE_SAVE_MS);
	}

	function destroy(): void {
		disposed = true;
		void flush();
	}

	return { update, flush, destroy };
}
