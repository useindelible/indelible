<script lang="ts">
	import { t } from '$lib/i18n';
	interface Props {
		name: string;
		handle?: string | null;
		currentItemId?: string;
	}

	let { name, handle = null }: Props = $props();
</script>

<div class="author-card">
	<div class="author-avatar" aria-hidden="true">
		<svg
			viewBox="0 0 24 24"
			fill="none"
			stroke="currentColor"
			stroke-width="1.2"
			stroke-linecap="round"
			stroke-linejoin="round"
		>
			<path d="M20 21v-2a4 4 0 0 0-4-4H8a4 4 0 0 0-4 4v2" />
			<circle cx="12" cy="7" r="4" />
		</svg>
	</div>
	<div class="author-info">
		<div class="author-name">{name}</div>
		{#if handle}
			<div class="author-handle">{handle}</div>
		{/if}
	</div>
	<button
		class="follow-btn"
		type="button"
		disabled
		aria-label={$t('library_author_follow_unavailable')}
	>
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
			<path d="M2 12h20" />
			<path
				d="M12 2a15.3 15.3 0 0 1 4 10 15.3 15.3 0 0 1-4 10 15.3 15.3 0 0 1-4-10 15.3 15.3 0 0 1 4-10z"
			/>
		</svg>
		{$t('library_author_follow')}
	</button>
</div>

<style>
	.author-card {
		background: var(--fill-hover);
		border-radius: 14px;
		padding: 12px;
		display: flex;
		align-items: center;
		gap: 12px;
	}

	.author-avatar {
		width: 48px;
		height: 48px;
		border-radius: 50%;
		background: linear-gradient(135deg, #667eea, #764ba2);
		color: var(--text-on-color);
		display: flex;
		align-items: center;
		justify-content: center;
		flex-shrink: 0;
	}

	.author-avatar svg {
		width: 24px;
		height: 24px;
	}

	.author-info {
		flex: 1;
		min-width: 0;
	}

	.author-name {
		font-size: 14px;
		font-weight: 600;
		letter-spacing: -0.01em;
		color: var(--text-primary);
		white-space: nowrap;
		overflow: hidden;
		text-overflow: ellipsis;
	}

	.author-handle {
		font-size: 12px;
		font-weight: 400;
		color: var(--text-secondary);
		white-space: nowrap;
		overflow: hidden;
		text-overflow: ellipsis;
	}

	.follow-btn {
		display: inline-flex;
		align-items: center;
		justify-content: center;
		gap: 6px;
		padding: 5px 12px;
		border-radius: 6px;
		font-size: 12px;
		font-weight: 500;
		letter-spacing: -0.01em;
		font-family: var(--font-sans);
		min-height: 28px;
		white-space: nowrap;
		cursor: not-allowed;
		border: none;
		background: var(--accent);
		color: var(--text-on-color);
		opacity: 0.4;
		flex-shrink: 0;
	}

	.follow-btn svg {
		width: 13px;
		height: 13px;
	}
</style>
