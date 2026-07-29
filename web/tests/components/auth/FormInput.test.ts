import { describe, it, expect } from 'vitest';
import { render, screen } from '@testing-library/svelte';
import FormInput from '$lib/components/auth/FormInput.svelte';

describe('FormInput', () => {
	it('renders with a label', () => {
		render(FormInput, {
			props: { label: 'Email', value: '' }
		});

		expect(screen.getByText('Email')).toBeTruthy();
	});

	it('renders an input element', () => {
		render(FormInput, {
			props: { label: 'Email', value: '', type: 'email' }
		});

		const input = screen.getByRole('textbox');
		expect(input).toBeTruthy();
		expect(input.getAttribute('type')).toBe('email');
	});

	it('shows error text when error prop is set', () => {
		render(FormInput, {
			props: { label: 'Email', value: '', error: 'Email is required' }
		});

		const errorEl = screen.getByRole('alert');
		expect(errorEl.textContent).toBe('Email is required');
	});

	it('does not show error text when error is not set', () => {
		render(FormInput, {
			props: { label: 'Email', value: '' }
		});

		expect(screen.queryByRole('alert')).toBeNull();
	});

	it('applies error styling class when error is present', () => {
		render(FormInput, {
			props: { label: 'Password', value: '', error: 'Required' }
		});

		const input = screen.getByLabelText('Password');
		expect(input.className).toContain('form-input-error');
	});

	it('passes through HTML attributes like placeholder and required', () => {
		render(FormInput, {
			props: {
				label: 'Email',
				value: '',
				placeholder: 'you@example.com',
				required: true
			}
		});

		const input = screen.getByPlaceholderText('you@example.com');
		expect(input).toBeTruthy();
		expect(input.hasAttribute('required')).toBe(true);
	});
});
