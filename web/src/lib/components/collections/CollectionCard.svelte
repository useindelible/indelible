<script lang="ts">
	import { resolve } from '$app/paths';
	import type { CollectionResponse } from '$lib/api/generated/types.gen';
	import { t } from '$lib/i18n';

	interface Props {
		collection: CollectionResponse;
		subCount?: number;
		onEdit?: (collection: CollectionResponse) => void;
		onDelete?: (collection: CollectionResponse) => void;
	}

	let { collection, subCount = 0, onEdit, onDelete }: Props = $props();

	const colorIndex = $derived(
		collection.id.split('').reduce((acc, c) => acc + c.charCodeAt(0), 0) % 6
	);
</script>

<a href={resolve('/(app)/collections/[id]', { id: collection.id })} class="collection-card">
	<div class="card-gradient" style:background={`var(--collection-gradient-${colorIndex})`}>
		<span class="gradient-icon" aria-hidden="true">
			{collection.icon || '📁'}
		</span>
		{#if onEdit || onDelete}
			<div class="card-actions">
				{#if onEdit}
					<button
						type="button"
						class="action-btn"
						aria-label={$t('collection_edit_named', { values: { name: collection.name } })}
						onclick={(e) => {
							e.stopPropagation();
							e.preventDefault();
							onEdit?.(collection);
						}}
					>
						<svg
							viewBox="0 0 24 24"
							fill="none"
							stroke="currentColor"
							stroke-width="1.6"
							stroke-linecap="round"
							stroke-linejoin="round"
						>
							<path d="M11 4H4a2 2 0 00-2 2v14a2 2 0 002 2h14a2 2 0 002-2v-7" />
							<path d="M18.5 2.5a2.121 2.121 0 013 3L12 15l-4 1 1-4 9.5-9.5z" />
						</svg>
					</button>
				{/if}
				{#if onDelete}
					<button
						type="button"
						class="action-btn action-btn-danger"
						aria-label={$t('collection_delete_named', { values: { name: collection.name } })}
						onclick={(e) => {
							e.stopPropagation();
							e.preventDefault();
							onDelete?.(collection);
						}}
					>
						<svg
							viewBox="0 0 24 24"
							fill="none"
							stroke="currentColor"
							stroke-width="1.6"
							stroke-linecap="round"
							stroke-linejoin="round"
						>
							<path d="M3 6h18" />
							<path d="M16 6V4a2 2 0 00-2-2h-4a2 2 0 00-2 2v2" />
							<path d="M19 6l-1 14a2 2 0 01-2 2H8a2 2 0 01-2-2L5 6" />
						</svg>
					</button>
				{/if}
			</div>
		{/if}
	</div>
	<div class="card-body">
		<span class="card-name">{collection.name}</span>
		{#if collection.description}
			<span class="card-desc">{collection.description}</span>
		{/if}
	</div>
	<div class="card-footer">
		<span class="card-count"
			>{$t('collection_item_count', {
				values: { count: collection.item_count }
			})}</span
		>
		{#if subCount > 0}
			<span class="card-sep">·</span>
			<span class="card-sub"
				>{$t('collection_subcollection_count', { values: { count: subCount } })}</span
			>
		{/if}
		<span
			class="card-dot"
			style:background={`var(--collection-gradient-${colorIndex})`}
			aria-hidden="true"
		></span>
	</div>
</a>

<style>
	.collection-card {
		display: flex;
		flex-direction: column;
		border-radius: var(--radius-xl);
		border: 1px solid var(--border-primary);
		background: var(--bg-elevated);
		text-decoration: none;
		color: inherit;
		overflow: hidden;
		transition:
			transform 0.15s ease,
			box-shadow 0.15s ease;
		cursor: pointer;
	}

	.collection-card:hover {
		transform: translateY(-3px);
		box-shadow: var(--shadow-1);
	}

	.card-gradient {
		height: 88px;
		display: flex;
		align-items: center;
		justify-content: center;
		position: relative;
		overflow: hidden;
	}

	.gradient-icon {
		font-size: 32px;
		line-height: 1;
		filter: drop-shadow(0 2px 4px rgba(0, 0, 0, 0.2));
		z-index: 1;
	}

	.card-actions {
		position: absolute;
		top: 8px;
		right: 8px;
		display: flex;
		align-items: center;
		gap: 4px;
		opacity: 0;
		transition: opacity 0.12s ease;
	}

	.collection-card:hover .card-actions {
		opacity: 1;
	}

	.action-btn {
		width: 26px;
		height: 26px;
		border-radius: 6px;
		border: none;
		background: var(--gradient-action-bg);
		backdrop-filter: blur(4px);
		display: flex;
		align-items: center;
		justify-content: center;
		cursor: pointer;
		color: var(--gradient-action-color);
		padding: 0;
		transition:
			background 0.12s ease,
			color 0.12s ease;
	}

	.action-btn:hover {
		background: var(--gradient-action-bg-hover);
	}

	.action-btn-danger:hover {
		background: var(--gradient-action-danger-bg-hover);
	}

	.action-btn svg {
		width: 13px;
		height: 13px;
	}

	.card-body {
		display: flex;
		flex-direction: column;
		gap: 4px;
		padding: 12px 14px 8px;
	}

	.card-name {
		font-family: var(--font-sans);
		font-size: 14px;
		font-weight: 600;
		letter-spacing: -0.01em;
		color: var(--text-primary);
		white-space: nowrap;
		overflow: hidden;
		text-overflow: ellipsis;
	}

	.card-desc {
		font-family: var(--font-sans);
		font-size: 12px;
		font-weight: 400;
		color: var(--text-secondary);
		display: -webkit-box;
		-webkit-line-clamp: 2;
		-webkit-box-orient: vertical;
		overflow: hidden;
		line-height: 1.5;
	}

	.card-footer {
		display: flex;
		align-items: center;
		gap: 4px;
		padding: 0 14px 12px;
	}

	.card-count {
		font-family: var(--font-sans);
		font-size: 11px;
		font-weight: 400;
		color: var(--text-tertiary);
	}

	.card-sep {
		font-size: 11px;
		color: var(--text-quaternary);
	}

	.card-sub {
		font-family: var(--font-sans);
		font-size: 11px;
		font-weight: 400;
		color: var(--text-tertiary);
	}

	.card-dot {
		width: 7px;
		height: 7px;
		border-radius: var(--radius-circle);
		margin-left: auto;
		flex-shrink: 0;
	}
</style>
