export type SidebarHomeView = 'feed' | 'search' | 'library' | undefined;
export type SidebarHomePath = '/feed' | '/search' | '/library';
export type SidebarIcon =
	| 'home'
	| 'articles'
	| 'books'
	| 'emails'
	| 'pdfs'
	| 'tweets'
	| 'videos'
	| 'podcasts'
	| 'tags'
	| 'feed'
	| 'collections'
	| 'pinned'
	| 'trash'
	| 'search'
	| 'preferences';

export function getDefaultHomePath(view: SidebarHomeView): SidebarHomePath {
	if (view === 'feed') return '/feed';
	if (view === 'search') return '/search';
	return '/library';
}

export function isSidebarPathActive(currentPathname: string, href: string): boolean {
	if (href === '/library') {
		return currentPathname === '/library';
	}
	return currentPathname.startsWith(href);
}

export function getSmartListHref(id: string, libraryPath: string): string {
	return `${libraryPath}?smart_list=${id}`;
}

export function getInitials(name: string): string {
	return name
		.split(' ')
		.slice(0, 2)
		.map((part) => part[0] ?? '')
		.join('')
		.toUpperCase();
}
