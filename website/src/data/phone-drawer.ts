/**
 * The navigation drawer on a phone.
 *
 * The item set mirrors SidebarNavList in the app. There is no Podcasts row:
 * first-class podcasts are deferred in the shipping nav, even though the
 * mobile design file still shows one.
 */
import type { PhoneIconName } from '../lib/phone-icons';

export interface DrawerItem {
	icon: PhoneIconName;
	label: string;
	count: string;
}

export interface DrawerCollection {
	name: string;
	colour: string;
	count: string;
}

export const DRAWER_LIBRARY: readonly DrawerItem[] = [
	{ icon: 'grid', label: 'All items', count: '91' },
	{ icon: 'article', label: 'Articles', count: '42' },
	{ icon: 'book', label: 'Books', count: '7' },
	{ icon: 'email', label: 'Emails', count: '12' },
	{ icon: 'pdf', label: 'PDFs', count: '9' },
	{ icon: 'video', label: 'Videos', count: '14' },
	{ icon: 'feed', label: 'Feed', count: '12' },
];

/** Highlighted row, matching the section the drawer was opened from. */
export const DRAWER_ACTIVE = 'Articles';

export const DRAWER_COLLECTIONS: readonly DrawerCollection[] = [
	{ name: 'Research', colour: '#8250DF', count: '26' },
	{ name: 'Writing craft', colour: '#1A7F37', count: '19' },
	{ name: 'Climate and energy', colour: '#BC4C00', count: '11' },
];

export const DRAWER_TAGS = { label: 'All tags', count: '38' } as const;
export const DRAWER_SUBTITLE = '91 saved items';
