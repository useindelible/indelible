<script lang="ts">
	interface Action {
		label: string;
		onclick: () => void;
		style?: 'primary' | 'secondary' | 'warning' | 'danger';
	}

	interface Props {
		variant: 'setup' | 'warning' | 'error';
		title: string;
		message: string;
		actions?: Action[];
	}

	let { variant, title, message, actions = [] }: Props = $props();
</script>

<div class="tts-banner {variant}" role="status">
	<div class="tts-banner-icon">
		{#if variant === 'setup'}
			<svg
				viewBox="0 0 24 24"
				fill="none"
				stroke="currentColor"
				stroke-width="1.8"
				stroke-linecap="round"
				stroke-linejoin="round"
				aria-hidden="true"
			>
				<path d="M11 5L6 9H2v6h4l5 4V5z" />
				<path d="M15.54 8.46a5 5 0 010 7.07" />
				<path d="M19.07 4.93a10 10 0 010 14.14" />
			</svg>
		{:else if variant === 'warning'}
			<svg
				viewBox="0 0 24 24"
				fill="none"
				stroke="currentColor"
				stroke-width="1.5"
				stroke-linecap="round"
				stroke-linejoin="round"
				aria-hidden="true"
			>
				<path
					d="M10.29 3.86L1.82 18a2 2 0 001.71 3h16.94a2 2 0 001.71-3L13.71 3.86a2 2 0 00-3.42 0z"
				/>
				<line x1="12" y1="9" x2="12" y2="13" />
				<line x1="12" y1="17" x2="12.01" y2="17" />
			</svg>
		{:else}
			<svg
				viewBox="0 0 24 24"
				fill="none"
				stroke="currentColor"
				stroke-width="1.5"
				stroke-linecap="round"
				stroke-linejoin="round"
				aria-hidden="true"
			>
				<circle cx="12" cy="12" r="10" />
				<line x1="15" y1="9" x2="9" y2="15" />
				<line x1="9" y1="9" x2="15" y2="15" />
			</svg>
		{/if}
	</div>

	<div class="tts-banner-body">
		<div class="tts-banner-title">{title}</div>
		<div class="tts-banner-message">{message}</div>
		{#if actions.length > 0}
			<div class="tts-banner-actions">
				{#each actions as action (action.label)}
					<button
						type="button"
						class="tts-banner-btn {action.style ?? 'secondary'}"
						onclick={action.onclick}
					>
						{action.label}
					</button>
				{/each}
			</div>
		{/if}
	</div>
</div>

<style>
	.tts-banner {
		display: flex;
		align-items: flex-start;
		gap: 12px;
		padding: 14px 16px 16px;
		border-bottom: 0.5px solid;
		flex-shrink: 0;
	}

	.tts-banner.setup {
		background: var(--tts-banner-setup-bg);
		border-color: var(--tts-banner-setup-border);
	}

	.tts-banner.warning {
		background: var(--tts-banner-warning-bg);
		border-color: var(--tts-banner-warning-border);
	}

	.tts-banner.error {
		background: var(--tts-banner-error-bg);
		border-color: var(--tts-banner-error-border);
	}

	.tts-banner-icon {
		width: 28px;
		height: 28px;
		border-radius: 8px;
		flex-shrink: 0;
		display: flex;
		align-items: center;
		justify-content: center;
	}

	.tts-banner.setup .tts-banner-icon {
		background: var(--tts-banner-setup-icon-bg);
		color: var(--accent);
	}

	.tts-banner.warning .tts-banner-icon {
		background: var(--tts-banner-warning-icon-bg);
		color: var(--warning);
	}

	.tts-banner.error .tts-banner-icon {
		background: var(--tts-banner-error-icon-bg);
		color: var(--destructive);
	}

	.tts-banner-icon :global(svg) {
		width: 14px;
		height: 14px;
	}

	.tts-banner-body {
		flex: 1;
		display: flex;
		flex-direction: column;
		gap: 4px;
		min-width: 0;
	}

	.tts-banner-title {
		font-size: 13px;
		font-weight: 600;
		letter-spacing: -0.01em;
		color: var(--text-primary);
		line-height: 1.35;
		font-family: var(--font-sans);
	}

	.tts-banner-message {
		font-size: 12px;
		font-weight: 400;
		color: var(--text-secondary);
		letter-spacing: -0.005em;
		line-height: 1.45;
		font-family: var(--font-sans);
	}

	.tts-banner-actions {
		display: flex;
		gap: 6px;
		margin-top: 8px;
	}

	.tts-banner-btn {
		padding: 6px 12px;
		border-radius: 7px;
		font-family: var(--font-sans);
		font-size: 12px;
		font-weight: 600;
		letter-spacing: -0.005em;
		border: none;
		cursor: pointer;
		transition: background 120ms ease;
		white-space: nowrap;
	}

	.tts-banner-btn.primary {
		background: var(--accent);
		color: var(--text-on-color);
	}

	.tts-banner-btn.primary:hover {
		background: var(--accent-hover);
	}

	.tts-banner-btn.secondary {
		background: var(--fill-hover);
		color: var(--text-primary);
		border: 0.5px solid var(--border-primary);
	}

	.tts-banner-btn.secondary:hover {
		background: var(--fill-selected);
	}

	.tts-banner-btn.warning {
		background: var(--warning);
		color: var(--text-on-color);
	}

	.tts-banner-btn.danger {
		background: var(--destructive);
		color: var(--text-on-color);
	}
</style>
