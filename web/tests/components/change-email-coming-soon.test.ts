import { render, screen } from '@testing-library/svelte';
import { describe, expect, it, vi } from 'vitest';

import EmailVerificationSection from '../../src/routes/(app)/preferences/account/components/EmailVerificationSection.svelte';

function props(overrides: Record<string, unknown> = {}) {
	return {
		email: 'reader@example.com',
		emailVerified: true,
		emailRevealOpen: false,
		newEmail: '',
		currentPassword: '',
		onOpen: vi.fn(),
		onCancel: vi.fn(),
		onNewEmailChange: vi.fn(),
		onCurrentPasswordChange: vi.fn(),
		...overrides
	};
}

describe('EmailVerificationSection', () => {
	it('marks changing the address as coming soon', () => {
		render(EmailVerificationSection, props());

		expect(screen.getByText(/Coming soon/i)).toBeTruthy();
	});

	it('keeps the change control inert instead of opening a form that cannot submit', () => {
		render(EmailVerificationSection, props());

		const button = screen.getByRole('button', { name: /Change email/i });
		expect((button as HTMLButtonElement).disabled).toBe(true);
	});

	it('offers no address or password field to fill in', () => {
		// The previous markup revealed both, behind a submit button hardcoded
		// disabled, so the form could be completed but never sent.
		const { container } = render(EmailVerificationSection, props({ emailRevealOpen: true }));

		expect(container.querySelector('input[type="password"]')).toBeNull();
		expect(container.querySelector('input[placeholder="New email address"]')).toBeNull();
	});

	it('promises no mail it cannot send', () => {
		const { container } = render(EmailVerificationSection, props());
		const text = container.textContent ?? '';

		expect(text).not.toMatch(/we'll send a verification link/i);
		expect(text).not.toMatch(/password resets.*are sent/i);
	});

	it('says why the address cannot be changed', () => {
		const { container } = render(EmailVerificationSection, props());

		expect(container.textContent ?? '').toMatch(/outbound email/i);
	});
});
