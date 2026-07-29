<script lang="ts">
	import { resolve } from '$app/paths';
	import type { CollectionResponse } from '$lib/api/generated/types.gen';

	interface Props {
		path: CollectionResponse[];
	}

	let { path }: Props = $props();
</script>

<nav class="breadcrumbs" aria-label="Collection breadcrumbs">
	<ol>
		<li>
			<a href={resolve('/(app)/collections')} class="crumb">Collections</a>
		</li>
		{#each path as segment, i (segment.id)}
			<li>
				<span class="separator" aria-hidden="true">/</span>
				{#if i < path.length - 1}
					<a href={resolve('/(app)/collections/[id]', { id: segment.id })} class="crumb"
						>{segment.name}</a
					>
				{:else}
					<span class="crumb crumb-current" aria-current="page">{segment.name}</span>
				{/if}
			</li>
		{/each}
	</ol>
</nav>

<style>
	.breadcrumbs ol {
		display: flex;
		align-items: center;
		gap: 2px;
		list-style: none;
		margin: 0;
		padding: 0;
		flex-wrap: wrap;
	}

	.breadcrumbs li {
		display: flex;
		align-items: center;
		gap: 2px;
	}

	.separator {
		font-size: 12px;
		color: var(--text-tertiary);
		margin: 0 2px;
	}

	.crumb {
		font-family: var(--font-sans);
		font-size: 13px;
		font-weight: 400;
		color: var(--text-secondary);
		text-decoration: none;
		letter-spacing: -0.01em;
	}

	a.crumb:hover {
		color: var(--accent);
	}

	.crumb-current {
		color: var(--text-primary);
		font-weight: 500;
	}
</style>
