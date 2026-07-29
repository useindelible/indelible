import { describe, expect, it } from 'vitest';
import type { DownloadObsidianArtifactResponses } from '$lib/api/generated';

describe('generated API contracts', () => {
	it('types Obsidian ZIP artifact downloads as binary blobs', () => {
		const artifact: DownloadObsidianArtifactResponses[200] = new Blob(['zip']);
		expect(artifact).toBeInstanceOf(Blob);
	});
});
