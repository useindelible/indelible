<script lang="ts">
	import SettingsGroup from '$lib/components/settings/SettingsGroup.svelte';
	import type { ReaderFontFamilyDto, ReaderFontSizeDto, ReaderLineHeightDto } from '$lib/api';
	import { bumpFontSize } from '../reading-appearance-model';

	interface Props {
		fontFamily: ReaderFontFamilyDto;
		fontSize: ReaderFontSizeDto;
		lineHeight: ReaderLineHeightDto;
		fontSizeLabel: Record<ReaderFontSizeDto, string>;
		onFontFamilyChange: (value: ReaderFontFamilyDto) => void;
		onFontSizeChange: (value: ReaderFontSizeDto) => void;
		onLineHeightChange: (value: ReaderLineHeightDto) => void;
	}

	let {
		fontFamily,
		fontSize,
		lineHeight,
		fontSizeLabel,
		onFontFamilyChange,
		onFontSizeChange,
		onLineHeightChange
	}: Props = $props();
</script>

<SettingsGroup title="Reader defaults" meta="Live-applied to the preview above">
	<div class="group-card">
		<div class="row with-stack">
			<div class="label-block">
				<div class="label">Font family</div>
				<div class="hint">The typeface used for article body text in the reader.</div>
			</div>
			<div class="tile-row" role="radiogroup" aria-label="Font family">
				<button
					type="button"
					class="font-tile"
					data-font="serif"
					class:selected={fontFamily === 'serif'}
					role="radio"
					aria-checked={fontFamily === 'serif'}
					aria-label="Serif"
					onclick={() => onFontFamilyChange('serif')}
				>
					<div class="glyph">Aa</div>
					<div class="tag">Serif</div>
				</button>
				<button
					type="button"
					class="font-tile"
					data-font="sans"
					class:selected={fontFamily === 'sans'}
					role="radio"
					aria-checked={fontFamily === 'sans'}
					aria-label="Sans"
					onclick={() => onFontFamilyChange('sans')}
				>
					<div class="glyph">Aa</div>
					<div class="tag">Sans</div>
				</button>
				<button
					type="button"
					class="font-tile"
					data-font="mono"
					class:selected={fontFamily === 'mono'}
					role="radio"
					aria-checked={fontFamily === 'mono'}
					aria-label="Mono"
					onclick={() => onFontFamilyChange('mono')}
				>
					<div class="glyph">Aa</div>
					<div class="tag">Mono</div>
				</button>
			</div>
		</div>
		<div class="row">
			<div class="label-block">
				<div class="label">Font size</div>
				<div class="hint">
					Body text size in the reader. Headings and other elements scale with it.
				</div>
			</div>
			<div class="stepper">
				<button
					type="button"
					class="stepper-btn"
					onclick={() => onFontSizeChange(bumpFontSize(fontSize, -1))}
					disabled={fontSize === 'small'}
					aria-label="Decrease size"
				>
					<svg viewBox="0 0 24 24" aria-hidden="true"><path d="M5 12h14" /></svg>
				</button>
				<span class="stepper-value">{fontSizeLabel[fontSize]}</span>
				<button
					type="button"
					class="stepper-btn"
					onclick={() => onFontSizeChange(bumpFontSize(fontSize, 1))}
					disabled={fontSize === 'large'}
					aria-label="Increase size"
				>
					<svg viewBox="0 0 24 24" aria-hidden="true"><path d="M12 5v14M5 12h14" /></svg>
				</button>
			</div>
		</div>
		<div class="row">
			<div class="label-block">
				<div class="label">Line height</div>
				<div class="hint">Compact is dense; relaxed is airy.</div>
			</div>
			<div class="segmented" role="radiogroup" aria-label="Line height">
				<button
					type="button"
					class="seg"
					class:active={lineHeight === 'compact'}
					role="radio"
					aria-checked={lineHeight === 'compact'}
					onclick={() => onLineHeightChange('compact')}
				>
					<span class="lh-bar lh-tight"><span></span><span></span><span></span></span>
					Compact
				</button>
				<button
					type="button"
					class="seg"
					class:active={lineHeight === 'relaxed'}
					role="radio"
					aria-checked={lineHeight === 'relaxed'}
					onclick={() => onLineHeightChange('relaxed')}
				>
					<span class="lh-bar lh-loose"><span></span><span></span><span></span></span>
					Relaxed
				</button>
			</div>
		</div>
	</div>
</SettingsGroup>

