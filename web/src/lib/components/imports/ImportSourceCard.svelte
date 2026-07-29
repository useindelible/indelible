<script lang="ts">
	import Button from '$lib/components/ui/Button.svelte';
	import ProviderIcon from './ProviderIcon.svelte';
	import type { IntegrationProviderId } from '$lib/integrations/providers';

	interface Props {
		provider: IntegrationProviderId;
		title: string;
		description: string;
		ctaLabel: string;
		onClick: () => void;
	}

	let { provider, title, description, ctaLabel, onClick }: Props = $props();
</script>

<article class="source-card">
	<header class="header">
		<ProviderIcon {provider} size={32} />
		<h3 class="title">{title}</h3>
	</header>
	<p class="description">{description}</p>
	<footer class="footer">
		<Button variant="primary" size="sm" onclick={onClick} fullWidth>
			{ctaLabel}
		</Button>
	</footer>
</article>

<style>
	.source-card {
		display: flex;
		flex-direction: column;
		gap: 10px;
		padding: 16px;
		border-radius: var(--radius-xl);
		background: var(--bg-secondary);
		border: 0.5px solid var(--border-primary);
	}

	.header {
		display: flex;
		align-items: center;
		gap: 10px;
	}

	.title {
		font-family: var(--font-sans);
		font-size: 15px;
		font-weight: 600;
		letter-spacing: -0.01em;
		color: var(--text-primary);
		margin: 0;
	}

	.description {
		font-family: var(--font-sans);
		font-size: 12.5px;
		color: var(--text-secondary);
		margin: 0;
		line-height: 1.5;
		flex: 1;
	}

	.footer {
		margin-top: 4px;
	}
</style>
