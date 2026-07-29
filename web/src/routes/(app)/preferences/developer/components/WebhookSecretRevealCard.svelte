<script lang="ts">
	interface WebhookSecret {
		name: string;
		raw_secret: string;
	}

	interface Props {
		secret: WebhookSecret | null;
		copied: boolean;
		onCopy: () => void;
		onDismiss: () => void;
	}

	let { secret, copied, onCopy, onDismiss }: Props = $props();
</script>

{#if secret}
	<div class="reveal">
		<div class="reveal-head">
			<div class="reveal-title">
				Signing secret ready <span class="badge-warn">Shown once</span>
			</div>
			<button type="button" class="btn ghost compact" onclick={onDismiss}>Dismiss</button>
		</div>
		<div class="reveal-desc">
			Copy this secret into {secret.name}. Indelible stores it encrypted and uses it to sign every
			webhook payload.
		</div>
		<div class="reveal-token">
			<code>{secret.raw_secret}</code>
			<button type="button" class="btn ghost compact" onclick={onCopy}>
				{copied ? 'Copied' : 'Copy'}
			</button>
		</div>
		<div class="reveal-foot">
			<button type="button" class="btn primary" onclick={onDismiss}>I've saved it</button>
		</div>
	</div>
{/if}

<style>
	.reveal {
		margin: 0 0 18px;
		background: var(--dev-card-strong);
		border-radius: 14px;
		padding: 22px 24px;
		box-shadow:
			0 12px 40px -16px var(--dev-accent-glow),
			inset 0 0 0 1.5px var(--dev-accent),
			0 0 0 4px var(--dev-accent-soft);
	}

	.reveal-head,
	.reveal-token,
	.reveal-foot {
		display: flex;
		align-items: center;
	}

	.reveal-head {
		justify-content: space-between;
		margin-bottom: 14px;
	}

	.reveal-title {
		font-size: 14px;
		font-weight: 600;
		color: var(--text-primary);
		letter-spacing: 0;
		display: inline-flex;
		align-items: center;
		gap: 8px;
	}

	.badge-warn {
		padding: 2px 8px;
		border-radius: 980px;
		font-size: 10.5px;
		font-weight: 700;
		background: var(--dev-warning-soft);
		color: var(--dev-warning-fg);
		letter-spacing: 0;
		text-transform: uppercase;
	}

	.reveal-desc {
		font-size: 12.5px;
		color: var(--text-secondary);
		margin-bottom: 14px;
		line-height: 1.5;
		max-width: 540px;
	}

	.reveal-token {
		gap: 8px;
		padding: 4px 4px 4px 14px;
		background: var(--bg-elevated);
		border-radius: 10px;
		box-shadow: inset 0 0 0 0.5px var(--border-primary);
	}

	code {
		font-family: 'SF Mono', 'Fira Code', Menlo, ui-monospace, monospace;
		font-size: 13px;
		color: var(--text-primary);
		letter-spacing: 0;
		flex: 1;
		user-select: all;
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}

	.reveal-foot {
		justify-content: flex-end;
		margin-top: 14px;
	}

	.btn {
		border: none;
		border-radius: 8px;
		padding: 7px 14px;
		font: inherit;
		font-size: 12.5px;
		font-weight: 500;
		cursor: pointer;
		letter-spacing: 0;
	}

	.btn.compact {
		padding: 5px 10px;
		font-size: 11.5px;
	}

	.btn.ghost {
		background: transparent;
		color: var(--text-primary);
		box-shadow: inset 0 0 0 0.5px var(--border-primary);
	}

	.btn.primary {
		background: var(--dev-accent);
		color: var(--text-on-color);
	}
</style>
