import { describe, it, expect, vi } from 'vitest';
import { render, screen } from '@testing-library/svelte';
import { createRawSnippet } from 'svelte';
import { createApiModuleMock } from './helpers/api-module-mock';
import ProgressBar from '$lib/components/onboarding/ProgressBar.svelte';
import StepLayout from '$lib/components/onboarding/StepLayout.svelte';
import SelectableCard from '$lib/components/onboarding/SelectableCard.svelte';

vi.mock('$lib/api', () => createApiModuleMock());

const emptySnippet = createRawSnippet(() => ({
	render: () => '<div></div>'
}));

describe('ProgressBar', () => {
	it('renders 6 step indicators', () => {
		render(ProgressBar, { props: { currentStep: 0 } });
		const dots = document.querySelectorAll('.progress-dot');
		expect(dots).toHaveLength(6);
	});

	it('highlights the current step', () => {
		render(ProgressBar, { props: { currentStep: 2 } });
		const dots = document.querySelectorAll('.progress-dot');
		expect(dots[2]?.classList.contains('current')).toBe(true);
	});

	it('marks previous steps as completed', () => {
		render(ProgressBar, { props: { currentStep: 3 } });
		const dots = document.querySelectorAll('.progress-dot');
		expect(dots[0]?.classList.contains('completed')).toBe(true);
		expect(dots[1]?.classList.contains('completed')).toBe(true);
		expect(dots[2]?.classList.contains('completed')).toBe(true);
		expect(dots[3]?.classList.contains('current')).toBe(true);
		expect(dots[4]?.classList.contains('completed')).toBe(false);
	});

	it('shows accessible label for current step', () => {
		render(ProgressBar, { props: { currentStep: 4 } });
		expect(screen.getByText('Step 5: AI Setup (current)')).toBeTruthy();
	});

	it('has aria-current on current step', () => {
		render(ProgressBar, { props: { currentStep: 1 } });
		const dots = document.querySelectorAll('.progress-dot');
		expect(dots[1]?.getAttribute('aria-current')).toBe('step');
		expect(dots[0]?.getAttribute('aria-current')).toBeNull();
	});
});

describe('StepLayout', () => {
	it('renders title', () => {
		render(StepLayout, {
			props: {
				title: 'Test Title',
				children: emptySnippet
			}
		});
		expect(screen.getByText('Test Title')).toBeTruthy();
	});

	it('renders description when provided', () => {
		render(StepLayout, {
			props: {
				title: 'Title',
				description: 'Test description',
				children: emptySnippet
			}
		});
		expect(screen.getByText('Test description')).toBeTruthy();
	});

	it('renders Continue button by default', () => {
		render(StepLayout, {
			props: {
				title: 'Title',
				children: emptySnippet
			}
		});
		expect(screen.getByText('Continue')).toBeTruthy();
	});

	it('renders custom continue label', () => {
		render(StepLayout, {
			props: {
				title: 'Title',
				continueLabel: 'Get Started',
				children: emptySnippet
			}
		});
		expect(screen.getByText('Get Started')).toBeTruthy();
	});

	it('renders Skip button when showSkip is true', () => {
		render(StepLayout, {
			props: {
				title: 'Title',
				showSkip: true,
				children: emptySnippet
			}
		});
		expect(screen.getByText('Skip')).toBeTruthy();
	});

	it('does not render Skip button by default', () => {
		render(StepLayout, {
			props: {
				title: 'Title',
				children: emptySnippet
			}
		});
		expect(screen.queryByText('Skip')).toBeNull();
	});

	it('shows spinner when submitting', () => {
		render(StepLayout, {
			props: {
				title: 'Title',
				submitting: true,
				children: emptySnippet
			}
		});
		expect(document.querySelector('.btn-spinner')).toBeTruthy();
	});

	it('disables buttons when submitting', () => {
		render(StepLayout, {
			props: {
				title: 'Title',
				submitting: true,
				showSkip: true,
				children: emptySnippet
			}
		});
		const buttons = document.querySelectorAll('button');
		buttons.forEach((btn) => {
			expect(btn.disabled).toBe(true);
		});
	});
});

describe('SelectableCard', () => {
	it('renders label', () => {
		render(SelectableCard, { props: { label: 'Test Card' } });
		expect(screen.getByText('Test Card')).toBeTruthy();
	});

	it('renders description when provided', () => {
		render(SelectableCard, {
			props: { label: 'Card', description: 'Card description' }
		});
		expect(screen.getByText('Card description')).toBeTruthy();
	});

	it('applies selected class when selected', () => {
		render(SelectableCard, {
			props: { label: 'Card', selected: true }
		});
		const button = document.querySelector('.selectable-card');
		expect(button?.classList.contains('selected')).toBe(true);
	});

	it('does not apply selected class when not selected', () => {
		render(SelectableCard, {
			props: { label: 'Card', selected: false }
		});
		const button = document.querySelector('.selectable-card');
		expect(button?.classList.contains('selected')).toBe(false);
	});

	it('calls onclick when clicked', async () => {
		const onclick = vi.fn();
		render(SelectableCard, {
			props: { label: 'Card', onclick }
		});
		const button = document.querySelector('.selectable-card') as HTMLButtonElement;
		button.click();
		expect(onclick).toHaveBeenCalledOnce();
	});
});
