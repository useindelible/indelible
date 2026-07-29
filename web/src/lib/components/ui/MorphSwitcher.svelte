<script lang="ts">
	interface Option {
		value: string;
		label: string;
		count?: number | null;
	}

	interface Props {
		options: readonly Option[];
		value?: string;
		onchange?: (value: string) => void;
		size?: 'md' | 'sm';
	}

	let { options, value = $bindable(''), onchange, size = 'md' }: Props = $props();

	function select(v: string) {
		value = v;
		onchange?.(v);
	}
</script>

{#snippet icon(name: string)}
	<svg viewBox="0 0 24 24" aria-hidden="true">
		{#if name === 'inbox'}
			<polyline points="22 12 16 12 14 15 10 15 8 12 2 12" />
			<path
				d="M5.45 5.11 2 12v6a2 2 0 0 0 2 2h16a2 2 0 0 0 2-2v-6l-3.45-6.89A2 2 0 0 0 16.76 4H7.24a2 2 0 0 0-1.79 1.11z"
			/>
		{:else if name === 'later'}
			<circle cx="12" cy="12" r="9" />
			<polyline points="12 7 12 12 15.5 13.5" />
		{:else if name === 'archive'}
			<polyline points="21 8 21 21 3 21 3 8" />
			<rect x="1" y="3" width="22" height="5" />
			<line x1="10" y1="12" x2="14" y2="12" />
		{:else if name === 'info'}
			<circle cx="12" cy="12" r="9" />
			<line x1="12" y1="11" x2="12" y2="16" />
			<circle cx="12" cy="8" r="0.5" fill="currentColor" />
		{:else if name === 'notebook'}
			<path d="M4 19.5A2.5 2.5 0 0 1 6.5 17H20" />
			<path d="M6.5 2H20v20H6.5A2.5 2.5 0 0 1 4 19.5v-15A2.5 2.5 0 0 1 6.5 2z" />
		{:else if name === 'chat'}
			<path
				d="M21 11.5a8.38 8.38 0 0 1-.9 3.8 8.5 8.5 0 0 1-7.6 4.7 8.38 8.38 0 0 1-3.8-.9L3 21l1.9-5.7a8.38 8.38 0 0 1-.9-3.8 8.5 8.5 0 0 1 4.7-7.6 8.38 8.38 0 0 1 3.8-.9h.5a8.48 8.48 0 0 1 8 8v.5z"
			/>
		{:else if name === 'unseen'}
			<circle cx="12" cy="12" r="9" />
			<circle cx="12" cy="12" r="1.5" fill="currentColor" />
		{:else if name === 'seen'}
			<path d="M1 12s4-8 11-8 11 8 11 8-4 8-11 8-11-8-11-8z" />
			<circle cx="12" cy="12" r="3" />
		{/if}
	</svg>
{/snippet}

<div class="morph" class:sm={size === 'sm'} role="tablist">
	{#each options as option (option.value)}
		<button
			type="button"
			role="tab"
			class="morph-btn"
			class:on={value === option.value}
			aria-selected={value === option.value}
			title={option.label}
			onclick={() => select(option.value)}
		>
			{@render icon(option.value)}
			<span class="m-rest">
				<span class="m-inner">
					<span class="m-label">{option.label}</span>
					{#if option.count != null}
						<span class="m-count">{option.count}</span>
					{/if}
				</span>
			</span>
		</button>
	{/each}
</div>

<style>
	.morph {
		display: inline-flex;
		align-items: center;
		gap: 6px;
	}

	.morph-btn {
		border: none;
		font-family: var(--font-sans);
		display: inline-flex;
		align-items: center;
		height: 34px;
		padding: 0 9px;
		border-radius: var(--radius-full);
		background: var(--fill-hover);
		color: var(--text-secondary);
		cursor: pointer;
		overflow: hidden;
		white-space: nowrap;
		transition:
			background 380ms ease,
			color 380ms ease,
			box-shadow 380ms ease,
			padding 480ms cubic-bezier(0.25, 1, 0.3, 1);
	}

	.morph-btn:hover {
		background: var(--seg-bg);
		color: var(--text-primary);
	}

	.morph-btn svg {
		width: 16px;
		height: 16px;
		stroke: currentColor;
		fill: none;
		stroke-width: 1.7;
		stroke-linecap: round;
		stroke-linejoin: round;
		flex-shrink: 0;
	}

	/* Width animates via grid 0fr -> 1fr so expansion and collapse track real
	   content width symmetrically; max-width tweening overshoots horizontally. */
	.m-rest {
		display: grid;
		grid-template-columns: 0fr;
		transition: grid-template-columns 480ms cubic-bezier(0.25, 1, 0.3, 1);
	}

	.m-inner {
		display: flex;
		align-items: center;
		min-width: 0;
		overflow: hidden;
		opacity: 0;
		transition: opacity 320ms ease;
	}

	.m-label {
		margin-left: 7px;
		font-size: 13px;
		font-weight: 600;
		letter-spacing: -0.01em;
	}

	.m-count {
		margin-left: 7px;
		font-size: 10.5px;
		font-weight: 700;
		font-variant-numeric: tabular-nums;
		background: var(--fill-on-accent);
		border-radius: var(--radius-full);
		padding: 1px 7px;
		line-height: 1.5;
	}

	.morph-btn.on {
		background: var(--accent);
		color: var(--text-on-color);
		padding: 0 14px 0 11px;
		box-shadow: 0 2px 12px var(--accent-glow);
		cursor: default;
	}

	.morph-btn.on .m-rest {
		grid-template-columns: 1fr;
	}

	.morph-btn.on .m-inner {
		opacity: 1;
	}

	.morph-btn:focus-visible {
		outline: 2px solid var(--accent);
		outline-offset: 1px;
	}

	/* Panel-scale variant */
	.morph.sm {
		gap: 2px;
	}

	.morph.sm .morph-btn {
		height: 30px;
		padding: 0 7px;
	}

	.morph.sm .morph-btn.on {
		padding: 0 12px 0 9px;
	}

	.morph.sm .morph-btn svg {
		width: 15px;
		height: 15px;
	}

	.morph.sm .m-label {
		font-size: 12.5px;
	}
</style>
