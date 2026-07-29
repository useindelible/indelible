import * as apiSdk from '$lib/api';
import type { SessionManifestResponse } from '$lib/api/generated/types.gen';
import type { TtsSessionChunk } from './tts-controller-model';
import type { TtsTimingEntry } from './tts-timing';

type TimingStore = {
	set: (elementIndex: number, timing: TtsTimingEntry) => unknown;
};

export function createTtsAudio(
	rawAudioUrl: string,
	apiBaseUrl: string,
	speed: number
): HTMLAudioElement {
	const isPassthrough = rawAudioUrl.startsWith('/');
	const audioUrl = isPassthrough ? `${apiBaseUrl}${rawAudioUrl}` : rawAudioUrl;
	const audio = new Audio();
	if (isPassthrough) {
		audio.crossOrigin = 'use-credentials';
	}
	audio.preload = 'auto';
	audio.src = audioUrl;
	audio.playbackRate = speed;
	return audio;
}

export function waitForMetadata(audio: HTMLAudioElement): Promise<void> {
	return new Promise((resolve) => {
		if (audio.readyState >= 1) {
			resolve();
			return;
		}
		const handler = () => {
			audio.removeEventListener('loadedmetadata', handler);
			audio.removeEventListener('error', handler);
			resolve();
		};
		audio.addEventListener('loadedmetadata', handler);
		audio.addEventListener('error', handler);
	});
}

export function waitForSeek(audio: HTMLAudioElement, timeoutMs = 1200): Promise<void> {
	return new Promise((resolve) => {
		let settled = false;
		const finish = () => {
			if (settled) return;
			settled = true;
			clearTimeout(timeout);
			audio.removeEventListener('seeked', finish);
			audio.removeEventListener('error', finish);
			resolve();
		};
		const timeout = setTimeout(finish, timeoutMs);
		audio.addEventListener('seeked', finish);
		audio.addEventListener('error', finish);
	});
}

export async function seekTtsAudioTo(
	audio: HTMLAudioElement,
	targetTimestamp: number
): Promise<number> {
	const target = Math.max(0, targetTimestamp);
	if (!Number.isFinite(target) || target <= 0) return audio.currentTime;
	const seeked = waitForSeek(audio);
	audio.currentTime = target;
	await seeked;
	if (Math.abs(audio.currentTime - target) > 0.5) {
		audio.currentTime = target;
	}
	return audio.currentTime;
}

export async function loadTtsChunkTimings(
	documentId: string,
	manifest: SessionManifestResponse,
	chunk: TtsSessionChunk,
	timingCache: TimingStore
): Promise<void> {
	if (chunk.timings?.length) {
		for (const timing of chunk.timings) {
			timingCache.set(timing.element_index, {
				start: timing.start_timestamp,
				end: timing.end_timestamp ?? null
			});
		}
		return;
	}

	await prefetchTimestamps(
		documentId,
		manifest.session.id,
		chunk.chunk_id,
		chunk.start_element_index,
		chunk.end_element_index,
		timingCache
	);
}

async function prefetchTimestamps(
	documentId: string,
	sessionId: string,
	chunkId: string,
	startIdx: number,
	endIdx: number,
	timingCache: TimingStore
) {
	const indices = Array.from({ length: endIdx - startIdx + 1 }, (_, i) => startIdx + i);
	const results = await Promise.allSettled(
		indices.map((idx) =>
			apiSdk.resolveDocumentTtsTimestamp({
				path: { document_id: documentId },
				query: { session_id: sessionId, chunk_id: chunkId, element_index: idx }
			})
		)
	);
	results.forEach((result, i) => {
		if (result.status === 'fulfilled' && result.value.data) {
			timingCache.set(startIdx + i, {
				start: result.value.data.start_timestamp,
				end: result.value.data.end_timestamp ?? null
			});
		}
	});
}
