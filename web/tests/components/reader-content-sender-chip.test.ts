import { fireEvent, render, screen } from '@testing-library/svelte';
import { describe, expect, it, vi } from 'vitest';
import type { EmbeddedSenderResponse } from '$lib/api/generated/types.gen';
import ReaderContent from '$lib/components/reader/ReaderContent.svelte';

vi.mock('$lib/stores/reader-preferences.svelte', () => ({
	getReaderPreferences: () => ({
		theme: 'system',
		fontFamily: 'var(--font-sans)',
		fontSize: 18,
		lineHeight: 1.75,
		contentWidth: 760,
		paragraphSpacing: 1.2,
		textAlign: 'left'
	})
}));

function makeSender(overrides: Partial<EmbeddedSenderResponse> = {}): EmbeddedSenderResponse {
	return {
		id: 'snd_test123',
		canonical_addr: 'stratechery@stratechery.com',
		display_name: 'Stratechery',
		list_id: null,
		blocked: false,
		...overrides
	};
}

function baseProps() {
	return {
		htmlContent: '<p>hello world</p>',
		title: 'Test Article',
		author: 'Ben Thompson',
		domain: 'stratechery.com',
		publishedAt: null,
		readingTimeMinutes: 4,
		onScroll: () => {},
		initialProgress: 0
	};
}

describe('ReaderContent sender chip', () => {
	it('renders no sender chip when no sender is present', () => {
		render(ReaderContent, { props: { ...baseProps(), sender: null } });
		expect(screen.queryByText(/Block sender/)).toBeNull();
		expect(screen.queryByText(/Sender blocked/)).toBeNull();
	});

	it('renders Block sender chip when sender is present and not blocked', () => {
		const onSenderBlockToggle = vi.fn(async () => {});
		render(ReaderContent, {
			props: { ...baseProps(), sender: makeSender(), onSenderBlockToggle }
		});
		expect(screen.getByText('Block sender')).toBeTruthy();
	});

	it('renders Sender blocked · Unblock chip when sender.blocked is true', () => {
		const onSenderBlockToggle = vi.fn(async () => {});
		render(ReaderContent, {
			props: {
				...baseProps(),
				sender: makeSender({ blocked: true }),
				onSenderBlockToggle
			}
		});
		expect(screen.getByText(/Sender blocked/)).toBeTruthy();
	});

	it('calls onSenderBlockToggle with blocked=true when chip is clicked from unblocked state', async () => {
		const onSenderBlockToggle = vi.fn(async () => {});
		const sender = makeSender();
		render(ReaderContent, {
			props: { ...baseProps(), sender, onSenderBlockToggle }
		});
		await fireEvent.click(screen.getByText('Block sender'));
		expect(onSenderBlockToggle).toHaveBeenCalledWith(sender, true);
	});

	it('calls onSenderBlockToggle with blocked=false when chip is clicked from blocked state', async () => {
		const onSenderBlockToggle = vi.fn(async () => {});
		const sender = makeSender({ blocked: true });
		render(ReaderContent, {
			props: { ...baseProps(), sender, onSenderBlockToggle }
		});
		const chip = screen.getByText(/Sender blocked/).closest('button');
		if (!chip) throw new Error('chip not found');
		await fireEvent.click(chip);
		expect(onSenderBlockToggle).toHaveBeenCalledWith(sender, false);
	});

	it('omits chip when onSenderBlockToggle is not provided even if sender exists', () => {
		render(ReaderContent, {
			props: { ...baseProps(), sender: makeSender() }
		});
		expect(screen.queryByText('Block sender')).toBeNull();
	});
});
