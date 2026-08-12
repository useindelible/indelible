import { render, screen, waitFor } from '@testing-library/svelte';
import { describe, expect, it, vi } from 'vitest';

const chatHarness = vi.hoisted(() => ({
	created: [] as Array<{
		scope: { type: string; documentId?: string; collectionId?: string };
		destroy: ReturnType<typeof vi.fn>;
	}>
}));

vi.mock('$app/paths', () => ({ resolve: (path: string) => path }));
vi.mock('$lib/stores/mila.svelte', () => ({
	getMilaConfig: () => ({ configured: true, loaded: true, loading: false, load: vi.fn() }),
	createMilaChat: vi.fn((scope: { type: string; documentId?: string; collectionId?: string }) => {
		const destroy = vi.fn();
		const hasPendingQuestion = chatHarness.created.length === 1;
		chatHarness.created.push({ scope, destroy });
		return {
			messages: hasPendingQuestion
				? [
						{
							id: 'msg_1',
							role: 'user',
							content: 'What is the central argument?',
							source_refs: [],
							streaming: false
						}
					]
				: [],
			streaming: hasPendingQuestion,
			loading: false,
			error: null,
			retrievalWarning: null,
			initialize: vi.fn(),
			destroy,
			sendMessage: vi.fn(),
			retry: vi.fn()
		};
	})
}));

import ChatTab from '../../src/lib/components/library/ChatTab.svelte';

describe('ChatTab scope lifecycle', () => {
	it('keeps a pending chat for an equivalent scope and replaces it when the scope changes', async () => {
		chatHarness.created.length = 0;
		const rendered = render(ChatTab, {
			scope: { type: 'single_document', documentId: 'doc_1' },
			label: 'Original article'
		});
		await waitFor(() => expect(chatHarness.created).toHaveLength(2));
		const activeChat = chatHarness.created[1];
		expect(screen.getByText('What is the central argument?')).toBeTruthy();

		await rendered.rerender({
			scope: { type: 'single_document', documentId: 'doc_1' },
			label: 'Refetched article'
		});
		expect(activeChat.destroy).not.toHaveBeenCalled();
		expect(screen.getByText('What is the central argument?')).toBeTruthy();

		await rendered.rerender({
			scope: { type: 'single_document', documentId: 'doc_2' },
			label: 'Another article'
		});
		await waitFor(() => expect(activeChat.destroy).toHaveBeenCalledOnce());
		const secondDocumentChat = chatHarness.created[2];

		await rendered.rerender({
			scope: { type: 'collection', collectionId: 'col_1' },
			label: 'Research'
		});
		await waitFor(() => expect(secondDocumentChat.destroy).toHaveBeenCalledOnce());
	});
});
