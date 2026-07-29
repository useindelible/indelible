import { describe, it, expect, beforeEach, vi } from 'vitest';
import { render, screen, fireEvent, waitFor } from '@testing-library/svelte';
import { createApiModuleMock } from './helpers/api-module-mock';

const mockUser = {
	id: 'usr_01924b6e-5c3a-7d4f-8e6b-a1b2c3d4e5f6',
	email: 'test@example.com',
	display_name: 'Test User',
	avatar_url: null,
	email_verified: false,
	onboarding_completed: false,
	has_password: true,
	theme: 'system' as const
};

function mockSessionResponse(user = mockUser) {
	return {
		user,
		session: {
			id: 'ses_01924b6e-5c3a-7d4f-8e6b-a1b2c3d4e5f6',
			client_type: 'web',
			last_active_at: '2026-03-21T00:00:00Z',
			expires_at: '2026-04-20T00:00:00Z',
			created_at: '2026-03-21T00:00:00Z'
		}
	};
}

vi.mock('$lib/api', () => createApiModuleMock());

vi.mock('$app/paths', () => ({
	resolve: (path: string) => path,
	base: ''
}));

vi.mock('$app/navigation', () => ({
	goto: vi.fn()
}));

vi.mock('$app/stores', async () => {
	const { readable } = await import('svelte/store');
	return {
		page: readable({
			url: new URL('http://localhost/verify-email'),
			params: {},
			route: { id: '' },
			status: 200,
			error: null,
			data: {},
			form: null
		})
	};
});

import { api } from '$lib/api';
import { goto } from '$app/navigation';
import { getAuth } from '$lib/stores/auth.svelte';
import VerifyEmail from '../src/routes/(auth)/verify-email/+page.svelte';

const mockPOST = vi.mocked(api.POST);
const mockGET = vi.mocked(api.GET);
const mockGoto = vi.mocked(goto);

describe('VerifyEmail page: pending state (no token)', () => {
	beforeEach(async () => {
		vi.clearAllMocks();
		mockPOST.mockResolvedValueOnce({
			data: {
				...mockUser,
				access_token: 'test-token',
				expires_at: 9999999999
			},
			error: undefined,
			response: new Response(null, { status: 200 })
		} as never);
		const auth = getAuth();
		await auth.login('test@example.com', 'password');
	});

	it('shows check your email message', () => {
		render(VerifyEmail, { props: { data: { token: null } } });

		expect(screen.getByText('Check your email')).toBeTruthy();
	});

	it('displays user email address', () => {
		render(VerifyEmail, { props: { data: { token: null } } });

		expect(screen.getByText('test@example.com')).toBeTruthy();
	});

	it('shows resend email button', () => {
		render(VerifyEmail, { props: { data: { token: null } } });

		expect(screen.getByRole('button', { name: 'Resend email' })).toBeTruthy();
	});

	it('starts countdown after successful resend', async () => {
		render(VerifyEmail, { props: { data: { token: null } } });

		mockPOST.mockResolvedValue({
			data: { message: 'Verification email sent.' },
			error: undefined,
			response: new Response(null, { status: 200 })
		} as never);

		const resendButton = screen.getByRole('button', { name: 'Resend email' });
		await fireEvent.click(resendButton);

		await waitFor(() => {
			expect(screen.getByText('Verification email sent!')).toBeTruthy();
			expect(screen.getByRole('button', { name: /Resend email \(\d+s\)/ })).toBeTruthy();
		});
	});

	it('disables resend button during countdown', async () => {
		render(VerifyEmail, { props: { data: { token: null } } });

		mockPOST.mockResolvedValue({
			data: { message: 'Verification email sent.' },
			error: undefined,
			response: new Response(null, { status: 200 })
		} as never);

		const resendButton = screen.getByRole('button', { name: 'Resend email' });
		await fireEvent.click(resendButton);

		await waitFor(() => {
			const button = screen.getByRole('button', { name: /Resend email/ });
			expect(button.hasAttribute('disabled')).toBe(true);
		});
	});
});

describe('VerifyEmail page: verification with token', () => {
	beforeEach(async () => {
		vi.clearAllMocks();
		mockPOST.mockResolvedValueOnce({
			data: {
				...mockUser,
				access_token: 'test-token',
				expires_at: 9999999999
			},
			error: undefined,
			response: new Response(null, { status: 200 })
		} as never);
		const auth = getAuth();
		await auth.login('test@example.com', 'password');
	});

	it('shows loading spinner when token is present', () => {
		mockPOST.mockImplementation(() => new Promise(() => {}));

		render(VerifyEmail, { props: { data: { token: 'test-token' } } });

		expect(screen.getByText('Verifying your email\u2026')).toBeTruthy();
	});

	it('shows success state after valid token verification', async () => {
		const verifiedUser = { ...mockUser, email_verified: true };
		mockPOST.mockResolvedValue({
			data: verifiedUser,
			error: undefined,
			response: new Response(null, { status: 200 })
		} as never);

		render(VerifyEmail, { props: { data: { token: 'valid-token' } } });

		await waitFor(() => {
			expect(screen.getByText('Email verified!')).toBeTruthy();
			expect(screen.getByRole('button', { name: 'Continue to Indelible' })).toBeTruthy();
		});
	});

	it('shows error state on invalid token', async () => {
		mockPOST.mockResolvedValue({
			data: undefined,
			error: { error: 'Invalid or expired token' },
			response: new Response(null, { status: 400 })
		} as never);

		render(VerifyEmail, { props: { data: { token: 'bad-token' } } });

		await waitFor(() => {
			expect(screen.getByText('Verification failed')).toBeTruthy();
			expect(
				screen.getByText('This link has expired or is invalid. Request a new one below.')
			).toBeTruthy();
		});
	});

	it('continue button navigates to home for already-onboarded users', async () => {
		const onboardedUser = { ...mockUser, email_verified: true, onboarding_completed: true };
		mockGET.mockResolvedValue({
			data: mockSessionResponse(onboardedUser),
			error: undefined,
			response: new Response(null, { status: 200 })
		} as never);
		const auth = getAuth();
		await auth.initialize();

		mockPOST.mockResolvedValue({
			data: onboardedUser,
			error: undefined,
			response: new Response(null, { status: 200 })
		} as never);

		render(VerifyEmail, { props: { data: { token: 'valid-token' } } });

		await waitFor(() => {
			expect(screen.getByRole('button', { name: 'Continue to Indelible' })).toBeTruthy();
		});

		await fireEvent.click(screen.getByRole('button', { name: 'Continue to Indelible' }));

		expect(mockGoto).toHaveBeenCalledWith('/');
	});

	it('continue button navigates to onboarding for new users', async () => {
		const verifiedNotOnboardedUser = {
			...mockUser,
			email_verified: true,
			onboarding_completed: false
		};
		mockPOST.mockResolvedValue({
			data: verifiedNotOnboardedUser,
			error: undefined,
			response: new Response(null, { status: 200 })
		} as never);

		render(VerifyEmail, { props: { data: { token: 'valid-token' } } });

		await waitFor(() => {
			expect(screen.getByRole('button', { name: 'Continue to Indelible' })).toBeTruthy();
		});

		await fireEvent.click(screen.getByRole('button', { name: 'Continue to Indelible' }));

		expect(mockGoto).toHaveBeenCalledWith('/onboarding/welcome');
	});
});
