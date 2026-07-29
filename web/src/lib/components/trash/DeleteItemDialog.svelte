<script lang="ts">
	interface Props {
		item: { id: string; title: string } | null;
		onConfirm: (id: string) => void;
		onClose: () => void;
		deleting: boolean;
	}

	let { item, onConfirm, onClose, deleting }: Props = $props();
</script>

{#if item}
	<div
		class="cmd-backdrop"
		role="dialog"
		aria-modal="true"
		aria-label="Delete permanently"
		tabindex="-1"
		onclick={onClose}
		onkeydown={(e) => {
			if (e.key === 'Escape') {
				e.preventDefault();
				onClose();
			}
		}}
	>
		<div class="cmd-card" role="none" onclick={(e) => e.stopPropagation()} onkeydown={() => {}}>
			<div class="cmd-body">
				<h2 class="dialog-title">Delete permanently?</h2>
				<p class="dialog-text">
					Permanently delete &ldquo;{item.title}&rdquo;? This will remove the item and all its
					highlights, notes, and collection links.
				</p>
			</div>
			<div class="cmd-controls">
				<button type="button" class="cmd-secondary" onclick={onClose}>Cancel</button>
				<button
					type="button"
					class="cmd-action cmd-action-danger"
					disabled={deleting}
					onclick={() => onConfirm(item.id)}
				>
					{deleting ? 'Deleting...' : 'Delete'}
				</button>
			</div>
		</div>
	</div>
{/if}

<style>
	.cmd-backdrop {
		position: fixed;
		inset: 0;
		background: rgba(0, 0, 0, 0.4);
		backdrop-filter: blur(4px);
		-webkit-backdrop-filter: blur(4px);
		display: flex;
		align-items: flex-start;
		justify-content: center;
		padding-top: 80px;
		z-index: 300;
		box-sizing: border-box;
	}

	:global([data-theme='dark']) .cmd-backdrop {
		background: rgba(0, 0, 0, 0.6);
	}

	.cmd-card {
		width: 420px;
		max-width: calc(100vw - 32px);
		background: var(--bg-elevated);
		border-radius: 16px;
		box-shadow:
			0 24px 80px rgba(0, 0, 0, 0.22),
			0 0 0 0.5px rgba(0, 0, 0, 0.06);
	}

	:global([data-theme='dark']) .cmd-card {
		box-shadow:
			0 24px 80px rgba(0, 0, 0, 0.55),
			0 0 0 0.5px rgba(255, 255, 255, 0.08);
	}

	.cmd-body {
		padding: 20px 20px 8px;
		display: flex;
		flex-direction: column;
		gap: 8px;
	}

	.dialog-title {
		font-size: 17px;
		font-weight: 600;
		color: var(--text-primary);
		margin: 0;
		letter-spacing: -0.02em;
	}

	.dialog-text {
		font-size: 13px;
		font-weight: 400;
		line-height: 1.5;
		color: var(--text-secondary);
		margin: 0;
	}

	.cmd-controls {
		display: flex;
		align-items: center;
		gap: 6px;
		padding: 14px 20px 18px;
	}

	.cmd-secondary {
		padding: 6px 14px;
		border-radius: 980px;
		border: 1px solid var(--border-primary);
		background: transparent;
		font-family: var(--font-sans);
		font-size: 13px;
		font-weight: 500;
		color: var(--text-secondary);
		cursor: pointer;
		transition: background 120ms ease;
		letter-spacing: -0.01em;
	}

	.cmd-secondary:hover {
		background: var(--fill-hover);
	}

	.cmd-action {
		margin-left: auto;
		padding: 6px 16px;
		border-radius: 980px;
		border: none;
		font-family: var(--font-sans);
		font-size: 13px;
		font-weight: 600;
		cursor: pointer;
		letter-spacing: -0.01em;
		color: var(--text-on-color);
		flex-shrink: 0;
		transition: opacity 120ms ease;
	}

	.cmd-action:hover:not(:disabled) {
		opacity: 0.88;
	}

	.cmd-action:disabled {
		opacity: 0.32;
		cursor: not-allowed;
	}

	.cmd-action-danger {
		background: var(--destructive);
	}
</style>
