import { SvelteURL } from 'svelte/reactivity';

class MockPage {
	url = $state<SvelteURL>(new SvelteURL('http://localhost/dashboard'));
	params = $state<Record<string, string>>({});
	error = $state<{ message: string } | null>(null);
	status = $state(200);
}

export const page = new MockPage();
