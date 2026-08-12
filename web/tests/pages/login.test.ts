import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, fireEvent, waitFor } from '@testing-library/svelte';
import { createApiModuleMock } from '../helpers/api-module-mock';

const mockGoto = vi.fn();
const pageMock = vi.hoisted(() => ({
	currentUrl: new URL('http://localhost/login')
}));

vi.mock('$app/navigation', () => ({
	goto: (...args: unknown[]) => mockGoto(...args)
}));

vi.mock('$app/paths', () => ({
	base: '',
	resolve: (path: string) => path
}));

vi.mock('$app/stores', () => {
	return {
		page: {
			subscribe: (fn: (value: { url: URL }) => void) => {
				fn({ url: pageMock.currentUrl });
				return () => {};
			}
		}
	};
});

const mockLogin = vi.fn();
const mockRegister = vi.fn();

let mockAuthError: string | null = null;
let mockNeedsVerification = false;

vi.mock('$lib/stores/auth.svelte', () => ({
	getAuth: () => ({
		get error() {
			return mockAuthError;
		},
		get isAuthenticated() {
			return false;
		},
		get needsVerification() {
			return mockNeedsVerification;
		},
		get needsOnboarding() {
			return false;
		},
		login: mockLogin,
		register: mockRegister
	})
}));

vi.mock('$lib/api', () => {
	const module = createApiModuleMock();
	module.api.GET.mockResolvedValue({
		data: { providers: [], signups_enabled: true, setup_required: false }
	});
	return module;
});

import LoginPage from '../../src/routes/(auth)/login/+page.svelte';
import { api } from '$lib/api';
import { resetInstanceStatusCache } from '$lib/api/instance';

