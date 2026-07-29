import type { SessionManifestResponse } from '$lib/api/generated/types.gen';

export type TtsStatus = 'idle' | 'loading' | 'playing' | 'paused' | 'unavailable';
export type TtsSessionChunk = SessionManifestResponse['chunks'][number];

export type TtsUnavailableBanner = {
	variant: 'setup' | 'error';
	title: string;
	message: string;
};

export function getTtsUnavailableBanner(message: string): TtsUnavailableBanner {
	if (/content|readable/i.test(message)) {
		return {
			variant: 'setup',
			title: 'No readable content',
			message: 'TTS is not available for this document.'
		};
	}
	return {
		variant: 'error',
		title: 'TTS unavailable',
		message: message || 'Audio playback is not available right now.'
	};
}

export function messageForTtsError(err: unknown): string {
	const text =
		typeof err === 'string'
			? err
			: err instanceof Error
				? err.message
				: err && typeof err === 'object' && 'status' in err
					? String((err as { status?: number }).status ?? '')
					: '';
	if (/quota|429/i.test(text)) {
		return 'Monthly TTS quota exhausted. Upgrade your plan or wait until next month.';
	}
	if (/503|disabled/i.test(text)) {
		return 'TTS is not enabled on this server.';
	}
	return 'Could not start audio. Please try again.';
}

export function formatTtsResumePosition(seconds: number): string {
	const safe = Number.isFinite(seconds) && seconds > 0 ? Math.floor(seconds) : 0;
	const hours = Math.floor(safe / 3600);
	const minutes = Math.floor((safe % 3600) / 60);
	const remainingSeconds = safe % 60;
	if (hours > 0) {
		return `${hours}:${minutes.toString().padStart(2, '0')}:${remainingSeconds
			.toString()
			.padStart(2, '0')}`;
	}
	return `${minutes}:${remainingSeconds.toString().padStart(2, '0')}`;
}

export function findChunkById(
	manifest: SessionManifestResponse,
	chunkId: string | null
): TtsSessionChunk | null {
	if (!chunkId) return null;
	return manifest.chunks.find((chunk) => chunk.chunk_id === chunkId) ?? null;
}

export function chunkContainsElement(chunk: TtsSessionChunk, elementIndex: number): boolean {
	return chunk.start_element_index <= elementIndex && elementIndex <= chunk.end_element_index;
}

export function findReadyChunkForElement(
	manifest: SessionManifestResponse,
	elementIndex: number
): TtsSessionChunk | null {
	return (
		manifest.chunks.find(
			(chunk) =>
				chunk.state === 'ready' && chunk.audio_url && chunkContainsElement(chunk, elementIndex)
		) ?? null
	);
}

export function findNextReadyChunk(
	manifest: SessionManifestResponse,
	chunkId: string | null
): TtsSessionChunk | null {
	const currentChunk = findChunkById(manifest, chunkId);
	if (!currentChunk) return null;
	return (
		manifest.chunks.find(
			(chunk) =>
				chunk.state === 'ready' &&
				chunk.audio_url &&
				chunk.position > currentChunk.position &&
				chunk.position === currentChunk.position + 1
		) ?? null
	);
}

export function countForwardReadyChunks(
	manifest: SessionManifestResponse,
	chunk: TtsSessionChunk
): number {
	return manifest.chunks.filter(
		(candidate) =>
			candidate.state === 'ready' && candidate.audio_url && candidate.position > chunk.position
	).length;
}
