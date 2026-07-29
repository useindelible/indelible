<script lang="ts">
	import { page } from '$app/state';

	let { children } = $props();
</script>

<div class="onboarding-layout">
	<main class="onboarding-content">
		{#key page.url.pathname}
			{@render children()}
		{/key}
	</main>
</div>

<style>
	.onboarding-layout {
		min-height: 100vh;
		display: flex;
		align-items: center;
		justify-content: center;
		background: var(--bg-primary);
		padding: 24px;
	}

	.onboarding-content {
		width: 100%;
		max-width: 660px;
		display: flex;
		flex-direction: column;
		align-items: center;
	}
</style>
