import { beforeEach, describe, expect, it, vi } from 'vitest';
import { fireEvent, render, screen, waitFor, within } from '@testing-library/svelte';

const pageState = vi.hoisted(() => ({
	url: new URL('http://localhost/preferences/developer')
}));

const tokenApi = vi.hoisted(() => ({
	createApiToken: vi.fn(),
	loadApiTokens: vi.fn(),
	revokeApiToken: vi.fn()
}));

const webhookApi = vi.hoisted(() => ({
	createWebhookEndpoint: vi.fn(),
	deleteWebhookEndpoint: vi.fn(),
	listWebhookDeliveries: vi.fn(),
	listWebhookEndpoints: vi.fn(),
	rotateWebhookSecret: vi.fn(),
	testWebhookEndpoint: vi.fn(),
	updateWebhookEndpoint: vi.fn()
}));

vi.mock('$app/state', () => ({ page: pageState }));
vi.mock('$lib/api/tokens', () => tokenApi);
vi.mock('$lib/api/webhooks', async (importOriginal) => ({
	...(await importOriginal<typeof import('$lib/api/webhooks')>()),
	...webhookApi
}));

import DeveloperPage from '../src/routes/(app)/preferences/developer/+page.svelte';

async function renderDeveloperPage(url = 'http://localhost/preferences/developer') {
	pageState.url = new URL(url);
	render(DeveloperPage);
	await waitFor(() => expect(tokenApi.loadApiTokens).toHaveBeenCalledOnce());
}

function tokenIssueForm() {
	return within(screen.getByPlaceholderText('e.g. Personal MacBook').closest('.issue-form')!);
}

async function dirtyTokenDraft() {
	await fireEvent.click(screen.getByRole('button', { name: 'Issue token' }));
	const form = tokenIssueForm();
	await fireEvent.click(form.getByRole('button', { name: 'Create token' }));
	expect(form.getByRole('alert').textContent).toBe('Pick at least one permission.');
	await fireEvent.input(form.getByPlaceholderText('e.g. Personal MacBook'), {
		target: { value: 'Broad access' }
	});
	await fireEvent.click(form.getByRole('button', { name: 'Grant everything' }));
	await fireEvent.change(form.getByLabelText(/Token auto-revokes/), {
		target: { value: '365' }
	});
}

function expectDefaultTokenDraft() {
	const form = tokenIssueForm();
	expect((form.getByPlaceholderText('e.g. Personal MacBook') as HTMLInputElement).value).toBe(
		'Personal MacBook'
	);
	expect(form.getByText('No permissions granted')).toBeTruthy();
	expect((form.getByLabelText(/Token auto-revokes/) as HTMLSelectElement).value).toBe('90');
	expect(form.queryByRole('alert')).toBeNull();
}

function token(id: string, expiresAt: string | null) {
	return {
		id,
		object: 'api_token',
		name: `Token ${id}`,
		prefix: `ind_${id}`,
		permissions: ['library:read'],
		created_at: '2026-08-01T00:00:00Z',
		last_used_at: null,
		expires_at: expiresAt
	};
}

function endpoint(id: string) {
	return {
		id,
		name: `Endpoint ${id}`,
		url: `https://example.com/${id}`,
		events: ['library_entry.saved'],
		is_active: true,
		last_status: 'healthy',
		delivery_history: [],
		secret_preview: 'whsec_abc...',
		created_at: '2026-08-01T00:00:00Z',
		updated_at: '2026-08-01T00:00:00Z'
	};
}

