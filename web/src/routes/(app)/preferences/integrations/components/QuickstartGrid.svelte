<script lang="ts">
	interface Props {
		onCopyInbox: () => void;
		onStartNotion: () => void;
	}

	let { onCopyInbox, onStartNotion }: Props = $props();
</script>

<div class="quickstart-stack">
	<a
		class="quickstart-card"
		href="https://chrome.google.com/webstore"
		target="_blank"
		rel="noopener"
	>
		<div class="mark browser">
			<svg viewBox="0 0 24 24" aria-hidden="true">
				<circle cx="12" cy="12" r="10" />
				<line x1="2" y1="12" x2="22" y2="12" />
				<path
					d="M12 2a15.3 15.3 0 0 1 4 10 15.3 15.3 0 0 1-4 10 15.3 15.3 0 0 1-4-10 15.3 15.3 0 0 1 4-10z"
				/>
			</svg>
		</div>
		<div class="meta">
			<div class="title">Install browser extension</div>
			<div class="sub">Clip articles in one click</div>
		</div>
		<div class="arrow">
			<svg viewBox="0 0 24 24" aria-hidden="true"><path d="M5 12h14M13 5l7 7-7 7" /></svg>
		</div>
	</a>
	<button type="button" class="quickstart-card" onclick={onCopyInbox}>
		<div class="mark email">
			<svg viewBox="0 0 24 24" aria-hidden="true">
				<rect x="2" y="4" width="20" height="16" rx="2" />
				<path d="M22 4L12 13 2 4" />
			</svg>
		</div>
		<div class="meta">
			<div class="title">Set up email forwarding</div>
			<div class="sub">Auto-capture newsletters</div>
		</div>
		<div class="arrow">
			<svg viewBox="0 0 24 24" aria-hidden="true"><path d="M5 12h14M13 5l7 7-7 7" /></svg>
		</div>
	</button>
	<button type="button" class="quickstart-card" onclick={onStartNotion}>
		<div class="mark notion">N</div>
		<div class="meta">
			<div class="title">Connect Notion</div>
			<div class="sub">Sync highlights to a database</div>
		</div>
		<div class="arrow">
			<svg viewBox="0 0 24 24" aria-hidden="true"><path d="M5 12h14M13 5l7 7-7 7" /></svg>
		</div>
	</button>
</div>

<style>
	.quickstart-stack {
		flex-shrink: 0;
		display: flex;
		flex-direction: column;
		gap: 8px;
		width: 320px;
	}

	.quickstart-card {
		display: flex;
		align-items: center;
		gap: 12px;
		padding: 12px 14px;
		border-radius: 12px;
		background: var(--int-metric-card-bg);
		box-shadow:
			0 0 0 0.5px var(--int-metric-card-border),
			var(--shadow-1);
		cursor: pointer;
		text-align: left;
		width: 100%;
		border: none;
		text-decoration: none;
		font-family: inherit;
		color: inherit;
	}

	.quickstart-card:hover {
		transform: translateY(-1px);
	}

	.mark {
		width: 32px;
		height: 32px;
		border-radius: 8px;
		flex-shrink: 0;
		display: flex;
		align-items: center;
		justify-content: center;
		color: var(--text-on-color);
		background: var(--int-ring-connected);
		font-weight: 700;
	}

	.mark svg {
		width: 18px;
		height: 18px;
		stroke: currentColor;
		fill: none;
		stroke-width: 1.7;
		stroke-linecap: round;
		stroke-linejoin: round;
	}

	.mark.email {
		background: var(--success);
	}

	.mark.notion {
		background: var(--int-brand-notion-bg);
		color: var(--int-brand-notion-text);
		box-shadow: inset 0 0 0 0.5px var(--border-primary);
		font-family: 'Times New Roman', serif;
		font-style: italic;
	}

	.meta {
		flex: 1;
		min-width: 0;
	}

	.title {
		font-size: 13.5px;
		font-weight: 600;
		letter-spacing: -0.01em;
		color: var(--int-metric-num);
		margin-bottom: 1px;
	}

	.sub {
		font-size: 11.5px;
		color: var(--int-metric-label);
		letter-spacing: -0.005em;
	}

	.arrow {
		width: 22px;
		height: 22px;
		border-radius: 50%;
		background: var(--int-status-coming-bg);
		color: var(--int-ring-connected);
		display: flex;
		align-items: center;
		justify-content: center;
		flex-shrink: 0;
	}

	.arrow svg {
		width: 12px;
		height: 12px;
		stroke: currentColor;
		fill: none;
		stroke-width: 2;
		stroke-linecap: round;
		stroke-linejoin: round;
	}
</style>
