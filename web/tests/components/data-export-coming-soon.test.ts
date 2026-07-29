import { render, screen } from '@testing-library/svelte';
import { describe, expect, it } from 'vitest';

import DataExportSection from '../../src/routes/(app)/preferences/account/components/DataExportSection.svelte';

describe('DataExportSection', () => {
	it('marks the archive as coming soon rather than an action about to run', () => {
		render(DataExportSection);

		expect(screen.getByText(/Coming soon/i)).toBeTruthy();
	});

	it('promises no delivery it cannot make', () => {
		const { container } = render(DataExportSection);
		const text = container.textContent ?? '';

		expect(text).not.toMatch(/ready in a few minutes/i);
		expect(text).not.toMatch(/we'll email you/i);
	});

	it('keeps the request control inert', () => {
		render(DataExportSection);

		const button = screen.getByRole('button', { name: /Request archive/i });
		expect((button as HTMLButtonElement).disabled).toBe(true);
	});
});
