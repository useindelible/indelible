import type { SessionManifestResponse } from '$lib/api/generated/types.gen';
import type { MessageKey } from '$lib/i18n';

export type TtsStatus = 'idle' | 'loading' | 'playing' | 'paused' | 'unavailable';
export type TtsSessionChunk = SessionManifestResponse['chunks'][number];

export type TtsUnavailableBanner = {
	variant: 'setup' | 'error';
	titleKey: MessageKey;
	messageKey: MessageKey;
};

export function getTtsUnavailableBanner(messageKey: MessageKey): TtsUnavailableBanner {
	if (messageKey === 'reader_tts_no_readable_content') {
		return {
			variant: 'setup',
			titleKey: 'reader_tts_no_readable_content',
			messageKey: 'reader_tts_not_available_document'
		};
	}
	return {
		variant: 'error',
		titleKey: 'reader_tts_unavailable',
		messageKey
	};
}

export function messageForTtsError(err: unknown): MessageKey {
	const text =
		typeof err === 'string'
			? err
			: err instanceof Error
				? err.message
				: err && typeof err === 'object' && 'status' in err
					? String((err as { status?: number }).status ?? '')
					: '';
	if (/content|readable/i.test(text)) return 'reader_tts_no_readable_content';
	if (/quota|429/i.test(text)) return 'reader_tts_quota_exhausted';
	if (/503|disabled/i.test(text)) return 'reader_tts_not_enabled';
	return 'reader_tts_could_not_start';
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
