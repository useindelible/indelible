<script lang="ts">
	import '$lib/styles/tokens.css';
	import '../app.css';
	import { initTheme } from '$lib/styles/theme';
	import { onMount } from 'svelte';

	let { children } = $props();

	onMount(() => {
		initTheme();
	});
</script>

{@render children()}
