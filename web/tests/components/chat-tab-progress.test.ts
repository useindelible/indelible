import { fireEvent, render, screen } from '@testing-library/svelte';
import { describe, expect, it, vi } from 'vitest';

const cancel = vi.fn();
const progress = vi.hoisted(() => ({ phase: 'preparing', elapsedSeconds: 12 }));

vi.mock('$app/paths', () => ({ resolve: (path: string) => path }));
vi.mock('$lib/stores/mila.svelte', () => ({
	getMilaConfig: () => ({ configured: true, loaded: true, loading: false, load: vi.fn() }),
	createMilaChat: () => ({
		messages: [
			{
				id: 'msg_user',
				role: 'user',
				content: 'What is the central argument?',
				source_refs: [],
				streaming: false
			},
			{
				id: 'msg_assistant',
				role: 'assistant',
				content: '',
				source_refs: [],
				streaming: true
			}
		],
		streaming: true,
		get phase() {
			return progress.phase;
		},
		get elapsedSeconds() {
			return progress.elapsedSeconds;
		},
		loading: false,
		error: null,
		retrievalWarning: null,
		initialize: vi.fn(),
		destroy: vi.fn(),
		sendMessage: vi.fn(),
		retry: vi.fn(),
		cancel
	})
}));

import ChatTab from '../../src/lib/components/library/ChatTab.svelte';

describe('ChatTab response progress', () => {
	it('replaces the blank assistant response with phase, elapsed time and Cancel', async () => {
		progress.phase = 'preparing';
		progress.elapsedSeconds = 12;
		render(ChatTab, {
			scope: { type: 'single_document', documentId: 'doc_1' },
			label: 'Runtime evidence'
		});

		expect(screen.getByRole('status').textContent).toContain('Preparing response');
		expect(screen.getByRole('status').textContent).toContain('12s');
		await fireEvent.click(screen.getByRole('button', { name: 'Cancel response' }));
		expect(cancel).toHaveBeenCalledOnce();
	});

	it('explains a prolonged pre-delta wait without claiming provider state as fact', () => {
		progress.phase = 'preparing';
		progress.elapsedSeconds = 45;
		render(ChatTab, {
			scope: { type: 'single_document', documentId: 'doc_1' },
			label: 'Runtime evidence'
		});

		expect(screen.getByRole('status').textContent).toContain(
			'Still preparing — the provider may be starting'
		);
	});
});
