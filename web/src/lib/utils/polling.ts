/**
 * Lightweight interval-based polling helper. The route layer composes its own
 * stop/start logic on top: this utility owns the timer, supports a `shouldStop`
 * predicate evaluated against each fetch result, and exposes a transient-error
 * tolerance so a single 5xx blip does not abort an in-flight import.
 */

export interface PollResult<T> {
	value?: T;
	error?: { httpStatus: number; message: string };
	terminal?: boolean;
}

export interface CreatePollOptions<T> {
	fetcher: () => Promise<PollResult<T>>;
	intervalMs: number;
	shouldStop: (result: PollResult<T>) => boolean;
	maxConsecutiveErrors?: number;
	onUpdate?: (result: PollResult<T>) => void;
	onError?: (error: { httpStatus: number; message: string }) => void;
	onStop?: () => void;
}

export interface PollHandle {
	start: () => void;
	stop: () => void;
	isRunning: () => boolean;
}

export function createPoll<T>(options: CreatePollOptions<T>): PollHandle {
	const {
		fetcher,
		intervalMs,
		shouldStop,
		maxConsecutiveErrors = 3,
		onUpdate,
		onError,
		onStop
	} = options;

	let timer: ReturnType<typeof setTimeout> | null = null;
	let consecutiveErrors = 0;
	let running = false;
	let inflight = false;

	function clearTimer() {
		if (timer !== null) {
			clearTimeout(timer);
			timer = null;
		}
	}

	function schedule() {
		clearTimer();
		if (!running) return;
		timer = setTimeout(() => {
			void tick();
		}, intervalMs);
	}

	function stop() {
		if (!running) return;
		running = false;
		clearTimer();
		onStop?.();
	}

	async function tick() {
		if (!running || inflight) return;
		inflight = true;
		try {
			const result = await fetcher();
			if (!running) return;

			if (result.error) {
				consecutiveErrors += 1;
				onError?.(result.error);
				if (consecutiveErrors >= maxConsecutiveErrors || result.terminal) {
					stop();
					return;
				}
			} else {
				consecutiveErrors = 0;
				onUpdate?.(result);
				if (shouldStop(result)) {
					stop();
					return;
				}
			}
			schedule();
		} finally {
			inflight = false;
		}
	}

	return {
		start() {
			if (running) return;
			running = true;
			consecutiveErrors = 0;
			void tick();
		},
		stop,
		isRunning: () => running
	};
}
