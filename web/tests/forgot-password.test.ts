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
import ForgotPassword from '../src/routes/(auth)/forgot-password/+page.svelte';

const mockPOST = vi.mocked(api.POST);
const mockGET = vi.mocked(api.GET);

describe('ForgotPassword page', () => {
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

	it('renders email input and submit button', () => {
		render(ForgotPassword);

		expect(screen.getByText('Reset your password')).toBeTruthy();
		expect(screen.getByLabelText('Email')).toBeTruthy();
		expect(screen.getByRole('button', { name: 'Send reset link' })).toBeTruthy();
	});

	it('renders back to login link', () => {
		render(ForgotPassword);

		const link = screen.getByText('Back to sign in');
		expect(link).toBeTruthy();
		expect(link.getAttribute('href')).toBe('/login');
	});

	it('shows success message after submit', async () => {
		render(ForgotPassword);

		mockPOST.mockResolvedValue({
			data: { message: 'If an account exists, a reset email has been sent.' },
			error: undefined,
			response: new Response(null, { status: 200 })
		} as never);

		const emailInput = screen.getByLabelText('Email');
		await fireEvent.input(emailInput, { target: { value: 'test@example.com' } });
		await fireEvent.submit(
			screen.getByRole('button', { name: 'Send reset link' }).closest('form')!
		);

		await waitFor(() => {
			expect(screen.getByText('Check your email')).toBeTruthy();
			expect(
				screen.getByText("If an account exists for that address, we've sent a password reset link.")
			).toBeTruthy();
		});
	});

	it('shows success regardless of email existence', async () => {
		render(ForgotPassword);

		mockPOST.mockResolvedValue({
			data: { message: 'If an account exists, a reset email has been sent.' },
			error: undefined,
			response: new Response(null, { status: 200 })
		} as never);

		const emailInput = screen.getByLabelText('Email');
		await fireEvent.input(emailInput, { target: { value: 'nonexistent@example.com' } });
		await fireEvent.submit(
			screen.getByRole('button', { name: 'Send reset link' }).closest('form')!
		);

		await waitFor(() => {
			expect(screen.getByText('Check your email')).toBeTruthy();
		});
	});

	it('shows rate limit error on 429 response', async () => {
		render(ForgotPassword);

		mockPOST.mockResolvedValue({
			data: undefined,
			error: { error: 'Too many requests' },
			response: new Response(null, { status: 429 })
		} as never);

		const emailInput = screen.getByLabelText('Email');
		await fireEvent.input(emailInput, { target: { value: 'test@example.com' } });
		await fireEvent.submit(
			screen.getByRole('button', { name: 'Send reset link' }).closest('form')!
		);

		await waitFor(() => {
			expect(screen.getByText('Too many requests. Please try again later.')).toBeTruthy();
		});
	});
});
