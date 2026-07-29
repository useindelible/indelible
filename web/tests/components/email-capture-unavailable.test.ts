import { render, screen } from '@testing-library/svelte';
import { describe, expect, it, vi } from 'vitest';

import EmailForwardingCard from '../../src/routes/(app)/preferences/integrations/components/EmailForwardingCard.svelte';
import EmailEnvelopeCard from '../../src/routes/(app)/preferences/email/components/EmailEnvelopeCard.svelte';

const UNAVAILABLE_COPY = /an administrator must configure an email ingest domain/i;

describe('EmailForwardingCard without a configured domain', () => {
	it('disables Copy and says the feature needs administrator configuration', () => {
		render(EmailForwardingCard, {
			props: {
				inboxAddress: '',
				feedAddress: '',
				copiedInbox: false,
				copiedFeed: false,
				onCopy: vi.fn()
			}
		});

		expect(screen.getAllByText(UNAVAILABLE_COPY).length).toBeGreaterThan(0);
		for (const button of screen.queryAllByRole('button', { name: /copy/i })) {
			expect((button as HTMLButtonElement).disabled).toBe(true);
		}
	});

	it('keeps Copy enabled when addresses exist', () => {
		render(EmailForwardingCard, {
			props: {
				inboxAddress: 'tok-lib@library.example',
				feedAddress: 'tok@feed.example',
				copiedInbox: false,
				copiedFeed: false,
				onCopy: vi.fn()
			}
		});

		expect(screen.getByText('tok-lib@library.example')).toBeTruthy();
		expect(screen.queryByText(UNAVAILABLE_COPY)).toBeNull();
		const copyButtons = screen.getAllByRole('button', { name: /copy/i });
		expect(copyButtons.length).toBe(2);
		for (const button of copyButtons) {
			expect((button as HTMLButtonElement).disabled).toBe(false);
		}
	});
});

describe('EmailEnvelopeCard without a configured domain', () => {
	const baseProps = {
		dest: 'feed' as const,
		label: 'Feed inbox · Triage',
		headline: 'For newsletters.',
		primary: null,
		copied: false,
		onCopy: vi.fn(),
		onOpenComposer: vi.fn()
	};

	it('shows an explicit unavailable state instead of a fake issuing state', () => {
		render(EmailEnvelopeCard, { props: { ...baseProps, address: '' } });

		expect(screen.queryByText(/issuing/i)).toBeNull();
		expect(screen.getByText(UNAVAILABLE_COPY)).toBeTruthy();
		expect(screen.queryByRole('button', { name: /new .*address/i })).toBeNull();
	});

	it('keeps the address, copy and new-address affordances when configured', () => {
		render(EmailEnvelopeCard, { props: { ...baseProps, address: 'tok@feed.example' } });

		expect(screen.getByText('tok@feed.example')).toBeTruthy();
		expect(screen.queryByText(UNAVAILABLE_COPY)).toBeNull();
		expect(
			(screen.getByRole('button', { name: /copy feed address/i }) as HTMLButtonElement).disabled
		).toBe(false);
		expect(screen.getByRole('button', { name: /create a new feed address/i })).toBeTruthy();
	});
});
