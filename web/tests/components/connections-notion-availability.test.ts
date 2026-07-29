import { render, screen } from '@testing-library/svelte';
import { describe, expect, it, vi } from 'vitest';

import ConnectionsSection from '../../src/routes/(app)/preferences/integrations/components/ConnectionsSection.svelte';

function sectionProps(overrides: Record<string, unknown> = {}) {
	return {
		connectionsLoading: false,
		connectionsError: null,
		inboxAddress: 'tok-lib@library.example',
		feedAddress: 'tok@feed.example',
		copiedInbox: false,
		copiedFeed: false,
		extStore: { label: 'Chrome Web Store', href: 'https://example.com' },
		notionConnection: undefined,
		obsidianConnection: undefined,
		notionStatus: { label: 'Not connected', variant: 'coming' as const },
		obsidianStatus: { label: 'Not connected', variant: 'coming' as const },
		syncStateByConnection: {},
		syncErrorByConnection: {},
		notionConnectError: null,
		notionAvailable: true,
		onCopyAddress: vi.fn(),
		onStartNotion: vi.fn(),
		onOpenNotion: vi.fn(),
		onOpenObsidian: vi.fn(),
		onSync: vi.fn(),
		onDisconnect: vi.fn(),
		...overrides
	};
}

describe('Notion connect availability gating', () => {
	it('disables Connect and explains the missing server configuration', () => {
		render(ConnectionsSection, { props: sectionProps({ notionAvailable: false }) });

		const connect = screen.getByRole('button', {
			name: 'Connect Notion'
		}) as HTMLButtonElement;
		expect(connect.disabled).toBe(true);
		expect(screen.getByText(/not configured on this server/i)).toBeTruthy();
		// Every prerequisite, including the key that seals the returned tokens.
		expect(screen.getByText(/NOTION_CLIENT_ID/)).toBeTruthy();
		expect(screen.getByText(/AUTH_CREDENTIAL_KEY/)).toBeTruthy();
	});

	it('keeps Connect enabled when the server holds Notion credentials', () => {
		render(ConnectionsSection, { props: sectionProps({ notionAvailable: true }) });

		const connect = screen.getByRole('button', {
			name: 'Connect Notion'
		}) as HTMLButtonElement;
		expect(connect.disabled).toBe(false);
		expect(screen.queryByText(/not configured on this server/i)).toBeNull();
	});
});
