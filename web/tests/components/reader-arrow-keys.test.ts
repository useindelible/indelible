import { afterEach, describe, expect, it, vi } from 'vitest';
import { cleanup, fireEvent, render, screen } from '@testing-library/svelte';

import ViewTabs from '$lib/components/reader/ViewTabs.svelte';

describe('ViewTabs arrow keys', () => {
	afterEach(cleanup);

	it('switches tabs without letting the arrow reach window-level bindings', async () => {
		const onTabChange = vi.fn();
		const reachedWindow = vi.fn();
		window.addEventListener('keydown', reachedWindow);

		render(ViewTabs, {
			props: { activeTab: 'reader', availableTabs: ['reader', 'original'], onTabChange }
		});

		await fireEvent.keyDown(screen.getByRole('tablist'), { key: 'ArrowRight' });

		expect(onTabChange).toHaveBeenCalledWith('original');
		expect(reachedWindow).not.toHaveBeenCalled();
		window.removeEventListener('keydown', reachedWindow);
	});
});