describe('Login page', () => {
	beforeEach(() => {
		vi.clearAllMocks();
		resetInstanceStatusCache();
		mockAuthError = null;
		mockNeedsVerification = false;
		pageMock.currentUrl = new URL('http://localhost/login');
	});

	it('renders email and password fields', () => {
		render(LoginPage);

		expect(screen.getByLabelText('Email')).toBeTruthy();
		expect(screen.getByLabelText('Password')).toBeTruthy();
	});

	it('renders Sign In button', () => {
		render(LoginPage);

		expect(screen.getByRole('button', { name: /sign in/i })).toBeTruthy();
	});

	it('renders forgot password link', () => {
		render(LoginPage);

		const link = screen.getByText('Forgot password?');
		expect(link).toBeTruthy();
		expect(link.getAttribute('href')).toBe('/forgot-password');
	});

	it('renders register link', async () => {
		render(LoginPage);

		const link = await screen.findByText('Sign up');
		expect(link).toBeTruthy();
		expect(link.getAttribute('href')).toBe('/register');
	});

	it('hides the register link when signups are disabled', async () => {
		vi.mocked(api.GET).mockResolvedValueOnce({
			data: { providers: [], signups_enabled: false, setup_required: false }
		} as never);
		render(LoginPage);

		await waitFor(() => {
			expect(screen.queryByText('Sign up')).toBeNull();
		});
	});

	it('shows validation errors on empty submit', async () => {
		render(LoginPage);

		const button = screen.getByRole('button', { name: /sign in/i });
		await fireEvent.click(button);

		expect(screen.getByText('Email is required')).toBeTruthy();
		expect(screen.getByText('Password is required')).toBeTruthy();
		expect(mockLogin).not.toHaveBeenCalled();
	});

	it('calls auth.login on valid submit', async () => {
		mockLogin.mockResolvedValue({ success: true });
		render(LoginPage);

		const emailInput = screen.getByLabelText('Email');
		const passwordInput = screen.getByLabelText('Password');

		await fireEvent.input(emailInput, { target: { value: 'test@example.com' } });
		await fireEvent.input(passwordInput, { target: { value: 'password123' } });

		const button = screen.getByRole('button', { name: /sign in/i });
		await fireEvent.click(button);

		await waitFor(() => {
			expect(mockLogin).toHaveBeenCalledWith('test@example.com', 'password123');
		});
	});

	it('redirects to home on successful login', async () => {
		mockLogin.mockResolvedValue({ success: true });
		render(LoginPage);

		const emailInput = screen.getByLabelText('Email');
		const passwordInput = screen.getByLabelText('Password');

		await fireEvent.input(emailInput, { target: { value: 'test@example.com' } });
		await fireEvent.input(passwordInput, { target: { value: 'password123' } });
		await fireEvent.click(screen.getByRole('button', { name: /sign in/i }));

		await waitFor(() => {
			expect(mockGoto).toHaveBeenCalledWith('/');
		});
	});

	it('redirects to a same-origin absolute path after successful login', async () => {
		pageMock.currentUrl = new URL(
			'http://localhost/login?redirect=%2Freader%2Fdoc_book%3Ftab%3Dnotes'
		);
		mockLogin.mockResolvedValue({ success: true });
		render(LoginPage);

		await fireEvent.input(screen.getByLabelText('Email'), {
			target: { value: 'test@example.com' }
		});
		await fireEvent.input(screen.getByLabelText('Password'), {
			target: { value: 'password123' }
		});
		await fireEvent.click(screen.getByRole('button', { name: /sign in/i }));

		await waitFor(() => {
			expect(mockGoto).toHaveBeenCalledWith('/reader/doc_book?tab=notes');
		});
	});

	it('falls back home when the redirect param is external', async () => {
		pageMock.currentUrl = new URL(
			'http://localhost/login?redirect=https%3A%2F%2Fevil.example%2Fsteal'
		);
		mockLogin.mockResolvedValue({ success: true });
		render(LoginPage);

		await fireEvent.input(screen.getByLabelText('Email'), {
			target: { value: 'test@example.com' }
		});
		await fireEvent.input(screen.getByLabelText('Password'), {
			target: { value: 'password123' }
		});
		await fireEvent.click(screen.getByRole('button', { name: /sign in/i }));

		await waitFor(() => {
			expect(mockGoto).toHaveBeenCalledWith('/');
		});
	});

	it('falls back home when the redirect param is protocol-relative', async () => {
		pageMock.currentUrl = new URL('http://localhost/login?redirect=%2F%2Fevil.example%2Fsteal');
		mockLogin.mockResolvedValue({ success: true });
		render(LoginPage);

		await fireEvent.input(screen.getByLabelText('Email'), {
			target: { value: 'test@example.com' }
		});
		await fireEvent.input(screen.getByLabelText('Password'), {
			target: { value: 'password123' }
		});
		await fireEvent.click(screen.getByRole('button', { name: /sign in/i }));

		await waitFor(() => {
			expect(mockGoto).toHaveBeenCalledWith('/');
		});
	});

	it('shows error on failed login', async () => {
		mockLogin.mockResolvedValue({ success: false });
		mockAuthError = 'Email or password is incorrect.';
		render(LoginPage);

		const emailInput = screen.getByLabelText('Email');
		const passwordInput = screen.getByLabelText('Password');

		await fireEvent.input(emailInput, { target: { value: 'test@example.com' } });
		await fireEvent.input(passwordInput, { target: { value: 'wrong' } });
		await fireEvent.click(screen.getByRole('button', { name: /sign in/i }));

		await waitFor(() => {
			expect(mockLogin).toHaveBeenCalled();
		});
		expect(screen.getByRole('alert').textContent).toBe('Email or password is incorrect.');
	});

	it('shows cooldown after 5 failed attempts', async () => {
		mockLogin.mockResolvedValue({ success: false });
		render(LoginPage);

		const emailInput = screen.getByLabelText('Email');
		const passwordInput = screen.getByLabelText('Password');

		await fireEvent.input(emailInput, { target: { value: 'test@example.com' } });
		await fireEvent.input(passwordInput, { target: { value: 'wrong' } });

		for (let i = 0; i < 5; i++) {
			await fireEvent.click(screen.getByRole('button', { name: /sign in/i }));
			await waitFor(() => {
				expect(mockLogin).toHaveBeenCalledTimes(i + 1);
			});
		}

		await waitFor(() => {
			expect(screen.getByText(/too many attempts/i)).toBeTruthy();
		});
	});

	it('renders page title', () => {
		render(LoginPage);

		expect(screen.getByText('Welcome back')).toBeTruthy();
	});

	it('does not call login when form has empty email', async () => {
		render(LoginPage);

		const passwordInput = screen.getByLabelText('Password');
		await fireEvent.input(passwordInput, { target: { value: 'password123' } });
		await fireEvent.click(screen.getByRole('button', { name: /sign in/i }));

		expect(mockLogin).not.toHaveBeenCalled();
	});
});
