import { describe, expect, it } from 'vitest';
import { render, screen } from '@testing-library/svelte';

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
