import { redirect } from '@sveltejs/kit';

export const load = ({ url }) => {
	const job = url.searchParams.get('job');
	const target = job
		? `/preferences/integrations?job=${encodeURIComponent(job)}`
		: '/preferences/integrations';
	throw redirect(302, target);
};
