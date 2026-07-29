import type { TtsTimingSourceDto } from '$lib/api/generated/types.gen';

export type TtsTimingEntry = {
	start: number;
	end: number | null;
};

export type TtsTimingChunk = {
	start_element_index: number;
	end_element_index: number;
	duration_seconds?: number | null;
	timing_source: TtsTimingSourceDto;
};

export type TtsResumeAnchor = {
	elementIndex: number;
	progressInElement: number;
	progressInChunk: number | null;
};

const FINAL_ELEMENT_GRACE_SECONDS = 0.25;

export function selectActiveTtsElement(
	chunk: TtsTimingChunk,
	timings: Iterable<[number, TtsTimingEntry]>,
	playbackTime: number,
	observedDuration = 0
): number {
	const entries = Array.from(timings)
		.filter(([idx, timing]) => {
			return (
				chunk.start_element_index <= idx &&
				idx <= chunk.end_element_index &&
				Number.isFinite(timing.start)
			);
		})
		.sort(([a], [b]) => a - b);

	if (entries.length === 0) {
		return chunk.start_element_index;
	}

	const manifestDuration = chunk.duration_seconds ?? 0;
	const effectiveDuration =
		Number.isFinite(observedDuration) && observedDuration > 0 ? observedDuration : manifestDuration;
	if (
		chunk.timing_source === 'heuristic' &&
		Number.isFinite(effectiveDuration) &&
		effectiveDuration > 0 &&
		playbackTime >= Math.max(0, effectiveDuration - FINAL_ELEMENT_GRACE_SECONDS)
	) {
		return chunk.end_element_index;
	}

	let latestIndex = chunk.start_element_index;
	let latestStart = -Infinity;
	for (const [idx, timing] of entries) {
		if (playbackTime >= timing.start && (timing.end == null || playbackTime < timing.end)) {
			return idx;
		}
		if (timing.start <= playbackTime && timing.start > latestStart) {
			latestIndex = idx;
			latestStart = timing.start;
		}
	}

	return latestIndex;
}

export function createTtsResumeAnchor(
	chunk: TtsTimingChunk,
	timings: Iterable<[number, TtsTimingEntry]>,
	playbackTime: number,
	observedDuration = 0
): TtsResumeAnchor {
	const entries = new Map(timings);
	const elementIndex = selectActiveTtsElement(chunk, entries, playbackTime, observedDuration);
	const manifestDuration = chunk.duration_seconds ?? 0;
	const effectiveDuration =
		Number.isFinite(observedDuration) && observedDuration > 0 ? observedDuration : manifestDuration;
	const progressInChunk =
		Number.isFinite(effectiveDuration) && effectiveDuration > 0
			? Math.min(1, Math.max(0, playbackTime / effectiveDuration))
			: null;
	const timing = entries.get(elementIndex);
	if (
		!timing ||
		!Number.isFinite(timing.start) ||
		timing.end == null ||
		timing.end <= timing.start
	) {
		return { elementIndex, progressInElement: 0, progressInChunk };
	}

	const progressInElement = Math.min(
		1,
		Math.max(0, (playbackTime - timing.start) / (timing.end - timing.start))
	);
	return { elementIndex, progressInElement, progressInChunk };
}

export function timestampForResumeAnchor(
	timings: ReadonlyMap<number, TtsTimingEntry>,
	anchor: TtsResumeAnchor,
	chunk?: TtsTimingChunk
): number {
	const timing = timings.get(anchor.elementIndex);
	if (
		timing &&
		Number.isFinite(timing.start) &&
		timing.end != null &&
		Number.isFinite(timing.end) &&
		timing.end > timing.start
	) {
		const progressInElement = Math.min(1, Math.max(0, anchor.progressInElement));
		return timing.start + (timing.end - timing.start) * progressInElement;
	}

	if (
		chunk?.timing_source === 'heuristic' &&
		chunk.duration_seconds != null &&
		Number.isFinite(chunk.duration_seconds) &&
		chunk.duration_seconds > 0 &&
		anchor.progressInChunk != null
	) {
		return chunk.duration_seconds * Math.min(1, Math.max(0, anchor.progressInChunk));
	}

	if (timing && Number.isFinite(timing.start)) {
		return timing.start;
	}

	return 0;
}
