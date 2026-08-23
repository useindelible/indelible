import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, fireEvent, waitFor } from '@testing-library/svelte';
import { locale, setupI18nSync } from '$lib/i18n';
import en from '$lib/i18n/locales/en.json';
import fr from '$lib/i18n/locales/fr.json';

vi.mock('$app/navigation', () => ({
	goto: vi.fn()
}));

vi.mock('$app/paths', () => ({
	base: '',
	resolve: (path: string) => path
}));

vi.mock('$app/state', () => {
	const url = new URL('http://localhost/preferences/email');
	return { page: { url } };
});

vi.mock('$lib/stores/auth.svelte', () => ({
	getAuth: () => ({
		user: {
			id: 'usr_1',
			email: 'test@example.com',
			ingest_email: 'tgr8q2pf@feed.useindelible.com',
			ingest_library_email: 'tgr8q2pf@library.useindelible.com'
		}
	})
}));

const apiMocks = vi.hoisted(() => ({
	listEmailAliases: vi.fn(),
	createEmailAlias: vi.fn(),
	deleteEmailAlias: vi.fn(),
	listEmailSenders: vi.fn(),
	updateEmailSender: vi.fn(),
	unsubscribeEmailSender: vi.fn()
}));

vi.mock('$lib/api', () => apiMocks);

import EmailPage from '../src/routes/(app)/preferences/email/+page.svelte';

function aliasResponse(overrides: Partial<Record<string, unknown>> = {}) {
	return {
		object: 'email_alias',
		id: 'als_feed_1',
		local_part: 'newsletters',
		address: 'newsletters@feed.useindelible.com',
		destination: 'feed',
		status: 'active',
		is_default: false,
		retire_at: null,
		retired_at: null,
		created_at: '2026-03-01T00:00:00Z',
		...overrides
	};
}

function senderResponse(overrides: Partial<Record<string, unknown>> = {}) {
	return {
		object: 'email_sender',
		id: 'snd_stratechery',
		canonical_addr: 'notifications@stratechery.com',
		display_name: 'Ben Thompson · Stratechery',
		list_id: 'stratechery-daily.list-manage.com',
		render_default: 'reader',
		routing_default: 'feed',
		blocked: false,
		blocked_at: null,
		delivery_count: 284,
		first_seen_at: '2024-01-01T00:00:00Z',
		last_seen_at: '2026-05-18T11:48:00Z',
		...overrides
	};
}

const okResponse = { status: 200, ok: true } as Response;

