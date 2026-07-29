import { describe, it, expect, vi } from 'vitest';
import { render, screen, fireEvent } from '@testing-library/svelte';
import IntegrationCallbackBanner from '$lib/components/integrations/IntegrationCallbackBanner.svelte';

describe('IntegrationCallbackBanner', () => {
	it('renders nothing when callback is null', () => {
		render(IntegrationCallbackBanner, {
			props: { callback: null, onDismiss: () => {} }
		});
		expect(screen.queryByTestId('integration-callback-banner')).toBeNull();
	});

	it('renders the success state with the provider display name', () => {
		render(IntegrationCallbackBanner, {
			props: {
				callback: { kind: 'success', provider: 'notion' },
				onDismiss: () => {}
			}
		});
		const banner = screen.getByTestId('integration-callback-banner');
		expect(banner.dataset.kind).toBe('success');
		expect(screen.getByText(/Notion connected/)).toBeTruthy();
	});

	it('shows the Notion CTA on success and fires onAction', async () => {
		const onAction = vi.fn();
		render(IntegrationCallbackBanner, {
			props: {
				callback: { kind: 'success', provider: 'notion' },
				onDismiss: () => {},
				onAction
			}
		});
		await fireEvent.click(screen.getByText('Open Notion settings'));
		expect(onAction).toHaveBeenCalledWith({
			kind: 'success',
			provider: 'notion'
		});
	});

	it('renders the denied state with cancellation copy', () => {
		render(IntegrationCallbackBanner, {
			props: {
				callback: { kind: 'denied', provider: 'notion' },
				onDismiss: () => {}
			}
		});
		expect(screen.getByText(/connection cancelled/i)).toBeTruthy();
		expect(screen.getByText(/declined/i)).toBeTruthy();
	});

	it('renders the provider_error state', () => {
		render(IntegrationCallbackBanner, {
			props: {
				callback: { kind: 'provider_error', provider: 'notion' },
				onDismiss: () => {}
			}
		});
		expect(screen.getByText(/couldn['’]t complete the connection/i)).toBeTruthy();
	});

	it('renders the server_error state', () => {
		render(IntegrationCallbackBanner, {
			props: {
				callback: { kind: 'server_error', provider: 'notion' },
				onDismiss: () => {}
			}
		});
		expect(screen.getByText(/Something went wrong/)).toBeTruthy();
	});

	it('falls back to a generic provider label when provider is unknown', () => {
		render(IntegrationCallbackBanner, {
			props: {
				callback: { kind: 'success', provider: null },
				onDismiss: () => {}
			}
		});
		expect(screen.getByText(/integration connected/i)).toBeTruthy();
	});

	it('fires onDismiss when the Dismiss button is clicked', async () => {
		const onDismiss = vi.fn();
		render(IntegrationCallbackBanner, {
			props: {
				callback: { kind: 'success', provider: 'notion' },
				onDismiss
			}
		});
		await fireEvent.click(screen.getByText('Dismiss'));
		expect(onDismiss).toHaveBeenCalled();
	});
});
