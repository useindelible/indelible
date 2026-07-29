import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';

import { createTocStore } from './toc-store.svelte';

type TocPayload = {
	status: 'ready' | 'none' | 'pending';
	truncated: boolean;
	entries: {
		source_heading_index: number;
		id: string;
		title: string;
		depth: number;
		word_count: number;
	}[];
};

function fetcherReturning(sequence: TocPayload[]) {
	let calls = 0;
	const fetcher = vi.fn(async () => {
		const payload = sequence[Math.min(calls, sequence.length - 1)];
		calls += 1;
		return { data: payload };
	});
	return fetcher;
}

const READY: TocPayload = {
	status: 'ready',
	truncated: false,
	entries: [{ source_heading_index: 0, id: 'ind-toc-a', title: 'A', depth: 0, word_count: 10 }]
};
const PENDING: TocPayload = { status: 'pending', truncated: false, entries: [] };
const NONE: TocPayload = { status: 'none', truncated: false, entries: [] };

describe('createTocStore', () => {
	beforeEach(() => {
		vi.useFakeTimers();
	});
	afterEach(() => {
		vi.useRealTimers();
	});

	it('transitions pending to ready without a manual refresh', async () => {
		const fetcher = fetcherReturning([PENDING, PENDING, READY]);
		const store = createTocStore('doc_1', fetcher);
		store.start();
		await vi.advanceTimersByTimeAsync(0);
		expect(store.state.kind).toBe('pending');

		await vi.advanceTimersByTimeAsync(2000);
		expect(store.state.kind).toBe('pending');
		await vi.advanceTimersByTimeAsync(4000);
		expect(store.state.kind).toBe('ready');
		expect(fetcher).toHaveBeenCalledTimes(3);
		if (store.state.kind === 'ready') {
			expect(store.state.entries[0]?.id).toBe('ind-toc-a');
		}

		// Terminal: no further polling ever fires.
		await vi.advanceTimersByTimeAsync(120_000);
		expect(fetcher).toHaveBeenCalledTimes(3);
	});

	it('hides on none and never polls again', async () => {
		const fetcher = fetcherReturning([NONE]);
		const store = createTocStore('doc_1', fetcher);
		store.start();
		await vi.advanceTimersByTimeAsync(0);
		expect(store.state.kind).toBe('hidden');
		await vi.advanceTimersByTimeAsync(120_000);
		expect(fetcher).toHaveBeenCalledTimes(1);
	});

	it('hides on fetch failure', async () => {
		const fetcher = vi.fn(async () => {
			throw new Error('network');
		});
		const store = createTocStore('doc_1', fetcher);
		store.start();
		await vi.advanceTimersByTimeAsync(0);
		expect(store.state.kind).toBe('hidden');
	});

	it('stop clears the scheduled poll', async () => {
		const fetcher = fetcherReturning([PENDING, READY]);
		const store = createTocStore('doc_1', fetcher);
		store.start();
		await vi.advanceTimersByTimeAsync(0);
		expect(store.state.kind).toBe('pending');
		store.stop();
		await vi.advanceTimersByTimeAsync(60_000);
		expect(fetcher).toHaveBeenCalledTimes(1);
	});

	it('gives up after the polling budget and hides', async () => {
		const fetcher = fetcherReturning([PENDING]);
		const store = createTocStore('doc_1', fetcher);
		store.start();
		await vi.advanceTimersByTimeAsync(0);
		// Budget is five minutes; backoff caps at 30s, so this covers it.
		await vi.advanceTimersByTimeAsync(6 * 60_000);
		expect(store.state.kind).toBe('hidden');
		const callsAtBudget = fetcher.mock.calls.length;
		await vi.advanceTimersByTimeAsync(120_000);
		expect(fetcher.mock.calls.length).toBe(callsAtBudget);
	});
});