describe('developer token issue form', () => {
	beforeEach(() => {
		vi.clearAllMocks();
		tokenApi.loadApiTokens.mockResolvedValue({ success: true, data: [] });
		webhookApi.listWebhookEndpoints.mockResolvedValue([]);
	});

	it.each([
		['Cancel', () => tokenIssueForm().getByRole('button', { name: 'Cancel' })],
		['close icon', () => tokenIssueForm().getByRole('button', { name: 'Close' })]
	])('resets a cancelled draft before an ordinary reopen via %s', async (_label, closeButton) => {
		await renderDeveloperPage();
		await dirtyTokenDraft();

		await fireEvent.click(closeButton());
		await fireEvent.click(screen.getByRole('button', { name: 'Issue token' }));

		expectDefaultTokenDraft();
	});

	it('keeps collapsed forms out of the accessibility tree', async () => {
		await renderDeveloperPage();
		const tokenForm = screen
			.getByPlaceholderText('e.g. Personal MacBook')
			.closest('.issue-form')! as HTMLElement & { inert: boolean };
		const webhookForm = screen
			.getByPlaceholderText('e.g. Internal Slack inbox')
			.closest('.add-form')! as HTMLElement & { inert: boolean };

		expect(tokenForm.inert).toBe(true);
		expect(webhookForm.inert).toBe(true);
		expect(screen.queryByRole('button', { name: 'Create token' })).toBeNull();
		expect(screen.queryByRole('button', { name: 'Create endpoint' })).toBeNull();
	});

	it('returns focus to the token trigger after cancellation', async () => {
		await renderDeveloperPage();
		const trigger = screen.getByRole('button', { name: 'Issue token' });

		await fireEvent.click(trigger);
		const form = tokenIssueForm();
		(form.getByPlaceholderText('e.g. Personal MacBook') as HTMLInputElement).focus();
		await fireEvent.click(form.getByRole('button', { name: 'Cancel' }));

		expect(document.activeElement).toBe(trigger);
	});

	it('returns focus to the webhook trigger after cancellation', async () => {
		await renderDeveloperPage();
		const trigger = screen.getByRole('button', { name: 'Add endpoint' });

		await fireEvent.click(trigger);
		const input = screen.getByPlaceholderText('e.g. Internal Slack inbox');
		const form = within(input.closest('.add-form')!);
		input.focus();
		await fireEvent.click(form.getByRole('button', { name: 'Cancel' }));

		expect(document.activeElement).toBe(trigger);
	});

	it('initializes the Obsidian deep link with only Obsidian sync', async () => {
		await renderDeveloperPage('http://localhost/preferences/developer?permission=obsidian%3Async');
		const form = tokenIssueForm();

		expect((form.getByPlaceholderText('e.g. Personal MacBook') as HTMLInputElement).value).toBe(
			'Obsidian plugin'
		);
		expect(form.getByText(/permission granted/).textContent).toContain('1');
		expect(form.getByRole('button', { name: /Obsidian sync/ }).getAttribute('aria-pressed')).toBe(
			'true'
		);
		expect(form.getByRole('button', { name: /AI use/ }).getAttribute('aria-pressed')).toBe('false');
	});

	it('renders the granted permissions as the array the request will carry', async () => {
		await renderDeveloperPage();
		await fireEvent.click(screen.getByRole('button', { name: 'Issue token' }));
		const form = tokenIssueForm();

		expect(form.getByText(/Nothing granted yet/)).toBeTruthy();

		// Read + write on Library must surface library:read alongside library:write.
		const libraryLevels = within(form.getByRole('radiogroup', { name: 'Library access' }));
		await fireEvent.click(libraryLevels.getByRole('radio', { name: 'Read + write' }));
		expect([...form.getAllByText(/^library:/)].map((chip) => chip.textContent)).toEqual([
			'library:read',
			'library:write'
		]);

		// ai:write implies ai:read, and the AI read row says so rather than silently checking.
		await fireEvent.click(form.getByRole('button', { name: /AI configure/ }));
		expect(form.getByText('ai:read', { selector: '.permission' })).toBeTruthy();
		expect(form.getByRole('button', { name: /AI read/ }).className).toContain('implied');
	});

	it('resolves the expiry choice to a revocation date', async () => {
		await renderDeveloperPage();
		await fireEvent.click(screen.getByRole('button', { name: 'Issue token' }));
		const form = tokenIssueForm();

		expect(form.getByText(/^Revokes/)).toBeTruthy();

		await fireEvent.change(form.getByLabelText(/Token auto-revokes/), {
			target: { value: 'never' }
		});
		expect(form.getByText('Stays valid until you revoke it.')).toBeTruthy();
	});

	it('shows only authoritative hero counts and no fabricated activity', async () => {
		tokenApi.loadApiTokens.mockResolvedValue({
			success: true,
			data: [token('current', '2027-08-01T00:00:00Z'), token('expired', '2026-01-01T00:00:00Z')]
		});
		webhookApi.listWebhookEndpoints.mockResolvedValue([
			endpoint('one'),
			endpoint('two'),
			endpoint('three')
		]);
		await renderDeveloperPage();

		const hero = screen.getByText('Developer · Tokens & Webhooks').closest('.hero')!;
		const tokenStat = within(hero).getByText('Tokens').closest('.hero-stat')!;
		const endpointStat = within(hero).getByText('Endpoints').closest('.hero-stat')!;
		expect(within(tokenStat).getByText('2', { selector: '.num' })).toBeTruthy();
		expect(within(endpointStat).getByText('3', { selector: '.num' })).toBeTruthy();
		expect(within(hero).queryByText('Events · 24h')).toBeNull();
		expect(within(hero).queryByText('Delivery')).toBeNull();
		expect(within(hero).queryByText('activity.log')).toBeNull();
		expect(within(hero).queryByText('Live')).toBeNull();
		expect(within(hero).queryByText('tail -f /var/log/indelible/api')).toBeNull();
		expect(hero.textContent).not.toContain('/v1/items');
		expect(hero.textContent).not.toContain('/v1/highlights');
	});
});
