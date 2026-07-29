import { getArticleToc, type ArticleTocEntry } from '$lib/api';

export type TocState =
	| { kind: 'loading' }
	| { kind: 'pending' }
	| { kind: 'hidden' }
	| { kind: 'ready'; entries: ArticleTocEntry[]; truncated: boolean };

type TocPayload = {
	status: 'ready' | 'none' | 'pending';
	truncated: boolean;
	entries: ArticleTocEntry[];
};

export type TocFetcher = (documentId: string) => Promise<{ data: TocPayload | undefined }>;

const INITIAL_DELAY_MS = 2_000;
const MAX_DELAY_MS = 30_000;
// A backfill normally lands in seconds; past this budget the outline is
// effectively unavailable for this session and the affordance hides.
const POLL_BUDGET_MS = 5 * 60_000;

const defaultFetcher: TocFetcher = (documentId) =>
	getArticleToc({ path: { document_id: documentId } });

export function createTocStore(documentId: string, fetchToc: TocFetcher = defaultFetcher) {
	let state = $state<TocState>({ kind: 'loading' });
	let timer: ReturnType<typeof setTimeout> | undefined;
	let delay = INITIAL_DELAY_MS;
	let elapsed = 0;
	let stopped = false;

	async function poll(): Promise<void> {
		let payload: TocPayload | undefined;
		try {
			({ data: payload } = await fetchToc(documentId));
		} catch {
			payload = undefined;
		}
		if (stopped) return;
		if (!payload || payload.status === 'none') {
			state = { kind: 'hidden' };
			return;
		}
		if (payload.status === 'ready') {
			state = { kind: 'ready', entries: payload.entries, truncated: payload.truncated };
			return;
		}
		if (elapsed >= POLL_BUDGET_MS) {
			state = { kind: 'hidden' };
			return;
		}
		state = { kind: 'pending' };
		timer = setTimeout(() => {
			void poll();
		}, delay);
		elapsed += delay;
		delay = Math.min(delay * 2, MAX_DELAY_MS);
	}

	return {
		get state(): TocState {
			return state;
		},
		start(): void {
			stopped = false;
			void poll();
		},
		stop(): void {
			stopped = true;
			if (timer) clearTimeout(timer);
			timer = undefined;
		}
	};
}
