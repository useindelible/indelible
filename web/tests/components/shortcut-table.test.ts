import { afterEach, describe, expect, it, vi } from 'vitest';
import { cleanup, render, screen } from '@testing-library/svelte';

import { isApplePlatform } from '$lib/shortcuts/platform';
import ShortcutTable from '$lib/components/shortcuts/ShortcutTable.svelte';

vi.mock('$lib/shortcuts/platform', () => ({ isApplePlatform: vi.fn() }));

function capsFor(label: string): string[] {
	const row = screen.getByText(label).parentElement!;
	return [...row.querySelectorAll('.kbd')].map((cap) => cap.textContent);
}

describe('ShortcutTable', () => {
	afterEach(cleanup);

	it('shows the command glyph on Apple platforms', () => {
		vi.mocked(isApplePlatform).mockReturnValue(true);
		render(ShortcutTable);
		expect(capsFor('Search')).toEqual(['⌘', 'K']);
	});

	it('shows the control label everywhere else', () => {
		vi.mocked(isApplePlatform).mockReturnValue(false);
		render(ShortcutTable);
		expect(capsFor('Search')).toEqual(['Ctrl', 'K']);
		expect(capsFor('Save URL')).toEqual(['N']);
	});
});
