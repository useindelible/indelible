<script lang="ts">
	import { browser } from '$app/environment';
	import {
		getReaderPreferences,
		type ReaderTheme,
		type ReaderTypeface
	} from '$lib/stores/reader-preferences.svelte';
	import { t, type MessageKey } from '$lib/i18n';

	interface Props {
		anchorEl: HTMLElement;
		onClose: () => void;
	}

	let { anchorEl, onClose }: Props = $props();

	const prefs = getReaderPreferences();
	let popoverEl = $state<HTMLDivElement | undefined>(undefined);
	let popoverTop = $state(0);
	let popoverRight = $state(0);

	// 280px panel + 16px margins; right-anchoring to the Aa button alone would
	// push the panel past the left edge on narrow viewports.
	const POPOVER_WIDTH = 280;
	const VIEWPORT_MARGIN = 12;

	$effect(() => {
		if (anchorEl) {
			const rect = anchorEl.getBoundingClientRect();
			popoverTop = rect.bottom + 6;
			const maxRight = window.innerWidth - POPOVER_WIDTH - VIEWPORT_MARGIN;
			popoverRight = Math.max(VIEWPORT_MARGIN, Math.min(window.innerWidth - rect.right, maxRight));
		}
	});

	const themes: { value: ReaderTheme; labelKey: MessageKey }[] = [
		{ value: 'light', labelKey: 'reader_theme_light' },
		{ value: 'dark', labelKey: 'reader_theme_dark' },
		{ value: 'sepia', labelKey: 'reader_theme_sepia' },
		{ value: 'auto', labelKey: 'reader_theme_auto' }
	];

	const typefaces: { value: ReaderTypeface; label: string; preview: string }[] = [
		{ value: 'serif', label: 'Lora', preview: "'Lora', Georgia, serif" },
		{ value: 'sans', label: 'Geist', preview: "'Geist', -apple-system, sans-serif" },
		{ value: 'mono', label: 'Mono', preview: "'Geist Mono', 'SF Mono', monospace" }
	];

	function handlePointerDown(e: PointerEvent) {
		if (!popoverEl || !anchorEl) return;
		const target = e.target as Node;
		if (!popoverEl.contains(target) && !anchorEl.contains(target)) {
			onClose();
		}
	}

	function handleKeydown(e: KeyboardEvent) {
		if (e.key === 'Escape') {
			e.preventDefault();
			e.stopPropagation();
			onClose();
		}
	}

	$effect(() => {
		if (browser) {
			document.addEventListener('pointerdown', handlePointerDown);
			return () => {
				document.removeEventListener('pointerdown', handlePointerDown);
			};
		}
	});
</script>

<svelte:window onkeydown={handleKeydown} />

<div
	class="typography-popover"
	bind:this={popoverEl}
	style:top="{popoverTop}px"
	style:right="{popoverRight}px"
