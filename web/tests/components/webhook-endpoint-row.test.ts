import { describe, expect, it, vi } from 'vitest';
import { fireEvent, render, screen } from '@testing-library/svelte';
import type { WebhookDelivery, WebhookEndpoint } from '$lib/api/webhooks';
import WebhookEndpointRow from '../../src/routes/(app)/preferences/developer/components/WebhookEndpointRow.svelte';

function endpoint(overrides: Partial<WebhookEndpoint> = {}): WebhookEndpoint {
	return {
		id: 'wh_1',
		name: 'Automation',
		url: 'https://example.com/hook',
		events: ['library_entry.saved', 'feed.poll_failed'],
		is_active: true,
		last_status: 'healthy',
		delivery_history: ['s2xx', 's4xx', 'pending'],
		secret_preview: 'whsec_abc...',
		created_at: '2026-06-10T12:00:00Z',
		updated_at: '2026-06-10T12:00:00Z',
		...overrides
	};
}

function delivery(overrides: Partial<WebhookDelivery> = {}): WebhookDelivery {
	return {
		id: 'del_1',
		endpoint_id: 'wh_1',
		event: 'library_entry.saved',
		target: 'Automation',
		status_code: 200,
		latency_ms: 42,
		attempt: 1,
		delivered_at: '2026-06-10T13:59:00Z',
		...overrides
	};
}

function props(overrides = {}) {
	return {
		endpoint: endpoint(),
		expanded: true,
		deliveries: [delivery()],
		testEvent: 'library_entry.saved',
		onToggleExpanded: vi.fn(),
		onRotateSecret: vi.fn(),
		onSendTest: vi.fn(),
		onToggleActive: vi.fn(),
		onDelete: vi.fn(),
		onSetTestEvent: vi.fn(),
		...overrides
	};
}

describe('WebhookEndpointRow', () => {
	it('renders endpoint summary and expanded delivery details', () => {
		render(WebhookEndpointRow, { props: props() });
		expect(screen.getByText('https://example.com/hook')).toBeTruthy();
		expect(screen.getAllByText('Automation').length).toBeGreaterThan(0);
		expect(screen.getByText('Healthy')).toBeTruthy();
		expect(screen.getAllByText('library_entry.saved').length).toBeGreaterThan(0);
		expect(screen.getByText('42ms')).toBeTruthy();
	});

	it('uses callback props for endpoint actions', async () => {
		const rowProps = props();
		render(WebhookEndpointRow, { props: rowProps });

		await fireEvent.click(screen.getByRole('button', { name: /https:\/\/example.com\/hook/i }));
		await fireEvent.change(screen.getByLabelText('Test event'), {
			target: { value: 'feed.poll_failed' }
		});
		await fireEvent.click(screen.getByRole('button', { name: 'Send test' }));
		await fireEvent.click(screen.getByRole('button', { name: /Rotate/i }));
		await fireEvent.click(screen.getByRole('button', { name: 'Toggle active' }));
		await fireEvent.click(screen.getByRole('button', { name: 'Delete' }));

		expect(rowProps.onToggleExpanded).toHaveBeenCalledWith('wh_1');
		expect(rowProps.onSetTestEvent).toHaveBeenCalledWith('wh_1', 'feed.poll_failed');
		expect(rowProps.onSendTest).toHaveBeenCalledWith('wh_1');
		expect(rowProps.onRotateSecret).toHaveBeenCalledWith('wh_1');
		expect(rowProps.onToggleActive).toHaveBeenCalledWith('wh_1', false);
		expect(rowProps.onDelete).toHaveBeenCalledWith('wh_1');
	});
});
