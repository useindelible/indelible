export const READER_POLL_INTERVAL_MS = 2_000;
export const READER_RETRY_AFTER_MS = 30_000;

interface ReaderPollStateInput {
	ready: boolean;
	canPoll: boolean;
	now: number;
	startedAt: number | undefined;
	retryAfterMs?: number;
}

interface ReaderPollState {
	startedAt: number | undefined;
	showRetry: boolean;
	shouldPoll: boolean;
}

interface ReaderPollControllerOptions {
	canPoll: () => boolean;
	onPoll: () => void;
	onRetryVisibleChange: (visible: boolean) => void;
	now?: () => number;
	pollIntervalMs?: number;
	retryAfterMs?: number;
}

export function computeReaderPollState({
	ready,
	canPoll,
	now,
	startedAt,
	retryAfterMs = READER_RETRY_AFTER_MS
}: ReaderPollStateInput): ReaderPollState {
	if (ready || !canPoll) {
		return {
			startedAt: undefined,
			showRetry: false,
			shouldPoll: false
		};
	}

	const nextStartedAt = startedAt ?? now;
	return {
		startedAt: nextStartedAt,
		showRetry: now - nextStartedAt >= retryAfterMs,
		shouldPoll: true
	};
}

export function createReaderPollController({
	canPoll,
	onPoll,
	onRetryVisibleChange,
	now = () => Date.now(),
	pollIntervalMs = READER_POLL_INTERVAL_MS,
	retryAfterMs = READER_RETRY_AFTER_MS
}: ReaderPollControllerOptions) {
	let timer: ReturnType<typeof setTimeout> | undefined;
	let startedAt: number | undefined;
	let retryVisible = false;

	function clearTimer() {
		if (!timer) return;
		clearTimeout(timer);
		timer = undefined;
	}

	function setRetryVisible(visible: boolean) {
		retryVisible = visible;
		onRetryVisibleChange(visible);
	}

	function reset() {
		clearTimer();
		startedAt = undefined;
		if (retryVisible) setRetryVisible(false);
	}

	function schedule(ready: boolean) {
		clearTimer();
		const next = computeReaderPollState({
			ready,
			canPoll: canPoll(),
			now: now(),
			startedAt,
			retryAfterMs
		});
		startedAt = next.startedAt;
		if (retryVisible !== next.showRetry) setRetryVisible(next.showRetry);
		if (next.shouldPoll) timer = setTimeout(onPoll, pollIntervalMs);
	}

	function retry() {
		// Reset the long-wait timer so the hint hides until preparation is slow again.
		reset();
		onPoll();
	}

	return {
		reset,
		retry,
		schedule,
		destroy: clearTimer
	};
}
