import type { PageLoadEvent } from './$types';

export function load({ url }: PageLoadEvent) {
	const token = url.searchParams.get('token');
	return { token };
}
