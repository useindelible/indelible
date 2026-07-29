import { describe, expect, it } from 'vitest';

import type { SessionManifestResponse } from '$lib/api/generated/types.gen';
import {
	countForwardReadyChunks,
	findNextReadyChunk,
	findReadyChunkForElement,
	formatTtsResumePosition,
	getTtsUnavailableBanner,
	messageForTtsError
} from '$lib/components/reader/tts-controller-model';

type SessionChunk = SessionManifestResponse['chunks'][number];

function chunk(overrides: Partial<SessionChunk>): SessionChunk {
	return {
		chunk_id: 'chunk_1',
		position: 0,
		state: 'ready',
		audio_url: '/audio/1',
		start_element_index: 0,
		end_element_index: 2,
		duration_seconds: 9,
		timing_source: 'provider_transcript',
		timings: [],
		...overrides
	} as SessionChunk;
}

function manifest(chunks: SessionChunk[]): SessionManifestResponse {
	return {
		session: { id: 'session_1' },
		start: { chunk_id: chunks[0]?.chunk_id ?? null, element_index: 0, start_timestamp: 0 },
		chunks
	} as SessionManifestResponse;
}

describe('tts controller model', () => {
	it('maps unavailable messages to banner copy', () => {
		expect(getTtsUnavailableBanner('No readable content found')).toEqual({
			variant: 'setup',
			title: 'No readable content',
			message: 'TTS is not available for this document.'
		});

		expect(getTtsUnavailableBanner('disabled')).toEqual({
			variant: 'error',
			title: 'TTS unavailable',
			message: 'disabled'
		});
	});

	it('formats saved resume positions', () => {
		expect(formatTtsResumePosition(75)).toBe('1:15');
		expect(formatTtsResumePosition(3671)).toBe('1:01:11');
		expect(formatTtsResumePosition(Number.NaN)).toBe('0:00');
	});

	it('normalizes TTS startup errors', () => {
		expect(messageForTtsError('quota exceeded')).toMatch(/quota exhausted/i);
		expect(messageForTtsError({ status: 503 })).toMatch(/not enabled/i);
		expect(messageForTtsError(new Error('unknown'))).toMatch(/Could not start audio/i);
	});

	it('selects ready chunks by element and adjacency', () => {
		const activeManifest = manifest([
			chunk({ chunk_id: 'a', position: 0, start_element_index: 0, end_element_index: 2 }),
			chunk({ chunk_id: 'b', position: 1, start_element_index: 3, end_element_index: 5 }),
			chunk({
				chunk_id: 'c',
				position: 2,
				start_element_index: 6,
				end_element_index: 8,
				state: 'queued'
			})
		]);

		expect(findReadyChunkForElement(activeManifest, 4)?.chunk_id).toBe('b');
		expect(findNextReadyChunk(activeManifest, 'a')?.chunk_id).toBe('b');
		expect(countForwardReadyChunks(activeManifest, activeManifest.chunks[0]!)).toBe(1);
	});
});
