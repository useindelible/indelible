import { describe, expect, it, vi } from 'vitest';
import { fireEvent, render, screen } from '@testing-library/svelte';

vi.mock('$app/navigation', () => ({ goto: vi.fn() }));
vi.mock('$lib/stores/library.svelte', () => ({
	getLibrary: () => ({ activeType: undefined, setActiveType: vi.fn() })
}));

import ContentTypeDropdown from '$lib/components/library/ContentTypeDropdown.svelte';
import SidebarNavList from '$lib/components/library/SidebarNavList.svelte';
import SidebarNavItem from '$lib/components/library/SidebarNavItem.svelte';

describe('SidebarNavItem', () => {
	it('renders active navigation links with labels and badges', () => {
		render(SidebarNavItem, {
			props: {
				href: '/library/articles',
				label: 'Articles',
				icon: 'articles',
				active: true,
				badge: 12
			}
		});

		const link = screen.getByRole('link', { name: /Articles/ });
		expect(link.getAttribute('href')).toBe('/library/articles');
		expect(link.getAttribute('aria-current')).toBe('page');
		expect(link.classList.contains('active')).toBe(true);
		expect(screen.getByText('12')).toBeTruthy();
	});
});

describe('SidebarNavList', () => {
	it('does not advertise podcasts as a launch library type', () => {
		render(SidebarNavList, {
			props: {
				isActive: () => false,
				showCountBadge: true,
				itemTypeCounts: { podcast: 3 }
			}
		});

		expect(screen.queryByRole('link', { name: /Podcasts/ })).toBeNull();
	});
});

describe('ContentTypeDropdown', () => {
	it('does not advertise podcasts as a launch content type', async () => {
		render(ContentTypeDropdown);
		await fireEvent.click(screen.getByRole('button', { name: 'All' }));

		expect(screen.queryByRole('option', { name: 'Podcasts' })).toBeNull();
	});
});
