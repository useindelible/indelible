<script lang="ts">
	import type { Snippet } from 'svelte';

	interface Props {
		bg: string;
		size?: number;
		radius?: number;
		children: Snippet;
	}

	let { bg, size = 40, radius = 10, children }: Props = $props();
</script>

<div
	class="icon-box"
	style="background: {bg}; width: {size}px; height: {size}px; border-radius: {radius}px;"
	aria-hidden="true"
>
	{@render children()}
</div>

<style>
	.icon-box {
		display: flex;
		align-items: center;
		justify-content: center;
		flex-shrink: 0;
	}
</style>