<style>
	.group-card {
		background: var(--card-bg);
		border-radius: 14px;
		overflow: hidden;
		box-shadow: inset 0 0 0 0.5px var(--border-primary);
		container-type: inline-size;
		container-name: settings-card;
	}

	.row {
		display: flex;
		align-items: center;
		gap: 16px;
		padding: 14px 18px;
		min-height: 52px;
		border-top: 0.5px solid var(--border-hairline);
	}

	.row:first-child {
		border-top: none;
	}

	.row.with-stack {
		align-items: flex-start;
		flex-direction: column;
		gap: 14px;
		padding: 18px;
	}

	.label-block {
		flex: 1;
		min-width: 0;
	}

	.with-stack .label-block {
		width: 100%;
	}

	.label {
		font-family: var(--font-sans);
		font-size: 13px;
		font-weight: 500;
		color: var(--text-primary);
		margin-bottom: 2px;
	}

	.hint {
		font-family: var(--font-sans);
		font-size: 12px;
		color: var(--text-secondary);
		line-height: 1.4;
	}

	.tile-row {
		display: grid;
		grid-template-columns: repeat(3, 1fr);
		gap: 10px;
		width: 100%;
	}

	.font-tile {
		background: var(--bg-elevated);
		border-radius: 12px;
		padding: 18px 16px 12px;
		box-shadow: inset 0 0 0 0.5px var(--border-primary);
		cursor: pointer;
		text-align: center;
		transition:
			box-shadow 140ms,
			transform 140ms;
		display: flex;
		flex-direction: column;
		gap: 6px;
		align-items: center;
		border: none;
	}

	.font-tile:hover {
		transform: translateY(-1px);
	}

	.font-tile.selected {
		box-shadow:
			inset 0 0 0 1.5px var(--accent),
			0 0 0 4px var(--accent-soft);
	}

	.glyph {
		font-size: 32px;
		line-height: 1;
		font-weight: 500;
		color: var(--text-primary);
	}

	.font-tile[data-font='serif'] .glyph {
		font-family: 'New York', 'Iowan Old Style', Georgia, 'Times New Roman', serif;
	}
	.font-tile[data-font='sans'] .glyph {
		font-family: -apple-system, 'SF Pro Display', 'Helvetica Neue', sans-serif;
	}
	.font-tile[data-font='mono'] .glyph {
		font-family: 'SF Mono', 'Fira Code', Menlo, ui-monospace, monospace;
		font-size: 28px;
	}

	.tag {
		font-family: var(--font-sans);
		font-size: 11px;
		font-weight: 600;
		color: var(--text-tertiary);
		letter-spacing: 0.06em;
		text-transform: uppercase;
	}

	.font-tile.selected .tag {
		color: var(--accent);
	}

	.stepper,
	.segmented {
		display: inline-flex;
		align-items: center;
		background: var(--bg-elevated);
		border-radius: 10px;
		box-shadow: inset 0 0 0 0.5px var(--border-primary);
		padding: 3px;
		gap: 2px;
	}

	.stepper-btn {
		width: 30px;
		height: 30px;
		border-radius: 8px;
		color: var(--text-primary);
		display: inline-flex;
		align-items: center;
		justify-content: center;
		transition: background 120ms;
		background: transparent;
		border: none;
		cursor: pointer;
	}

	.stepper-btn:hover:not(:disabled) {
		background: var(--fill-hover);
	}

	.stepper-btn:disabled {
		opacity: 0.35;
		cursor: default;
	}

	.stepper-btn svg {
		width: 12px;
		height: 12px;
		stroke: currentColor;
		fill: none;
		stroke-width: 2;
		stroke-linecap: round;
		stroke-linejoin: round;
	}

	.stepper-value {
		min-width: 72px;
		text-align: center;
		font-variant-numeric: tabular-nums;
		font-family: var(--font-sans);
		font-size: 13px;
		font-weight: 500;
		color: var(--text-primary);
		padding: 0 6px;
	}

	.seg {
		padding: 6px 14px;
		border-radius: 7px;
		font-family: var(--font-sans);
		font-size: 12.5px;
		font-weight: 500;
		color: var(--text-secondary);
		transition:
			background 140ms,
			color 140ms,
			box-shadow 140ms;
		display: inline-flex;
		align-items: center;
		gap: 5px;
		background: transparent;
		border: none;
		cursor: pointer;
	}

	.seg.active {
		background: var(--bg-elevated);
		color: var(--text-primary);
		box-shadow: var(--shadow-1);
	}

	.lh-bar {
		display: inline-flex;
		flex-direction: column;
		gap: 2px;
		margin-right: 3px;
	}

	.lh-bar span {
		width: 12px;
		height: 1.5px;
		background: currentColor;
		opacity: 0.85;
		border-radius: 1px;
	}

	.lh-tight span + span {
		margin-top: 1px;
	}

	.lh-loose span + span {
		margin-top: 4px;
	}

	/* The line-height segmented control is wide; wrap it under its label
	   instead of crushing the label. The stepper row stays inline. */
	@container settings-card (max-width: 539px) {
		.row:has(.segmented) {
			flex-wrap: wrap;
		}

		.row:has(.segmented) .label-block {
			flex: 1 1 100%;
		}
	}
</style>
