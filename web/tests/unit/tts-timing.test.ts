import { describe, expect, it } from 'vitest';
import {
	createTtsResumeAnchor,
	selectActiveTtsElement,
	timestampForResumeAnchor,
	type TtsTimingEntry
} from '$lib/components/reader/tts-timing';

const chunk = {
	start_element_index: 0,
	end_element_index: 2,
	duration_seconds: 9,
	timing_source: 'provider_transcript' as const
};

function timings(entries: Array<[number, TtsTimingEntry]>) {
	return new Map(entries);
}

describe('tts timing selection', () => {
	it('selects by timing range instead of start-only timestamps', () => {
		const map = timings([
			[0, { start: 0, end: 3 }],
			[1, { start: 3, end: 6 }],
			[2, { start: 6, end: 9 }]
		]);

		expect(selectActiveTtsElement(chunk, map, 2.9)).toBe(0);
		expect(selectActiveTtsElement(chunk, map, 3.1)).toBe(1);
		expect(selectActiveTtsElement(chunk, map, 6.1)).toBe(2);
	});

	it('uses the final-element grace only for heuristic timings', () => {
		const map = timings([
			[0, { start: 0, end: 4 }],
			[1, { start: 4, end: 8 }],
			[2, { start: 8, end: 8.4 }]
		]);

		expect(selectActiveTtsElement({ ...chunk, timing_source: 'heuristic' }, map, 8.8, 9)).toBe(2);
	});

	it('keeps provider transcript ranges authoritative near the end of a chunk', () => {
		const map = timings([
			[0, { start: 0, end: 4 }],
			[1, { start: 4, end: 8.75 }],
			[2, { start: 8.95, end: 9 }]
		]);

		expect(selectActiveTtsElement(chunk, map, 8.8, 9)).toBe(1);
	});

	it('captures resume progress inside the current spoken element', () => {
		const map = timings([
			[0, { start: 0, end: 4 }],
			[1, { start: 4, end: 10 }],
			[2, { start: 10, end: 14 }]
		]);

		const anchor = createTtsResumeAnchor(chunk, map, 7);

		expect(anchor.elementIndex).toBe(1);
		expect(anchor.progressInElement).toBeCloseTo(0.5);
		expect(anchor.progressInChunk).toBeCloseTo(7 / 9);
	});

	it('maps a resume anchor onto regenerated voice timings', () => {
		const map = timings([[1, { start: 8, end: 20 }]]);

		expect(
			timestampForResumeAnchor(map, {
				elementIndex: 1,
				progressInElement: 0.5,
				progressInChunk: null
			})
		).toBe(14);
	});

	it('falls back to chunk progress when element timing range is unavailable', () => {
		const map = timings([[1, { start: 0, end: null }]]);

		expect(
			timestampForResumeAnchor(
				map,
				{ elementIndex: 1, progressInElement: 0, progressInChunk: 0.5 },
				{
					start_element_index: 1,
					end_element_index: 2,
					duration_seconds: 20,
					timing_source: 'heuristic'
				}
			)
		).toBe(10);
	});

	it('does not infer chunk progress for provider transcript timings', () => {
		const map = timings([[1, { start: 4, end: null }]]);

		expect(
			timestampForResumeAnchor(
				map,
				{ elementIndex: 1, progressInElement: 0, progressInChunk: 0.5 },
				{
					start_element_index: 1,
					end_element_index: 2,
					duration_seconds: 20,
					timing_source: 'provider_transcript'
				}
			)
		).toBe(4);
	});

	it('clamps resume progress inside regenerated timing ranges', () => {
		const map = timings([[1, { start: 8, end: 20 }]]);

		expect(
			timestampForResumeAnchor(map, {
				elementIndex: 1,
				progressInElement: 2,
				progressInChunk: null
			})
		).toBe(20);
	});
});
