import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, waitFor } from '@testing-library/svelte';
import { createApiModuleMock } from '../../helpers/api-module-mock';

vi.mock('$lib/api', () => createApiModuleMock());

import { api } from '$lib/api';
import OAuthButtons from '$lib/components/auth/OAuthButtons.svelte';

const mockGET = vi.mocked(api.GET);

describe('OAuthButtons', () => {
	beforeEach(() => {
		vi.clearAllMocks();
	});

	it('shows nothing when no providers are configured', async () => {
		mockGET.mockResolvedValue({ data: { providers: [] } });

		const { container } = render(OAuthButtons);

		await waitFor(() => {
			expect(container.querySelector('.oauth-buttons')).toBeNull();
		});
	});

	it('renders buttons when providers exist', async () => {
		mockGET.mockResolvedValue({
			data: {
				providers: [
					{ id: 'google', name: 'Google', enabled: true },
					{ id: 'apple', name: 'Apple', enabled: true },
					{ id: 'oidc', name: 'authentik', enabled: true }
				]
			}
		});

		render(OAuthButtons);

		await waitFor(() => {
			expect(screen.getByText('Continue with Google')).toBeTruthy();
			expect(screen.getByText('Continue with Apple')).toBeTruthy();
			expect(screen.getByText('Continue with authentik')).toBeTruthy();
		});
	});

	it('renders divider when providers exist', async () => {
		mockGET.mockResolvedValue({
			data: {
				providers: [{ id: 'google', name: 'Google', enabled: true }]
			}
		});

		render(OAuthButtons);

		await waitFor(() => {
			expect(screen.getByText('or')).toBeTruthy();
		});
	});

	it('does not render divider when no providers', async () => {
		mockGET.mockResolvedValue({ data: { providers: [] } });

		render(OAuthButtons);

		await waitFor(() => {
			expect(screen.queryByText('or')).toBeNull();
		});
	});

	it('uses custom divider text', async () => {
		mockGET.mockResolvedValue({
			data: {
				providers: [{ id: 'google', name: 'Google', enabled: true }]
			}
		});

		render(OAuthButtons, { props: { dividerText: 'or sign up with' } });

		await waitFor(() => {
			expect(screen.getByText('or sign up with')).toBeTruthy();
		});
	});

	it('handles API failure gracefully', async () => {
		mockGET.mockRejectedValue(new Error('Network error'));

		const { container } = render(OAuthButtons);

		await waitFor(() => {
			expect(container.querySelector('.oauth-buttons')).toBeNull();
		});
	});

	it('fetches providers on mount', async () => {
		mockGET.mockResolvedValue({ data: { providers: [] } });

		render(OAuthButtons);

		await waitFor(() => {
			expect(mockGET).toHaveBeenCalledWith('/api/v1/auth/providers');
		});
	});
});
