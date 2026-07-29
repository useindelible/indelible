import { describe, it, expect } from 'vitest';
import { render, screen } from '@testing-library/svelte';
import FormButton from '$lib/components/auth/FormButton.svelte';

describe('FormButton', () => {
	it('renders button with text content', () => {
		render(FormButton, {
			props: {
				children: (($$anchor: Comment) => {
					const text = document.createTextNode('Sign In');
					$$anchor.before(text);
				}) as unknown as import('svelte').Snippet
			}
		});

		expect(screen.getByRole('button')).toBeTruthy();
		expect(screen.getByRole('button').textContent?.trim()).toBe('Sign In');
	});

	it('is not disabled by default', () => {
		render(FormButton, {
			props: {
				children: (($$anchor: Comment) => {
					const text = document.createTextNode('Submit');
					$$anchor.before(text);
				}) as unknown as import('svelte').Snippet
			}
		});

		const button = screen.getByRole('button');
		expect(button.hasAttribute('disabled')).toBe(false);
	});

	it('is disabled when loading is true', () => {
		render(FormButton, {
			props: {
				loading: true,
				children: (($$anchor: Comment) => {
					const text = document.createTextNode('Submit');
					$$anchor.before(text);
				}) as unknown as import('svelte').Snippet
			}
		});

		const button = screen.getByRole('button');
		expect(button.hasAttribute('disabled')).toBe(true);
	});

	it('shows spinner when loading', () => {
		render(FormButton, {
			props: {
				loading: true,
				children: (($$anchor: Comment) => {
					const text = document.createTextNode('Submit');
					$$anchor.before(text);
				}) as unknown as import('svelte').Snippet
			}
		});

		const spinner = document.querySelector('.spinner');
		expect(spinner).toBeTruthy();
	});

	it('does not show spinner when not loading', () => {
		render(FormButton, {
			props: {
				children: (($$anchor: Comment) => {
					const text = document.createTextNode('Submit');
					$$anchor.before(text);
				}) as unknown as import('svelte').Snippet
			}
		});

		const spinner = document.querySelector('.spinner');
		expect(spinner).toBeNull();
	});

	it('is disabled when disabled prop is true', () => {
		render(FormButton, {
			props: {
				disabled: true,
				children: (($$anchor: Comment) => {
					const text = document.createTextNode('Submit');
					$$anchor.before(text);
				}) as unknown as import('svelte').Snippet
			}
		});

		const button = screen.getByRole('button');
		expect(button.hasAttribute('disabled')).toBe(true);
	});

	it('has aria-busy when loading', () => {
		render(FormButton, {
			props: {
				loading: true,
				children: (($$anchor: Comment) => {
					const text = document.createTextNode('Submit');
					$$anchor.before(text);
				}) as unknown as import('svelte').Snippet
			}
		});

		const button = screen.getByRole('button');
		expect(button.getAttribute('aria-busy')).toBe('true');
	});

	it('defaults to submit type', () => {
		render(FormButton, {
			props: {
				children: (($$anchor: Comment) => {
					const text = document.createTextNode('Submit');
					$$anchor.before(text);
				}) as unknown as import('svelte').Snippet
			}
		});

		const button = screen.getByRole('button');
		expect(button.getAttribute('type')).toBe('submit');
	});
});
