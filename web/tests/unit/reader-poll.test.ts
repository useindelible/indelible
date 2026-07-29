import { describe, expect, it } from 'vitest';
import { computeReaderPollState } from '../../src/routes/(app)/reader/[documentId]/reader-poll';

describe('reader poll state', () => {
	it('keeps polling silently before the retry threshold', () => {
		const result = computeReaderPollState({
			ready: false,
			canPoll: true,
			now: 10_000,
			startedAt: undefined
		});

		expect(result).toEqual({
			startedAt: 10_000,
			showRetry: false,
			shouldPoll: true
		});
	});

	it('shows retry after a long preparation wait', () => {
		const result = computeReaderPollState({
			ready: false,
			canPoll: true,
			now: 40_001,
			startedAt: 10_000
		});

		expect(result).toEqual({
			startedAt: 10_000,
			showRetry: true,
			shouldPoll: true
		});
	});

	it('resets retry state when readable content is ready or polling is unavailable', () => {
		expect(
			computeReaderPollState({
				ready: true,
				canPoll: true,
				now: 40_001,
				startedAt: 10_000
			})
		).toEqual({
			startedAt: undefined,
			showRetry: false,
			shouldPoll: false
		});

		expect(
			computeReaderPollState({
				ready: false,
				canPoll: false,
				now: 40_001,
				startedAt: 10_000
			})
		).toEqual({
			startedAt: undefined,
			showRetry: false,
			shouldPoll: false
		});
	});
});
