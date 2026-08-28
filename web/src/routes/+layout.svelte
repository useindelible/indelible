<script lang="ts">
	import '$lib/styles/tokens.css';
	import '../app.css';
	import { page } from '$app/state';
	import { initTheme } from '$lib/styles/theme';
	import { t } from '$lib/i18n';
	import { readTitleOverride, resolveTitle } from '$lib/stores/page-title.svelte';
	import { onMount } from 'svelte';

	let { children } = $props();

	const documentTitle = $derived(
		resolveTitle({
			pathname: page.url.pathname,
			errorStatus: page.error ? page.status : null,
			override: readTitleOverride(),
			translate: $t
		})
	);

	onMount(() => {
		initTheme();
	});
</script>

<svelte:head>
	<title>{documentTitle}</title>
</svelte:head>

{@render children()}
