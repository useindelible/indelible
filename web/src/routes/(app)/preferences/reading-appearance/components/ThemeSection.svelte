<script lang="ts">
	import SettingsGroup from '$lib/components/settings/SettingsGroup.svelte';
	import type { AccentColorDto, ThemeDto } from '$lib/api';

	interface Props {
		theme: ThemeDto;
		accentColor: AccentColorDto;
		accentSwatches: { value: AccentColorDto; label: string }[];
		onThemeChange: (theme: ThemeDto) => void;
		onAccentColorChange: (accentColor: AccentColorDto) => void;
	}

	let { theme, accentColor, accentSwatches, onThemeChange, onAccentColorChange }: Props = $props();
</script>

<SettingsGroup title="Theme" meta="Appearance + accent colour for the whole app">
	<div class="group-card">
		<div class="row">
			<div class="label-block">
				<div class="label">Appearance</div>
				<div class="hint">
					Light, dark, or follow your system. The reader preview above stays in sync.
				</div>
			</div>
			<div class="pill-group" role="tablist" aria-label="Theme">
				<button
					type="button"
					class="pill"
					class:active={theme === 'light'}
					role="tab"
					aria-selected={theme === 'light'}
					onclick={() => onThemeChange('light')}
				>
					<svg viewBox="0 0 24 24" aria-hidden="true">
						<circle cx="12" cy="12" r="4" />
						<path
							d="M12 3v2M12 19v2M3 12h2M19 12h2M5.6 5.6l1.4 1.4M17 17l1.4 1.4M5.6 18.4l1.4-1.4M17 7l1.4-1.4"
						/>
					</svg>
					Light
				</button>
				<button
					type="button"
					class="pill"
					class:active={theme === 'dark'}
					role="tab"
					aria-selected={theme === 'dark'}
					onclick={() => onThemeChange('dark')}
				>
					<svg viewBox="0 0 24 24" aria-hidden="true">
						<path d="M21 12.79A9 9 0 1 1 11.21 3 7 7 0 0 0 21 12.79z" />
					</svg>
					Dark
				</button>
				<button
					type="button"
					class="pill"
					class:active={theme === 'system'}
					role="tab"
					aria-selected={theme === 'system'}
					onclick={() => onThemeChange('system')}
				>
					<svg viewBox="0 0 24 24" aria-hidden="true">
						<rect x="3" y="4" width="18" height="13" rx="2" />
						<path d="M8 21h8M12 17v4" />
					</svg>
					System
				</button>
			</div>
		</div>
		<div class="row">
			<div class="label-block">
				<div class="label">Accent colour</div>
				<div class="hint">
					Used for buttons, links, and the SavePill. Live-applied to the reader preview link.
				</div>
			</div>
			<div class="swatch-row" role="radiogroup" aria-label="Accent colour">
				{#each accentSwatches as color (color.value)}
					<button
						type="button"
						class="swatch"
						data-color={color.value}
						class:selected={accentColor === color.value}
						role="radio"
						aria-checked={accentColor === color.value}
						aria-label={color.label}
						onclick={() => onAccentColorChange(color.value)}
					>
						<span class="check">
							<svg viewBox="0 0 24 24" aria-hidden="true"><path d="M5 12l4 4 10-10" /></svg>
						</span>
					</button>
				{/each}
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

	.label-block {
		flex: 1;
		min-width: 0;
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

	.pill-group {
		display: inline-flex;
		background: var(--bg-secondary);
		padding: 3px;
		border-radius: 11px;
		box-shadow: inset 0 0 0 0.5px var(--border-primary);
	}

	.pill {
		padding: 7px 14px;
		border-radius: 8px;
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
		gap: 6px;
		background: transparent;
		border: none;
		cursor: pointer;
	}

	.pill svg {
		width: 13px;
		height: 13px;
		stroke: currentColor;
		fill: none;
		stroke-width: 1.7;
		stroke-linecap: round;
		stroke-linejoin: round;
	}

	.pill.active {
		background: var(--bg-elevated);
		color: var(--text-primary);
		box-shadow: var(--shadow-1);
	}

	.swatch-row {
		display: inline-flex;
		gap: 10px;
	}

	.swatch {
		width: 30px;
		height: 30px;
		border-radius: 50%;
		position: relative;
		cursor: pointer;
		transition: transform 140ms;
		box-shadow: var(--shadow-1);
		border: none;
	}

	.swatch:hover {
		transform: scale(1.06);
	}

	.swatch::after {
		content: '';
		position: absolute;
		inset: -5px;
		border-radius: 50%;
		border: 1.5px solid currentColor;
		color: var(--text-primary);
		opacity: 0;
		transition: opacity 140ms;
	}

	.swatch.selected::after,
	.swatch.selected .check {
		opacity: 1;
	}

	.check {
		position: absolute;
		inset: 0;
		display: flex;
		align-items: center;
		justify-content: center;
		color: var(--text-on-color);
		opacity: 0;
		transition: opacity 140ms;
	}

	.check svg {
		width: 13px;
		height: 13px;
		stroke: currentColor;
		fill: none;
		stroke-width: 2.4;
		stroke-linecap: round;
		stroke-linejoin: round;
	}

	.swatch[data-color='blue'] {
		background: var(--accent-swatch-blue);
	}
	.swatch[data-color='green'] {
		background: var(--accent-swatch-green);
	}
	.swatch[data-color='rose'] {
		background: var(--accent-swatch-rose);
	}
	.swatch[data-color='orange'] {
		background: var(--accent-swatch-orange);
	}

	/* Both controls here are wide; below this the label shares the row as a
	   crushed sliver — wrap so the control drops under the label. */
	@container settings-card (max-width: 539px) {
		.row {
			flex-wrap: wrap;
		}

		.label-block {
			flex: 1 1 100%;
		}
	}
</style>
