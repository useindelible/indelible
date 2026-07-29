<script lang="ts">
	import type { AliasDestinationDto, EmailAliasResponse } from '$lib/api';
	import { formatIssued } from '../email-model';

	interface Props {
		dest: AliasDestinationDto;
		label: string;
		headline: string;
		address: string;
		primary: EmailAliasResponse | null;
		copied: boolean;
		onCopy: (key: string, text: string) => void;
		onOpenComposer: (destination: AliasDestinationDto) => void;
	}

	let { dest, label, headline, address, primary, copied, onCopy, onOpenComposer }: Props = $props();
</script>

<article class="envelope envelope-{dest}">
	<header class="envelope-head">
		<div class="envelope-tag envelope-tag-{dest}">
			<span class="tag-dot" aria-hidden="true"></span>
			<span class="tag-text">{label}</span>
		</div>
		<p class="envelope-headline">{headline}</p>
	</header>

	<section class="envelope-section">
		<div class="envelope-cap">Primary address</div>
		<div class="envelope-address-row">
			<div class="envelope-address">{address || '— issuing —'}</div>
			<button
				class="copy-btn"
				type="button"
				aria-label="Copy {dest} address"
				disabled={!address}
				onclick={() => onCopy(`primary-${dest}`, address)}
			>
				{#if copied}
					<svg viewBox="0 0 24 24" aria-hidden="true">
						<polyline points="20 6 9 17 4 12" />
					</svg>
					Copied
				{:else}
					<svg viewBox="0 0 24 24" aria-hidden="true">
						<rect x="9" y="9" width="13" height="13" rx="2" />
						<path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1" />
					</svg>
					Copy
				{/if}
			</button>
		</div>
		<div class="envelope-meta">
			<span class="envelope-since">
				{primary ? `Issued · ${formatIssued(primary.created_at)}` : 'Default address'}
			</span>
			<span class="envelope-sep" aria-hidden="true">·</span>
			<button
				class="envelope-rotate"
				type="button"
				aria-label="Create a new {dest} address"
				onclick={() => onOpenComposer(dest)}
			>
				<svg viewBox="0 0 24 24" aria-hidden="true">
					<path d="M12 5v14M5 12h14" />
				</svg>
				New {dest === 'feed' ? 'Feed' : 'Library'} address
			</button>
		</div>
	</section>
</article>

<style>
	.envelope {
		position: relative;
		background: var(--envelope-bg);
		border-radius: var(--radius-lg);
		padding: 24px 26px;
		box-shadow: var(--envelope-shadow);
		display: flex;
		flex-direction: column;
		gap: 0;
		transition:
			box-shadow 180ms ease,
			transform 180ms ease;
	}

	.envelope::before {
		content: '';
		position: absolute;
		left: 14px;
		right: 14px;
		top: 0;
		height: 6px;
		background-image: radial-gradient(
			circle at 6px center,
			var(--perf-dot) 0 1.5px,
			transparent 1.5px
		);
		background-size: 12px 6px;
		background-repeat: repeat-x;
		opacity: 0.7;
		pointer-events: none;
	}

	.envelope:hover {
		box-shadow: var(--envelope-shadow-hover);
		transform: translateY(-1px);
	}

	.envelope-head {
		display: flex;
		flex-direction: column;
		gap: 6px;
	}

	.envelope-tag {
		display: inline-flex;
		align-items: center;
		gap: 8px;
	}

	.tag-dot {
		width: 7px;
		height: 7px;
		border-radius: 50%;
		background: var(--accent);
	}

	.envelope-tag-library .tag-dot {
		background: var(--success);
	}

	.tag-text {
		font-family: var(--font-mono);
		font-size: 10px;
		font-weight: 600;
		letter-spacing: 0.18em;
		text-transform: uppercase;
		color: var(--text-secondary);
	}

	.envelope-headline {
		font-size: 13px;
		color: var(--text-secondary);
		line-height: 1.45;
		letter-spacing: -0.005em;
		margin: 4px 0 0;
	}

	.envelope-section {
		margin-top: 18px;
		padding-top: 16px;
		border-top: 0.5px solid var(--border-primary);
	}

	.envelope-cap {
		font-family: var(--font-mono);
		font-size: 9px;
		font-weight: 600;
		letter-spacing: 0.2em;
		text-transform: uppercase;
		color: var(--text-tertiary);
		display: inline-flex;
		align-items: baseline;
		gap: 6px;
		margin-bottom: 10px;
	}

	.envelope-address-row,
	.envelope-meta {
		display: flex;
		align-items: center;
		gap: 12px;
		justify-content: space-between;
	}

	.envelope-address {
		flex: 1;
		min-width: 0;
		font-family: var(--font-mono);
		font-size: 14px;
		color: var(--text-primary);
		letter-spacing: 0;
		user-select: all;
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}

	.envelope-meta {
		display: inline-flex;
		gap: 8px;
		margin-top: 12px;
		font-family: var(--font-body);
		font-size: 11.5px;
		letter-spacing: -0.005em;
	}

	.envelope-since {
		color: var(--text-tertiary);
	}

	.envelope-sep {
		color: var(--text-quaternary);
	}

	.copy-btn {
		display: inline-flex;
		align-items: center;
		gap: 6px;
		padding: 6px 11px;
		border-radius: 6px;
		border: none;
		background: transparent;
		color: var(--text-secondary);
		box-shadow: inset 0 0 0 0.5px var(--border-primary);
		font-size: 11.5px;
		font-weight: 500;
		cursor: pointer;
		letter-spacing: -0.005em;
		flex-shrink: 0;
		transition:
			background 140ms ease,
			color 140ms ease,
			box-shadow 140ms ease;
	}

	.copy-btn:hover:not(:disabled) {
		background: var(--accent-tint);
		color: var(--accent-strong);
		box-shadow: inset 0 0 0 0.5px var(--accent-line);
	}

	.copy-btn:disabled {
		opacity: 0.45;
		cursor: not-allowed;
	}

	.copy-btn svg {
		width: 12px;
		height: 12px;
		stroke: currentColor;
		fill: none;
		stroke-width: 1.75;
		stroke-linecap: round;
		stroke-linejoin: round;
	}

	.envelope-rotate {
		display: inline-flex;
		align-items: center;
		gap: 5px;
		padding: 3px 9px 3px 7px;
		border-radius: var(--radius-full);
		background: transparent;
		font-family: var(--font-body);
		font-size: 11.5px;
		font-weight: 500;
		color: var(--accent);
		border: 1px dashed var(--accent-line);
		cursor: pointer;
		letter-spacing: -0.005em;
		transition:
			background 140ms ease,
			border-color 140ms ease,
			color 140ms ease;
	}

	.envelope-rotate:hover {
		background: var(--accent-tint);
		border-color: var(--accent);
		color: var(--accent-strong);
	}

	.envelope-rotate svg {
		width: 11px;
		height: 11px;
		stroke: currentColor;
		fill: none;
		stroke-width: 2;
		stroke-linecap: round;
		stroke-linejoin: round;
	}
</style>
