import { describe, it, expect } from 'vitest';
import { render, screen } from '@testing-library/svelte';
import AuthDivider from '$lib/components/auth/AuthDivider.svelte';

describe('AuthDivider', () => {
	it('renders with default "or" text', () => {
		render(AuthDivider);

		expect(screen.getByText('or')).toBeTruthy();
	});

	it('renders with custom text', () => {
		render(AuthDivider, { props: { text: 'or sign up with' } });

		expect(screen.getByText('or sign up with')).toBeTruthy();
	});

	it('renders two divider lines', () => {
		const { container } = render(AuthDivider);

		const lines = container.querySelectorAll('.auth-divider-line');
		expect(lines.length).toBe(2);
	});
});
