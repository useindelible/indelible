import { describe, expect, it, vi } from 'vitest';
import { fireEvent, render, screen } from '@testing-library/svelte';
import DeleteAccountDialog from '../../src/routes/(app)/preferences/account/components/DeleteAccountDialog.svelte';

function renderDialog(overrides = {}) {
	const onClose = vi.fn();
	const onConfirmEmailChange = vi.fn();
	const onDelete = vi.fn();
	render(DeleteAccountDialog, {
		props: {
			email: 'user@example.com',
			confirmEmail: '',
			deleteEmailMatches: false,
			deleting: false,
			error: '',
			onClose,
			onConfirmEmailChange,
			onDelete,
			...overrides
		}
	});
	return { onClose, onConfirmEmailChange, onDelete };
}

describe('DeleteAccountDialog', () => {
	it('renders destructive account deletion copy and disables delete until confirmed', () => {
		renderDialog();
		expect(screen.getByRole('dialog', { name: 'Delete your account' })).toBeTruthy();
		expect(screen.getByText('user@example.com')).toBeTruthy();
		expect(screen.getByRole<HTMLButtonElement>('button', { name: 'Delete forever' }).disabled).toBe(
			true
		);
	});

	it('emits typed email changes and delete/cancel callbacks', async () => {
		const handlers = renderDialog({ confirmEmail: 'user@example.com', deleteEmailMatches: true });
		await fireEvent.input(screen.getByLabelText('Type your email to confirm'), {
			target: { value: 'user@example.com' }
		});
		await fireEvent.click(screen.getByRole('button', { name: 'Delete forever' }));
		await fireEvent.click(screen.getByRole('button', { name: 'Cancel' }));

		expect(handlers.onConfirmEmailChange).toHaveBeenCalledWith('user@example.com');
		expect(handlers.onDelete).toHaveBeenCalledOnce();
		expect(handlers.onClose).toHaveBeenCalledOnce();
	});

	it('shows errors and busy copy', () => {
		renderDialog({
			confirmEmail: 'user@example.com',
			deleteEmailMatches: true,
			deleting: true,
			error: 'Account deletion failed'
		});
		expect(screen.getByText('Account deletion failed')).toBeTruthy();
		expect(screen.getByRole<HTMLButtonElement>('button', { name: 'Deleting…' }).disabled).toBe(
			true
		);
	});
});
