import { describe, expect, it, vi } from 'vitest';
import { render, screen } from '@testing-library/svelte';

import WorkflowSection from '../../src/routes/(app)/preferences/reading-appearance/components/WorkflowSection.svelte';

describe('WorkflowSection', () => {
	it('names the auto-advance checkbox from its visible label', () => {
		render(WorkflowSection, {
			triageMode: 'focus',
			autoAdvance: false,
			onTriageModeChange: vi.fn(),
			onAutoAdvanceChange: vi.fn()
		});

		expect(screen.getByRole('checkbox', { name: 'Auto-advance' })).toBeTruthy();
	});
});
