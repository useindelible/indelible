<script lang="ts">
	import type { IntegrationProviderId } from '$lib/integrations/providers';

	interface Props {
		provider: IntegrationProviderId;
		size?: number;
	}

	let { provider, size = 32 }: Props = $props();

	const marks: Record<IntegrationProviderId, string> = {
		readwise: 'R',
		obsidian: 'O',
		notion: 'N'
	};

	const mark = $derived(marks[provider] ?? provider.charAt(0).toUpperCase());
	const radius = $derived(size <= 28 ? 7 : 9);
	const fontSize = $derived(Math.round(size * 0.44));
</script>

<span
	class="provider-icon provider-{provider}"
	style:width="{size}px"
	style:height="{size}px"
	style:border-radius="{radius}px"
	style:font-size="{fontSize}px"
	aria-hidden="true"
>
	{mark}
</span>

<style>
	.provider-icon {
		display: inline-flex;
		align-items: center;
		justify-content: center;
		flex-shrink: 0;
		font-family: var(--font-sans);
		font-weight: 700;
		letter-spacing: -0.02em;
		line-height: 1;
	}

	.provider-readwise {
		background: var(--provider-readwise-bg);
		color: var(--provider-readwise-fg);
	}

	/* Fallback for any other provider rendered here */
	.provider-obsidian,
	.provider-notion {
		background: var(--fill-secondary);
		color: var(--text-primary);
	}
</style>
