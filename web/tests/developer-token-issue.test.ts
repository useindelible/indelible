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
	await fireEvent.click(form.getByRole('button', { name: 'Select all' }));
	await fireEvent.change(form.getByLabelText(/Token auto-revokes/), {
		target: { value: '365' }
	});
}

function expectDefaultTokenDraft() {
	const form = tokenIssueForm();
	expect((form.getByPlaceholderText('e.g. Personal MacBook') as HTMLInputElement).value).toBe(
		'Personal MacBook'
	);
	expect(form.getByText('0 selected')).toBeTruthy();
	expect((form.getByLabelText(/Token auto-revokes/) as HTMLSelectElement).value).toBe('90');
	expect(form.queryByRole('alert')).toBeNull();
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

	it('initializes the Obsidian deep link with only Obsidian sync', async () => {
		await renderDeveloperPage('http://localhost/preferences/developer?permission=obsidian%3Async');
		const form = tokenIssueForm();

		expect((form.getByPlaceholderText('e.g. Personal MacBook') as HTMLInputElement).value).toBe(
			'Obsidian plugin'
		);
		expect(form.getByText('1 selected')).toBeTruthy();
		expect(form.getByRole('button', { name: /Obsidian sync/ }).getAttribute('aria-pressed')).toBe(
			'true'
		);
		expect(form.getByRole('button', { name: /AI use/ }).getAttribute('aria-pressed')).toBe('false');
	});
});
