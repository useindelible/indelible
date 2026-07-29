import { describe, it, expect, vi } from 'vitest';
import { render, screen, fireEvent } from '@testing-library/svelte';
import ImportRollbackDialog from '$lib/components/imports/ImportRollbackDialog.svelte';

describe('ImportRollbackDialog', () => {
	it('does not render when closed', () => {
		render(ImportRollbackDialog, {
			props: { open: false, onConfirm: () => {}, onCancel: () => {} }
		});
		expect(screen.queryByRole('dialog')).toBeNull();
	});

	it('renders the dialog when open', () => {
		render(ImportRollbackDialog, {
			props: { open: true, onConfirm: () => {}, onCancel: () => {} }
		});
		expect(screen.getByRole('dialog')).toBeTruthy();
		expect(screen.getByText('Roll back import?')).toBeTruthy();
	});

	it('calls onConfirm when the destructive button is clicked', async () => {
		const onConfirm = vi.fn();
		render(ImportRollbackDialog, {
			props: { open: true, onConfirm, onCancel: () => {} }
		});
		await fireEvent.click(screen.getByText('Roll back'));
		expect(onConfirm).toHaveBeenCalled();
	});

	it('calls onCancel when the cancel button is clicked', async () => {
		const onCancel = vi.fn();
		render(ImportRollbackDialog, {
			props: { open: true, onConfirm: () => {}, onCancel }
		});
		await fireEvent.click(screen.getByText('Cancel'));
		expect(onCancel).toHaveBeenCalled();
	});

	it('renders the error message when provided', () => {
		render(ImportRollbackDialog, {
			props: {
				open: true,
				errorMessage: 'rollback failed',
				onConfirm: () => {},
				onCancel: () => {}
			}
		});
		expect(screen.getByText('rollback failed')).toBeTruthy();
	});

	it('renders the 30-day Settings → Data copy', () => {
		render(ImportRollbackDialog, {
			props: {
				open: true,
				busy: false,
				errorMessage: null,
				onCancel: () => {},
				onConfirm: () => {}
			}
		});
		expect(screen.getByText(/recovered within 30 days from/)).toBeTruthy();
		expect(screen.getByText(/Settings → Data/)).toBeTruthy();
		expect(screen.getByText(/cannot be undone after 30 days/)).toBeTruthy();
	});

	it('disables Cancel while busy', () => {
		render(ImportRollbackDialog, {
			props: { open: true, busy: true, onConfirm: () => {}, onCancel: () => {} }
		});
		const cancel = screen.getByText('Cancel').closest('button');
		expect(cancel?.hasAttribute('disabled')).toBe(true);
	});
});
