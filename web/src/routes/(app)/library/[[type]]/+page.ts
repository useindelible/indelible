import { redirect } from '@sveltejs/kit';
import type { PageLoad } from './$types';

export const prerender = false;

export const load: PageLoad = ({ params, url }) => {
	if (!params.type || params.type === 'podcasts') {
		const qs = url.search;
		throw redirect(302, `/library/articles${qs}`);
	}
};
