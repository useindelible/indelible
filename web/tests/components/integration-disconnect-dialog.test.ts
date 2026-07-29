import { describe, it, expect, vi } from 'vitest';
import { render, screen, fireEvent } from '@testing-library/svelte';
import IntegrationDisconnectDialog from '$lib/components/integrations/IntegrationDisconnectDialog.svelte';

describe('IntegrationDisconnectDialog', () => {
	it('does not render when closed', () => {
		render(IntegrationDisconnectDialog, {
			props: {
				open: false,
				providerName: 'Notion',
				onConfirm: () => {},
				onCancel: () => {}
			}
		});
		expect(screen.queryByRole('dialog')).toBeNull();
	});

	it('renders the provider name in the title', () => {
		render(IntegrationDisconnectDialog, {
			props: {
				open: true,
				providerName: 'Notion',
				onConfirm: () => {},
				onCancel: () => {}
			}
		});
		expect(screen.getByRole('dialog')).toBeTruthy();
		expect(screen.getByText('Disconnect Notion?')).toBeTruthy();
	});

	it('fires onConfirm when the Disconnect button is clicked', async () => {
		const onConfirm = vi.fn();
		render(IntegrationDisconnectDialog, {
			props: {
				open: true,
				providerName: 'Notion',
				onConfirm,
				onCancel: () => {}
			}
		});
		await fireEvent.click(screen.getByText('Disconnect'));
		expect(onConfirm).toHaveBeenCalled();
	});

	it('fires onCancel when the Cancel button is clicked', async () => {
		const onCancel = vi.fn();
		render(IntegrationDisconnectDialog, {
			props: {
				open: true,
				providerName: 'Notion',
				onConfirm: () => {},
				onCancel
			}
		});
		await fireEvent.click(screen.getByText('Cancel'));
		expect(onCancel).toHaveBeenCalled();
	});

	it('disables both actions while busy', () => {
		render(IntegrationDisconnectDialog, {
			props: {
				open: true,
				providerName: 'Notion',
				busy: true,
				onConfirm: () => {},
				onCancel: () => {}
			}
		});
		const cancel = screen.getByText('Cancel').closest('button');
		expect(cancel?.hasAttribute('disabled')).toBe(true);
	});

	it('does not call onCancel on backdrop click while busy', async () => {
		const onCancel = vi.fn();
		render(IntegrationDisconnectDialog, {
			props: {
				open: true,
				providerName: 'Notion',
				busy: true,
				onConfirm: () => {},
				onCancel
			}
		});
		const backdrop = screen.getByTestId('disconnect-dialog');
		await fireEvent.click(backdrop);
		expect(onCancel).not.toHaveBeenCalled();
	});

	it('renders the error message when provided', () => {
		render(IntegrationDisconnectDialog, {
			props: {
				open: true,
				providerName: 'Notion',
				errorMessage: 'something went wrong',
				onConfirm: () => {},
				onCancel: () => {}
			}
		});
		expect(screen.getByText('something went wrong')).toBeTruthy();
	});
});
