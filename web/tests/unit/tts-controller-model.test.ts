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
		expect(getTtsUnavailableBanner('reader_tts_no_readable_content')).toEqual({
			variant: 'setup',
			titleKey: 'reader_tts_no_readable_content',
			messageKey: 'reader_tts_not_available_document'
		});

		expect(getTtsUnavailableBanner('reader_tts_not_enabled')).toEqual({
			variant: 'error',
			titleKey: 'reader_tts_unavailable',
			messageKey: 'reader_tts_not_enabled'
		});
	});

	it('formats saved resume positions', () => {
		expect(formatTtsResumePosition(75)).toBe('1:15');
		expect(formatTtsResumePosition(3671)).toBe('1:01:11');
		expect(formatTtsResumePosition(Number.NaN)).toBe('0:00');
	});

	it('normalizes TTS startup errors', () => {
		expect(messageForTtsError('quota exceeded')).toBe('reader_tts_quota_exhausted');
		expect(messageForTtsError({ status: 503 })).toBe('reader_tts_not_enabled');
		expect(messageForTtsError(new Error('unknown'))).toBe('reader_tts_could_not_start');
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