describe('Email preferences page (rebuilt)', () => {
	beforeEach(() => {
		vi.clearAllMocks();
		apiMocks.listEmailAliases.mockResolvedValue({
			data: { object: 'list', data: [aliasResponse()] },
			error: null,
			response: okResponse
		});
		apiMocks.listEmailSenders.mockResolvedValue({
			data: { object: 'list', data: [senderResponse()] },
			error: null,
			response: okResponse
		});
	});

	it('renders the Postroom hero headline', async () => {
		render(EmailPage);
		await waitFor(() => {
			expect(screen.getByText(/Newsletters arrive here/i)).toBeTruthy();
		});
	});

	it('renders the hero in French', async () => {
		setupI18nSync({ en, fr }, 'fr');
		try {
			render(EmailPage);
			await waitFor(() => {
				expect(screen.getByText('Les newsletters arrivent ici.')).toBeTruthy();
			});
		} finally {
			void locale.set('en');
		}
	});

	it('renders a 4-cell stats card', async () => {
		render(EmailPage);
		await waitFor(() => {
			expect(screen.getByText('Senders', { selector: '.stat-label' })).toBeTruthy();
		});
		expect(screen.getByText('Deliveries', { selector: '.stat-label' })).toBeTruthy();
		expect(screen.getByText('Blocked', { selector: '.stat-label' })).toBeTruthy();
		expect(screen.getByText('Last delivery', { selector: '.stat-label' })).toBeTruthy();
	});

	it('renders both inbox envelope cards with their primary addresses', async () => {
		render(EmailPage);
		await waitFor(() => {
			expect(screen.getByText(/Feed inbox/i)).toBeTruthy();
		});
		expect(screen.getByText(/Library inbox/i)).toBeTruthy();
		expect(screen.getByText('tgr8q2pf@feed.useindelible.com')).toBeTruthy();
		expect(screen.getByText('tgr8q2pf@library.useindelible.com')).toBeTruthy();
	});

	it('renders the senders register section heading', async () => {
		render(EmailPage);
		await waitFor(() => {
			expect(screen.getByText(/senders/i, { selector: 'h2, h2 *' })).toBeTruthy();
		});
	});

	it('renders a row for each sender returned by the API', async () => {
		render(EmailPage);
		await waitFor(() => {
			expect(screen.getByText('Ben Thompson · Stratechery')).toBeTruthy();
		});
		expect(screen.getByText('notifications@stratechery.com')).toBeTruthy();
	});

	it('renders filter chips for All / Feed / Library / Blocked', async () => {
		render(EmailPage);
		await waitFor(() => {
			expect(screen.getByRole('tab', { name: /^All/i })).toBeTruthy();
		});
		expect(screen.getByRole('tab', { name: /^Feed/i })).toBeTruthy();
		expect(screen.getByRole('tab', { name: /^Library/i })).toBeTruthy();
		expect(screen.getByRole('tab', { name: /^Blocked/i })).toBeTruthy();
	});

	it('calls updateEmailSender when the block toggle is flipped', async () => {
		apiMocks.updateEmailSender.mockResolvedValue({
			data: senderResponse({ blocked: true, blocked_at: '2026-05-18T12:00:00Z' }),
			error: null,
			response: okResponse
		});
		render(EmailPage);
		await waitFor(() => {
			expect(screen.getByText('Ben Thompson · Stratechery')).toBeTruthy();
		});
		const toggle = screen.getByLabelText(/Block notifications@stratechery.com/i);
		await fireEvent.click(toggle);
		await waitFor(() => {
			expect(apiMocks.updateEmailSender).toHaveBeenCalledWith(
				expect.objectContaining({
					path: { id: 'snd_stratechery' },
					body: expect.objectContaining({ blocked: true })
				})
			);
		});
	});

	it('calls unsubscribeEmailSender when the Unsubscribe button is clicked', async () => {
		apiMocks.unsubscribeEmailSender.mockResolvedValue({
			data: {
				object: 'email_unsubscribe',
				sender_id: 'snd_stratechery',
				blocked_at: '2026-05-18T12:00:00Z',
				job_id: 'job_1'
			},
			error: null,
			response: okResponse
		});
		render(EmailPage);
		await waitFor(() => {
			expect(screen.getByText('Ben Thompson · Stratechery')).toBeTruthy();
		});
		const unsubBtn = screen.getByRole('button', {
			name: /Unsubscribe from Ben Thompson · Stratechery/i
		});
		await fireEvent.click(unsubBtn);
		await waitFor(() => {
			expect(apiMocks.unsubscribeEmailSender).toHaveBeenCalledWith(
				expect.objectContaining({ path: { id: 'snd_stratechery' } })
			);
		});
	});

	it('calls createEmailAlias when an alias is drafted and issued', async () => {
		apiMocks.createEmailAlias.mockResolvedValue({
			data: aliasResponse({ local_part: 'ben-essays' }),
			error: null,
			response: okResponse
		});
		render(EmailPage);
		await waitFor(() => {
			expect(screen.getByText(/Feed inbox/i)).toBeTruthy();
		});

		const draftFeed = screen.getByRole('button', { name: /Create a new feed address/i });
		await fireEvent.click(draftFeed);

		const localInput = await screen.findByPlaceholderText('local-part');
		await fireEvent.input(localInput, { target: { value: 'ben-essays' } });

		const issueBtn = screen.getByRole('button', { name: /Make it primary/i });
		await fireEvent.click(issueBtn);

		await waitFor(() => {
			expect(apiMocks.createEmailAlias).toHaveBeenCalledWith(
				expect.objectContaining({
					body: expect.objectContaining({
						local_part: 'ben-essays',
						destination: 'feed'
					})
				})
			);
		});
	});

	it('keeps the collapsed alias composer inert', async () => {
		render(EmailPage);
		await waitFor(() => {
			expect(screen.getByText(/Feed inbox/i)).toBeTruthy();
		});

		const composer = screen
			.getByPlaceholderText('local-part')
			.closest('.draft-composer')! as HTMLElement & {
			inert: boolean;
		};
		expect(composer.inert).toBe(true);
		expect(screen.queryByRole('button', { name: /Make it primary/i })).toBeNull();
	});

	it('returns focus to the address trigger after cancellation', async () => {
		render(EmailPage);
		await waitFor(() => {
			expect(screen.getByText(/Feed inbox/i)).toBeTruthy();
		});
		const trigger = screen.getByRole('button', { name: /Create a new feed address/i });

		await fireEvent.click(trigger);
		const input = screen.getByPlaceholderText('local-part');
		input.focus();
		await fireEvent.click(screen.getByRole('button', { name: 'Cancel' }));

		expect(document.activeElement).toBe(trigger);
	});
});
