import { afterEach, describe, expect, it, vi } from 'vitest';
import { tick } from 'svelte';
import { fireEvent, render, screen } from '@testing-library/svelte';
import ConfirmDialog from '$lib/components/ui/ConfirmDialog.svelte';
import { locale, setupI18nSync } from '$lib/i18n';
import fr from '$lib/i18n/locales/fr.json';

function renderDialog(overrides = {}) {
	const onConfirm = vi.fn();
	const onCancel = vi.fn();
	render(ConfirmDialog, {
		props: {
			open: true,
			title: 'Delete feed?',
			message: 'This removes the subscription but keeps saved stories.',
			confirmLabel: 'Delete',
			cancelLabel: 'Keep feed',
			onConfirm,
			onCancel,
			...overrides
		}
	});
	return { onConfirm, onCancel };
}

describe('ConfirmDialog', () => {
	afterEach(() => locale.set('en'));

	it('does not render when closed', () => {
		renderDialog({ open: false });
		expect(screen.queryByRole('dialog')).toBeNull();
	});

	it('renders title, body, and action labels', () => {
		renderDialog();
		expect(screen.getByRole('dialog', { name: 'Delete feed?' })).toBeTruthy();
		expect(screen.getByText('This removes the subscription but keeps saved stories.')).toBeTruthy();
		expect(screen.getByRole('button', { name: 'Delete' })).toBeTruthy();
		expect(screen.getByRole('button', { name: 'Keep feed' })).toBeTruthy();
	});

	it('uses localized default action labels', () => {
		setupI18nSync({ fr }, 'fr');
		renderDialog({ confirmLabel: undefined, cancelLabel: undefined });

		expect(screen.getByRole('button', { name: 'Confirmer' })).toBeTruthy();
		expect(screen.getByRole('button', { name: 'Annuler' })).toBeTruthy();
	});

	it('calls the callback props for confirm and cancel actions', async () => {
		const { onConfirm, onCancel } = renderDialog();
		await fireEvent.click(screen.getByRole('button', { name: 'Delete' }));
		await fireEvent.click(screen.getByRole('button', { name: 'Keep feed' }));
		expect(onConfirm).toHaveBeenCalledTimes(1);
		expect(onCancel).toHaveBeenCalledTimes(1);
	});

	it('cancels on backdrop click and Escape when not busy', async () => {
		const { onCancel } = renderDialog();
		const backdrop = screen.getByTestId('confirm-dialog-backdrop');
		await fireEvent.click(backdrop);
		await fireEvent.keyDown(backdrop, { key: 'Escape' });
		expect(onCancel).toHaveBeenCalledTimes(2);
	});

	it('focuses the dialog so Escape works immediately after open', async () => {
		const { onCancel } = renderDialog();
		await tick();

		const dialog = screen.getByRole('dialog', { name: 'Delete feed?' });
		expect(document.activeElement).toBe(dialog);

		await fireEvent.keyDown(dialog, { key: 'Escape' });
		expect(onCancel).toHaveBeenCalledOnce();
	});

	it('keeps actions disabled while busy', async () => {
		const { onConfirm, onCancel } = renderDialog({ busy: true });
		await fireEvent.click(screen.getByRole('button', { name: 'Delete' }));
		await fireEvent.click(screen.getByRole('button', { name: 'Keep feed' }));
		expect(onConfirm).not.toHaveBeenCalled();
		expect(onCancel).not.toHaveBeenCalled();
	});

	it('renders an error message when provided', () => {
		renderDialog({ errorMessage: 'Could not delete that feed.' });
		expect(screen.getByRole('alert').textContent).toContain('Could not delete that feed.');
	});
});
