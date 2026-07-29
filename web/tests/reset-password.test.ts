import { describe, it, expect, beforeEach, vi } from 'vitest';
import { render, screen, fireEvent, waitFor } from '@testing-library/svelte';
import { createApiModuleMock } from './helpers/api-module-mock';

vi.mock('$lib/api', () => createApiModuleMock());

vi.mock('$app/paths', () => ({
	resolve: (path: string) => path,
	base: ''
}));

import { api } from '$lib/api';
import { getAuth } from '$lib/stores/auth.svelte';
import ResetPassword from '../src/routes/(auth)/reset-password/+page.svelte';

const mockPOST = vi.mocked(api.POST);
const mockGET = vi.mocked(api.GET);

describe('ResetPassword page', () => {
	beforeEach(async () => {
		vi.clearAllMocks();
		mockGET.mockResolvedValue({
			data: undefined,
			error: { error: 'Unauthorized' },
			response: new Response(null, { status: 401 })
		} as never);
		const auth = getAuth();
		await auth.initialize();
	});

	it('renders password fields when token is present', () => {
		render(ResetPassword, { props: { data: { token: 'valid-token' } } });

		expect(screen.getByText('Set new password')).toBeTruthy();
		expect(screen.getByLabelText('New password')).toBeTruthy();
		expect(screen.getByLabelText('Confirm password')).toBeTruthy();
		expect(screen.getByRole('button', { name: 'Reset password' })).toBeTruthy();
	});

	it('shows error when no token is present', () => {
		render(ResetPassword, { props: { data: { token: null } } });

		expect(screen.getByText('Invalid reset link')).toBeTruthy();
		expect(
			screen.getByText('This password reset link is missing a token. Please request a new one.')
		).toBeTruthy();
		expect(screen.getByText('Request a new link')).toBeTruthy();
	});

	it('validates password length minimum', async () => {
		render(ResetPassword, { props: { data: { token: 'valid-token' } } });

		const passwordInput = screen.getByLabelText('New password');
		const confirmInput = screen.getByLabelText('Confirm password');

		await fireEvent.input(passwordInput, { target: { value: 'short' } });
		await fireEvent.input(confirmInput, { target: { value: 'short' } });
		await fireEvent.submit(screen.getByRole('button', { name: 'Reset password' }).closest('form')!);

		await waitFor(() => {
			expect(screen.getByText('Password must be at least 8 characters.')).toBeTruthy();
		});
	});

	it('validates password match', async () => {
		render(ResetPassword, { props: { data: { token: 'valid-token' } } });

		const passwordInput = screen.getByLabelText('New password');
		const confirmInput = screen.getByLabelText('Confirm password');

		await fireEvent.input(passwordInput, { target: { value: 'password123' } });
		await fireEvent.input(confirmInput, { target: { value: 'password456' } });
		await fireEvent.submit(screen.getByRole('button', { name: 'Reset password' }).closest('form')!);

		await waitFor(() => {
			expect(screen.getByText('Passwords do not match.')).toBeTruthy();
		});
	});

	it('shows success message after valid reset', async () => {
		render(ResetPassword, { props: { data: { token: 'valid-token' } } });

		mockPOST.mockResolvedValue({
			data: {
				id: 'usr_01924b6e-5c3a-7d4f-8e6b-a1b2c3d4e5f6',
				object: 'user',
				email: 'test@example.com',
				display_name: 'Test User',
				email_verified: true,
				onboarding_completed: true
			},
			error: undefined,
			response: new Response(null, { status: 200 })
		} as never);

		const passwordInput = screen.getByLabelText('New password');
		const confirmInput = screen.getByLabelText('Confirm password');

		await fireEvent.input(passwordInput, { target: { value: 'newpassword123' } });
		await fireEvent.input(confirmInput, { target: { value: 'newpassword123' } });
		await fireEvent.submit(screen.getByRole('button', { name: 'Reset password' }).closest('form')!);

		await waitFor(() => {
			expect(screen.getByText('Password updated')).toBeTruthy();
			expect(screen.getByText('Your password has been reset successfully.')).toBeTruthy();
			expect(screen.getByText('Sign in')).toBeTruthy();
		});
	});

	it('shows expired token error', async () => {
		render(ResetPassword, { props: { data: { token: 'expired-token' } } });

		mockPOST.mockResolvedValue({
			data: undefined,
			error: { error: 'Token expired' },
			response: new Response(null, { status: 400 })
		} as never);

		const passwordInput = screen.getByLabelText('New password');
		const confirmInput = screen.getByLabelText('Confirm password');

		await fireEvent.input(passwordInput, { target: { value: 'newpassword123' } });
		await fireEvent.input(confirmInput, { target: { value: 'newpassword123' } });
		await fireEvent.submit(screen.getByRole('button', { name: 'Reset password' }).closest('form')!);

		await waitFor(() => {
			expect(screen.getByText('Link expired')).toBeTruthy();
			expect(screen.getByText('This password reset link has expired or is invalid.')).toBeTruthy();
		});
	});
});
