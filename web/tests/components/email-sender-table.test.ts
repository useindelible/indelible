import { describe, expect, it, vi } from 'vitest';
import { fireEvent, render, screen } from '@testing-library/svelte';
import type { EmailSenderResponse } from '$lib/api';
import EmailSenderTable from '../../src/routes/(app)/preferences/email/components/EmailSenderTable.svelte';

function sender(overrides: Partial<EmailSenderResponse> = {}): EmailSenderResponse {
	return {
		object: 'email_sender',
		id: 'snd_1',
		canonical_addr: 'news@example.com',
		display_name: 'Daily News',
		list_id: 'daily-news.example.com',
		render_default: 'reader',
		routing_default: 'feed',
		blocked: false,
		blocked_at: null,
		delivery_count: 12,
		first_seen_at: '2026-01-01T00:00:00Z',
		last_seen_at: '2026-06-10T13:50:00Z',
		...overrides
	};
}

describe('EmailSenderTable', () => {
	it('renders sender rows and action callbacks', async () => {
		const onRenderChange = vi.fn();
		const onRoutingChange = vi.fn();
		const onToggleBlock = vi.fn();
		const onUnsubscribe = vi.fn();

		render(EmailSenderTable, {
			props: {
				senders: [sender()],
				totalSenders: 1,
				updatingSender: null,
				unsubscribingSender: null,
				onRenderChange,
				onRoutingChange,
				onToggleBlock,
				onUnsubscribe
			}
		});

		expect(screen.getByText('Daily News')).toBeTruthy();
		expect(screen.getByText('news@example.com')).toBeTruthy();

		await fireEvent.change(screen.getByLabelText('Render mode for news@example.com'), {
			target: { value: 'original' }
		});
		await fireEvent.change(screen.getByLabelText('Routing for news@example.com'), {
			target: { value: 'library' }
		});
		await fireEvent.click(screen.getByLabelText('Block news@example.com'));
		await fireEvent.click(screen.getByRole('button', { name: /unsubscribe from daily news/i }));

		expect(onRenderChange).toHaveBeenCalledWith(
			expect.objectContaining({ id: 'snd_1' }),
			'original'
		);
		expect(onRoutingChange).toHaveBeenCalledWith(
			expect.objectContaining({ id: 'snd_1' }),
			'library'
		);
		expect(onToggleBlock).toHaveBeenCalledWith(expect.objectContaining({ id: 'snd_1' }));
		expect(onUnsubscribe).toHaveBeenCalledWith(expect.objectContaining({ id: 'snd_1' }));
	});

	it('renders the empty state', () => {
		render(EmailSenderTable, {
			props: {
				senders: [],
				totalSenders: 4,
				updatingSender: null,
				unsubscribingSender: null,
				onRenderChange: vi.fn(),
				onRoutingChange: vi.fn(),
				onToggleBlock: vi.fn(),
				onUnsubscribe: vi.fn()
			}
		});
		expect(screen.getByText('No senders match this filter.')).toBeTruthy();
	});
});
