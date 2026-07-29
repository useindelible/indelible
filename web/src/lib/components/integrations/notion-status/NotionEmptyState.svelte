<script lang="ts">
	interface Props {
		onReauthorize: () => void;
	}

	let { onReauthorize }: Props = $props();
</script>

<section class="group" data-testid="notion-status-panel" data-state="empty">
	<div class="group-label">Connection</div>
	<div class="group-desc">
		Connect your Notion workspace to export saved documents and highlights into an Indelible managed
		database.
	</div>
	<div class="group-card">
		<div class="empty-card">
			<div class="conn-card-icon empty">N</div>
			<div class="empty-text">
				<div class="empty-title">Notion is not connected</div>
				<div class="empty-hint">
					Indelible will provision a managed database in your workspace on the first export.
				</div>
			</div>
			<button class="btn primary" type="button" onclick={onReauthorize}>Connect Notion</button>
		</div>
	</div>
</section>

<style>
	.group {
		margin-bottom: 28px;
	}

	.group-label {
		font-size: 11px;
		font-weight: 600;
		letter-spacing: 0.1em;
		text-transform: uppercase;
		color: var(--text-tertiary);
		padding: 0 4px 4px;
	}

	.group-desc {
		font-size: 12.5px;
		color: var(--text-secondary);
		padding: 0 4px 10px;
		line-height: 1.45;
	}

	.group-card {
		background: var(--bg-elevated);
		border-radius: 14px;
		overflow: hidden;
		box-shadow: inset 0 0 0 0.5px var(--border-primary);
		container-type: inline-size;
		container-name: settings-card;
	}

	.empty-card {
		display: flex;
		align-items: center;
		gap: 18px;
		padding: 22px;
	}

	.empty-text {
		flex: 1;
		min-width: 0;
	}

	.empty-title {
		font-size: 15px;
		font-weight: 600;
		color: var(--text-primary);
		margin-bottom: 3px;
	}

	.empty-hint {
		font-size: 12.5px;
		color: var(--text-secondary);
		line-height: 1.4;
	}

	.conn-card-icon {
		width: 56px;
		height: 56px;
		border-radius: 14px;
		box-shadow: var(--shadow-1);
		display: flex;
		align-items: center;
		justify-content: center;
		flex-shrink: 0;
	}

	.conn-card-icon.empty {
		font-family: 'New York', 'Iowan Old Style', Georgia, serif;
		font-style: italic;
		font-weight: 700;
		font-size: 26px;
		color: var(--text-secondary);
		background: var(--bg-secondary);
	}

	.btn {
		display: inline-flex;
		align-items: center;
		gap: 6px;
		padding: 8px 14px;
		border-radius: 8px;
		font-family: inherit;
		font-size: 13px;
		font-weight: 500;
		cursor: pointer;
		border: none;
		transition:
			background 120ms ease,
			transform 80ms ease,
			opacity 120ms ease;
		color: var(--text-primary);
		background: var(--bg-secondary);
		box-shadow: inset 0 0 0 0.5px var(--border-primary);
		white-space: nowrap;
	}

	.btn.primary {
		background: var(--accent);
		color: var(--text-on-color);
	}

	@container settings-card (max-width: 539px) {
		.empty-card {
			flex-direction: column;
			align-items: flex-start;
			gap: 12px;
			padding: 16px;
		}
	}
</style>