>
	<!-- Theme -->
	<div class="control-section" role="group" aria-label={$t('reader_theme')}>
		<span class="control-label">{$t('reader_theme')}</span>
		<div class="button-row">
			{#each themes as theme (theme.value)}
				<button
					type="button"
					class="theme-btn"
					class:active={prefs.theme === theme.value}
					aria-pressed={prefs.theme === theme.value}
					onclick={() => {
						prefs.theme = theme.value;
					}}
				>
					{$t(theme.labelKey)}
				</button>
			{/each}
		</div>
	</div>

	<!-- Typeface -->
	<div class="control-section" role="group" aria-label={$t('reader_typeface')}>
		<span class="control-label">{$t('reader_typeface')}</span>
		<div class="button-row typeface-row">
			{#each typefaces as tf (tf.value)}
				<button
					type="button"
					class="typeface-btn"
					class:active={prefs.typeface === tf.value}
					aria-pressed={prefs.typeface === tf.value}
					style:font-family={tf.preview}
					onclick={() => {
						prefs.typeface = tf.value;
					}}
				>
					{tf.label}
				</button>
			{/each}
		</div>
	</div>

	<!-- Font Size -->
	<div class="control-section">
		<div class="control-header">
			<label class="control-label" for="font-size-slider">{$t('reader_font_size')}</label>
			<span class="control-value">{prefs.fontSize}px</span>
		</div>
		<input
			id="font-size-slider"
			type="range"
			min="14"
			max="28"
			step="2"
			value={prefs.fontSize}
			oninput={(e) => {
				prefs.fontSize = Number((e.target as HTMLInputElement).value);
			}}
			aria-label={$t('reader_font_size')}
			aria-valuemin={14}
			aria-valuemax={28}
			aria-valuenow={prefs.fontSize}
		/>
	</div>

	<!-- Line Height -->
	<div class="control-section">
		<div class="control-header">
			<label class="control-label" for="line-height-slider">{$t('reader_line_height')}</label>
			<span class="control-value">{prefs.lineHeight.toFixed(1)}</span>
		</div>
		<input
			id="line-height-slider"
			type="range"
			min="1.4"
			max="2.2"
			step="0.1"
			value={prefs.lineHeight}
			oninput={(e) => {
				prefs.lineHeight = Number((e.target as HTMLInputElement).value);
			}}
			aria-label={$t('reader_line_height')}
			aria-valuemin={1.4}
			aria-valuemax={2.2}
			aria-valuenow={prefs.lineHeight}
		/>
	</div>

	<!-- Content Width -->
	<div class="control-section">
		<div class="control-header">
			<label class="control-label" for="content-width-slider">{$t('reader_content_width')}</label>
			<span class="control-value">{prefs.contentWidth}px</span>
		</div>
		<input
			id="content-width-slider"
			type="range"
			min="480"
			max="840"
			step="40"
			value={prefs.contentWidth}
			oninput={(e) => {
				prefs.contentWidth = Number((e.target as HTMLInputElement).value);
			}}
			aria-label={$t('reader_content_width')}
			aria-valuemin={480}
			aria-valuemax={840}
			aria-valuenow={prefs.contentWidth}
		/>
	</div>

	<!-- Paragraph Spacing -->
	<div class="control-section">
		<div class="control-header">
			<label class="control-label" for="paragraph-spacing-slider"
				>{$t('reader_paragraph_spacing')}</label
			>
			<span class="control-value">{prefs.paragraphSpacing.toFixed(1)}em</span>
		</div>
		<input
			id="paragraph-spacing-slider"
			type="range"
			min="0.6"
			max="2.0"
			step="0.1"
			value={prefs.paragraphSpacing}
			oninput={(e) => {
				prefs.paragraphSpacing = Number((e.target as HTMLInputElement).value);
			}}
			aria-label={$t('reader_paragraph_spacing')}
			aria-valuemin={0.6}
			aria-valuemax={2.0}
			aria-valuenow={prefs.paragraphSpacing}
		/>
	</div>

	<!-- Text Alignment -->
	<div class="control-section" role="group" aria-label={$t('reader_text_alignment')}>
		<span class="control-label">{$t('reader_text_alignment')}</span>
		<div class="button-row align-row">
			<button
				type="button"
				class="align-btn"
				class:active={prefs.textAlign === 'left'}
				aria-pressed={prefs.textAlign === 'left'}
				onclick={() => {
					prefs.textAlign = 'left';
				}}
			>
				<svg
					viewBox="0 0 24 24"
					fill="none"
					stroke="currentColor"
					stroke-width="1.5"
					stroke-linecap="round"
				>
					<line x1="3" y1="6" x2="21" y2="6" />
					<line x1="3" y1="10" x2="15" y2="10" />
					<line x1="3" y1="14" x2="19" y2="14" />
					<line x1="3" y1="18" x2="13" y2="18" />
				</svg>
				{$t('reader_align_left')}
			</button>
			<button
				type="button"
				class="align-btn"
				class:active={prefs.textAlign === 'justify'}
				aria-pressed={prefs.textAlign === 'justify'}
				onclick={() => {
					prefs.textAlign = 'justify';
				}}
			>
				<svg
					viewBox="0 0 24 24"
					fill="none"
					stroke="currentColor"
					stroke-width="1.5"
					stroke-linecap="round"
				>
					<line x1="3" y1="6" x2="21" y2="6" />
					<line x1="3" y1="10" x2="21" y2="10" />
					<line x1="3" y1="14" x2="21" y2="14" />
					<line x1="3" y1="18" x2="21" y2="18" />
				</svg>
				{$t('reader_align_justified')}
			</button>
		</div>
	</div>
</div>

<style>
	.typography-popover {
		position: fixed;
		width: 280px;
		background: var(--bg-elevated);
		border-radius: 12px;
		box-shadow: var(--shadow-3);
		padding: 16px;
		display: flex;
		flex-direction: column;
		gap: 16px;
		z-index: 30;
		border: 0.5px solid var(--border-primary);
	}

	.control-section {
		display: flex;
		flex-direction: column;
		gap: 8px;
	}

	.control-header {
		display: flex;
		justify-content: space-between;
		align-items: baseline;
	}

	.control-label {
		font-size: 11px;
		font-weight: 600;
		letter-spacing: 0.06em;
		text-transform: uppercase;
		color: var(--text-tertiary);
		font-family: var(--font-sans);
		line-height: 1.2;
	}

	.control-value {
		font-size: 12px;
		font-weight: 500;
		color: var(--text-secondary);
		font-family: var(--font-sans);
	}

	.button-row {
		display: flex;
		gap: 4px;
	}

	.theme-btn {
		flex: 1;
		padding: 6px 0;
		border-radius: 7px;
		border: 1px solid var(--border-primary);
		background: transparent;
		font-size: 12px;
		font-weight: 500;
		color: var(--text-secondary);
		cursor: pointer;
		font-family: var(--font-sans);
		transition: all 120ms ease;
	}

	.theme-btn:hover {
		border-color: var(--border-secondary);
		color: var(--text-primary);
	}

	.theme-btn.active {
		background: var(--fill-selected);
		border-color: var(--accent);
		color: var(--accent);
	}

	.typeface-btn {
		flex: 1;
		padding: 8px 0;
		border-radius: 7px;
		border: 1px solid var(--border-primary);
		background: transparent;
		font-size: 13px;
		font-weight: 500;
		color: var(--text-secondary);
		cursor: pointer;
		transition: all 120ms ease;
	}

	.typeface-btn:hover {
		border-color: var(--border-secondary);
		color: var(--text-primary);
	}

	.typeface-btn.active {
		background: var(--fill-selected);
		border-color: var(--accent);
		color: var(--accent);
	}

	.align-btn {
		flex: 1;
		display: flex;
		align-items: center;
		justify-content: center;
		gap: 6px;
		padding: 7px 0;
		border-radius: 7px;
		border: 1px solid var(--border-primary);
		background: transparent;
		font-size: 12px;
		font-weight: 500;
		color: var(--text-secondary);
		cursor: pointer;
		font-family: var(--font-sans);
		transition: all 120ms ease;
	}

	.align-btn :global(svg) {
		width: 14px;
		height: 14px;
	}

	.align-btn:hover {
		border-color: var(--border-secondary);
		color: var(--text-primary);
	}

	.align-btn.active {
		background: var(--fill-selected);
		border-color: var(--accent);
		color: var(--accent);
	}

	input[type='range'] {
		-webkit-appearance: none;
		appearance: none;
		width: 100%;
		height: 4px;
		border-radius: 2px;
		background: var(--seg-bg);
		outline: none;
	}

	input[type='range']::-webkit-slider-thumb {
		-webkit-appearance: none;
		appearance: none;
		width: 16px;
		height: 16px;
		border-radius: 50%;
		background: var(--accent);
		cursor: pointer;
		border: 2px solid var(--bg-elevated);
		box-shadow: 0 1px 3px rgba(0, 0, 0, 0.15);
	}

	input[type='range']::-moz-range-thumb {
		width: 16px;
		height: 16px;
		border-radius: 50%;
		background: var(--accent);
		cursor: pointer;
		border: 2px solid var(--bg-elevated);
		box-shadow: 0 1px 3px rgba(0, 0, 0, 0.15);
	}
</style>
