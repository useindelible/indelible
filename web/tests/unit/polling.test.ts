import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { createPoll, type PollResult } from '$lib/utils/polling';

describe('createPoll', () => {
	beforeEach(() => {
		vi.useFakeTimers();
	});

	afterEach(() => {
		vi.useRealTimers();
	});

	function makeFetcher(results: PollResult<{ status: string }>[]) {
		const fn = vi.fn();
		for (const r of results) fn.mockResolvedValueOnce(r);
		return fn;
	}

	it('invokes the fetcher immediately on start', async () => {
		const fetcher = makeFetcher([{ value: { status: 'pending' } }]);
		const handle = createPoll<{ status: string }>({
			fetcher,
			intervalMs: 1000,
			shouldStop: () => false
		});
		handle.start();
		await vi.waitFor(() => expect(fetcher).toHaveBeenCalledTimes(1));
		handle.stop();
	});

	it('schedules subsequent fetches at the configured interval', async () => {
		const fetcher = makeFetcher([
			{ value: { status: 'pending' } },
			{ value: { status: 'pending' } },
			{ value: { status: 'pending' } }
		]);
		const handle = createPoll<{ status: string }>({
			fetcher,
			intervalMs: 2000,
			shouldStop: () => false
		});
		handle.start();
		await vi.waitFor(() => expect(fetcher).toHaveBeenCalledTimes(1));

		await vi.advanceTimersByTimeAsync(2000);
		expect(fetcher).toHaveBeenCalledTimes(2);

		await vi.advanceTimersByTimeAsync(2000);
		expect(fetcher).toHaveBeenCalledTimes(3);

		handle.stop();
	});

	it('stops when shouldStop returns true', async () => {
		const fetcher = makeFetcher([
			{ value: { status: 'running' } },
			{ value: { status: 'completed' } },
			{ value: { status: 'completed' } }
		]);
		const onUpdate = vi.fn();
		const onStop = vi.fn();
		const handle = createPoll<{ status: string }>({
			fetcher,
			intervalMs: 1000,
			shouldStop: (r) => r.value?.status === 'completed',
			onUpdate,
			onStop
		});
		handle.start();
		await vi.waitFor(() => expect(fetcher).toHaveBeenCalledTimes(1));

		await vi.advanceTimersByTimeAsync(1000);
		await vi.waitFor(() => expect(onStop).toHaveBeenCalled());

		expect(handle.isRunning()).toBe(false);

		await vi.advanceTimersByTimeAsync(5000);
		expect(fetcher).toHaveBeenCalledTimes(2);
	});

	it('stops when the caller marks awaiting_provider as terminal', async () => {
		const fetcher = makeFetcher([
			{ value: { status: 'pending' } },
			{ value: { status: 'awaiting_provider' } }
		]);
		const onStop = vi.fn();
		const handle = createPoll<{ status: string }>({
			fetcher,
			intervalMs: 1000,
			shouldStop: (r) => {
				if (!r.value) return false;
				if (r.value.status === 'awaiting_provider') return true;
				return r.value.status !== 'pending' && r.value.status !== 'running';
			},
			onStop
		});
		handle.start();
		await vi.waitFor(() => expect(fetcher).toHaveBeenCalledTimes(1));
		await vi.advanceTimersByTimeAsync(1000);
		await vi.waitFor(() => expect(onStop).toHaveBeenCalled());

		expect(handle.isRunning()).toBe(false);
	});

	it('tolerates fewer than maxConsecutiveErrors transient errors', async () => {
		const fetcher = makeFetcher([
			{ error: { httpStatus: 503, message: 'unavailable' } },
			{ error: { httpStatus: 503, message: 'unavailable' } },
			{ value: { status: 'completed' } }
		]);
		const onError = vi.fn();
		const onStop = vi.fn();
		const handle = createPoll<{ status: string }>({
			fetcher,
			intervalMs: 1000,
			shouldStop: (r) => r.value?.status === 'completed',
			maxConsecutiveErrors: 3,
			onError,
			onStop
		});
		handle.start();
		await vi.waitFor(() => expect(fetcher).toHaveBeenCalledTimes(1));
		await vi.advanceTimersByTimeAsync(1000);
		await vi.waitFor(() => expect(fetcher).toHaveBeenCalledTimes(2));
		await vi.advanceTimersByTimeAsync(1000);
		await vi.waitFor(() => expect(fetcher).toHaveBeenCalledTimes(3));
		await vi.waitFor(() => expect(onStop).toHaveBeenCalled());
		expect(onError).toHaveBeenCalledTimes(2);
	});

	it('stops after maxConsecutiveErrors transient errors', async () => {
		const fetcher = makeFetcher([
			{ error: { httpStatus: 500, message: 'boom' } },
			{ error: { httpStatus: 500, message: 'boom' } },
			{ error: { httpStatus: 500, message: 'boom' } }
		]);
		const onStop = vi.fn();
		const handle = createPoll<{ status: string }>({
			fetcher,
			intervalMs: 1000,
			shouldStop: () => false,
			maxConsecutiveErrors: 3,
			onStop
		});
		handle.start();
		await vi.waitFor(() => expect(fetcher).toHaveBeenCalledTimes(1));
		await vi.advanceTimersByTimeAsync(1000);
		await vi.waitFor(() => expect(fetcher).toHaveBeenCalledTimes(2));
		await vi.advanceTimersByTimeAsync(1000);
		await vi.waitFor(() => expect(fetcher).toHaveBeenCalledTimes(3));
		await vi.waitFor(() => expect(onStop).toHaveBeenCalled());
		expect(handle.isRunning()).toBe(false);
	});

	it('stops immediately on terminal error', async () => {
		const fetcher = makeFetcher([
			{ error: { httpStatus: 404, message: 'not found' }, terminal: true }
		]);
		const onStop = vi.fn();
		const handle = createPoll<{ status: string }>({
			fetcher,
			intervalMs: 1000,
			shouldStop: () => false,
			onStop
		});
		handle.start();
		await vi.waitFor(() => expect(onStop).toHaveBeenCalled());
		expect(handle.isRunning()).toBe(false);
	});

	it('resets the consecutive error counter on a successful fetch', async () => {
		const fetcher = makeFetcher([
			{ error: { httpStatus: 503, message: 'oops' } },
			{ error: { httpStatus: 503, message: 'oops' } },
			{ value: { status: 'pending' } },
			{ error: { httpStatus: 503, message: 'oops' } },
			{ error: { httpStatus: 503, message: 'oops' } },
			{ value: { status: 'completed' } }
		]);
		const handle = createPoll<{ status: string }>({
			fetcher,
			intervalMs: 1000,
			shouldStop: (r) => r.value?.status === 'completed',
			maxConsecutiveErrors: 3
		});
		handle.start();
		await vi.waitFor(() => expect(fetcher).toHaveBeenCalledTimes(1));
		for (let i = 0; i < 5; i += 1) {
			await vi.advanceTimersByTimeAsync(1000);
			await vi.waitFor(() => expect(fetcher).toHaveBeenCalledTimes(i + 2));
		}
		expect(handle.isRunning()).toBe(false);
	});

	it('does not start twice', async () => {
		const fetcher = makeFetcher([
			{ value: { status: 'pending' } },
			{ value: { status: 'pending' } }
		]);
		const handle = createPoll<{ status: string }>({
			fetcher,
			intervalMs: 1000,
			shouldStop: () => false
		});
		handle.start();
		handle.start();
		await vi.waitFor(() => expect(fetcher).toHaveBeenCalledTimes(1));
		handle.stop();
	});

	it('stop() prevents pending fetches from continuing', async () => {
		const fetcher = makeFetcher([
			{ value: { status: 'pending' } },
			{ value: { status: 'pending' } }
		]);
		const handle = createPoll<{ status: string }>({
			fetcher,
			intervalMs: 1000,
			shouldStop: () => false
		});
		handle.start();
		await vi.waitFor(() => expect(fetcher).toHaveBeenCalledTimes(1));
		handle.stop();
		await vi.advanceTimersByTimeAsync(5000);
		expect(fetcher).toHaveBeenCalledTimes(1);
	});
});
