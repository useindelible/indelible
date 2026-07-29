import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, fireEvent, waitFor } from '@testing-library/svelte';
import { createApiModuleMock } from '../helpers/api-module-mock';

const mockGoto = vi.fn();

vi.mock('$app/navigation', () => ({
	goto: (...args: unknown[]) => mockGoto(...args)
}));

vi.mock('$app/paths', () => ({
	base: '',
	resolve: (path: string) => path
}));

vi.mock('$app/stores', () => {
	const url = new URL('http://localhost/register');
	return {
		page: {
			subscribe: (fn: (value: { url: URL }) => void) => {
				fn({ url });
				return () => {};
			}
		}
	};
});

const mockRegister = vi.fn();
const mockLogin = vi.fn();

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

import RegisterPage from '../../src/routes/(auth)/register/+page.svelte';
import { api } from '$lib/api';
import { resetInstanceStatusCache } from '$lib/api/instance';

async function renderRegisterPageWithForm() {
	render(RegisterPage);
	await waitFor(() => {
		expect(screen.getByLabelText('Display name')).toBeTruthy();
	});
}

describe('Register page', () => {
	beforeEach(() => {
		vi.clearAllMocks();
		resetInstanceStatusCache();
		mockAuthError = null;
		mockNeedsVerification = false;
	});

	it('renders all form fields', async () => {
		await renderRegisterPageWithForm();

		expect(screen.getByLabelText('Display name')).toBeTruthy();
		expect(screen.getByLabelText('Email')).toBeTruthy();
		expect(screen.getByLabelText('Password')).toBeTruthy();
	});

	it('renders Create Account button', async () => {
		await renderRegisterPageWithForm();

		expect(screen.getByRole('button', { name: /create account/i })).toBeTruthy();
	});

	it('renders sign in link', () => {
		render(RegisterPage);

		const link = screen.getByText('Sign in');
		expect(link).toBeTruthy();
		expect(link.getAttribute('href')).toBe('/login');
	});

	it('shows validation errors on empty submit', async () => {
		await renderRegisterPageWithForm();

		await fireEvent.click(screen.getByRole('button', { name: /create account/i }));

		expect(screen.getByText('Display name is required')).toBeTruthy();
		expect(screen.getByText('Email is required')).toBeTruthy();
		expect(screen.getByText('Password is required')).toBeTruthy();
		expect(mockRegister).not.toHaveBeenCalled();
	});

	it('validates password minimum length', async () => {
		await renderRegisterPageWithForm();

		await fireEvent.input(screen.getByLabelText('Display name'), {
			target: { value: 'Test' }
		});
		await fireEvent.input(screen.getByLabelText('Email'), {
			target: { value: 'test@example.com' }
		});
		await fireEvent.input(screen.getByLabelText('Password'), {
			target: { value: 'short' }
		});

		await fireEvent.click(screen.getByRole('button', { name: /create account/i }));

		expect(screen.getByText('Password must be at least 8 characters')).toBeTruthy();
		expect(mockRegister).not.toHaveBeenCalled();
	});

	it('validates email format', async () => {
		await renderRegisterPageWithForm();

		await fireEvent.input(screen.getByLabelText('Display name'), {
			target: { value: 'Test' }
		});
		await fireEvent.input(screen.getByLabelText('Email'), {
			target: { value: 'notanemail' }
		});
		await fireEvent.input(screen.getByLabelText('Password'), {
			target: { value: 'password123' }
		});

		await fireEvent.click(screen.getByRole('button', { name: /create account/i }));

		expect(screen.getByText('Please enter a valid email address')).toBeTruthy();
		expect(mockRegister).not.toHaveBeenCalled();
	});

	it('calls auth.register on valid submit', async () => {
		mockRegister.mockResolvedValue({ success: true });
		await renderRegisterPageWithForm();

		await fireEvent.input(screen.getByLabelText('Display name'), {
			target: { value: 'Test User' }
		});
		await fireEvent.input(screen.getByLabelText('Email'), {
			target: { value: 'test@example.com' }
		});
		await fireEvent.input(screen.getByLabelText('Password'), {
			target: { value: 'password123' }
		});

		await fireEvent.click(screen.getByRole('button', { name: /create account/i }));

		await waitFor(() => {
			expect(mockRegister).toHaveBeenCalledWith('test@example.com', 'password123', 'Test User');
		});
	});

	it('redirects to verify-email when needsVerification', async () => {
		mockRegister.mockImplementation(async () => {
			mockNeedsVerification = true;
			return { success: true };
		});
		await renderRegisterPageWithForm();

		await fireEvent.input(screen.getByLabelText('Display name'), {
			target: { value: 'Test' }
		});
		await fireEvent.input(screen.getByLabelText('Email'), {
			target: { value: 'test@example.com' }
		});
		await fireEvent.input(screen.getByLabelText('Password'), {
			target: { value: 'password123' }
		});

		await fireEvent.click(screen.getByRole('button', { name: /create account/i }));

		await waitFor(() => {
			expect(mockGoto).toHaveBeenCalledWith('/verify-email');
		});
	});

	it('redirects to onboarding when no verification needed', async () => {
		mockRegister.mockImplementation(async () => {
			mockNeedsVerification = false;
			return { success: true };
		});
		await renderRegisterPageWithForm();

		await fireEvent.input(screen.getByLabelText('Display name'), {
			target: { value: 'Test' }
		});
		await fireEvent.input(screen.getByLabelText('Email'), {
			target: { value: 'test@example.com' }
		});
		await fireEvent.input(screen.getByLabelText('Password'), {
			target: { value: 'password123' }
		});

		await fireEvent.click(screen.getByRole('button', { name: /create account/i }));

		await waitFor(() => {
			expect(mockGoto).toHaveBeenCalledWith('/onboarding/welcome');
		});
	});

	it('shows server error for duplicate email', async () => {
		mockRegister.mockResolvedValue({ success: false });
		mockAuthError = 'An account with this email already exists';
		await renderRegisterPageWithForm();

		await fireEvent.input(screen.getByLabelText('Display name'), {
			target: { value: 'Test' }
		});
		await fireEvent.input(screen.getByLabelText('Email'), {
			target: { value: 'taken@example.com' }
		});
		await fireEvent.input(screen.getByLabelText('Password'), {
			target: { value: 'password123' }
		});

		await fireEvent.click(screen.getByRole('button', { name: /create account/i }));

		await waitFor(() => {
			expect(mockRegister).toHaveBeenCalled();
		});
	});

	it('renders page title', async () => {
		await renderRegisterPageWithForm();

		expect(screen.getByText('Create your account')).toBeTruthy();
	});

	it('hides the signup form when signups are disabled', async () => {
		vi.mocked(api.GET).mockResolvedValueOnce({
			data: { providers: [], signups_enabled: false, setup_required: false }
		} as never);
		render(RegisterPage);

		await waitFor(() => {
			expect(screen.queryByLabelText('Display name')).toBeNull();
		});
		expect(screen.queryByRole('button', { name: /create account/i })).toBeNull();
	});

	it('shows first-run setup copy when setup is required', async () => {
		vi.mocked(api.GET).mockResolvedValueOnce({
			data: { providers: [], signups_enabled: true, setup_required: true }
		} as never);
		render(RegisterPage);

		await waitFor(() => {
			expect(screen.getByText('Set up Indelible')).toBeTruthy();
		});
		expect(screen.getByLabelText('Display name')).toBeTruthy();
	});

	it('validates display name max length', async () => {
		await renderRegisterPageWithForm();

		const longName = 'a'.repeat(101);
		await fireEvent.input(screen.getByLabelText('Display name'), {
			target: { value: longName }
		});
		await fireEvent.input(screen.getByLabelText('Email'), {
			target: { value: 'test@example.com' }
		});
		await fireEvent.input(screen.getByLabelText('Password'), {
			target: { value: 'password123' }
		});

		await fireEvent.click(screen.getByRole('button', { name: /create account/i }));

		expect(screen.getByText('Display name must be 100 characters or fewer')).toBeTruthy();
		expect(mockRegister).not.toHaveBeenCalled();
	});
});
