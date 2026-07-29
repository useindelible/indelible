<script lang="ts">
	import type { PreviewView } from '../obsidian-model';

	interface Props {
		previewView: PreviewView;
		previewing: boolean;
		previewFilePath: string;
		previewBody: string;
		previewBodyHtml: string;
		previewError: string | null;
		fullTextOffAndMissing: boolean;
		fullTextMissing: boolean;
		previewMissingSummary: boolean;
		onEnableFullText: () => void;
		onRenderPreview: () => void;
		onSetPreviewView: (view: PreviewView) => void;
	}

	let {
		previewView,
		previewing,
		previewFilePath,
		previewBody,
		previewBodyHtml,
		previewError,
		fullTextOffAndMissing,
		fullTextMissing,
		previewMissingSummary,
		onEnableFullText,
		onRenderPreview,
		onSetPreviewView
	}: Props = $props();
</script>

<section class="section">
	<div class="section-head">
		<h2 class="section-title">Preview</h2>
		<p class="section-sub">
			Rendered against a sample document — switch between note and full-text companion
		</p>
	</div>

	<div class="preview-card">
		<div class="preview-head">
			<div class="preview-spacer"></div>
			<div class="seg" role="tablist" aria-label="Preview view">
				<button
					type="button"
					class="seg-btn"
					class:is-active={previewView === 'note'}
					role="tab"
					aria-selected={previewView === 'note'}
					onclick={() => onSetPreviewView('note')}>Note</button
				>
				<button
					type="button"
					class="seg-btn"
					class:is-active={previewView === 'full'}
					role="tab"
					aria-selected={previewView === 'full'}
					onclick={() => onSetPreviewView('full')}>Full text</button
				>
			</div>
			<button
				type="button"
				class="btn btn-ghost rerender-btn"
				onclick={onRenderPreview}
				disabled={previewing}
			>
				<svg width="13" height="13" viewBox="0 0 14 14" fill="none" aria-hidden="true">
					<path
						d="M2 7a5 5 0 0 1 8.5-3.5L12 5M12 7a5 5 0 0 1-8.5 3.5L2 9"
						stroke="currentColor"
						stroke-width="1.4"
						stroke-linecap="round"
						stroke-linejoin="round"
					/>
					<path
						d="M12 2.2V5h-2.8M2 11.8V9h2.8"
						stroke="currentColor"
						stroke-width="1.4"
						stroke-linecap="round"
						stroke-linejoin="round"
					/>
				</svg>
				Re-render
			</button>
		</div>

		{#if previewFilePath}
			<div class="preview-path">
				<span class="pp-label">vault path</span>
				<span class="pp-path">{previewFilePath}</span>
			</div>
		{/if}

		{#if previewError}
			<div class="alert-block">
				<div class="alert">
					<span class="alert-ico" aria-hidden="true">
						<svg viewBox="0 0 20 20">
							<path
								d="M10 7v4M10 13.5v.01M2.6 16.4 9.13 3.34a1 1 0 0 1 1.74 0L17.4 16.4a1 1 0 0 1-.87 1.5H3.47a1 1 0 0 1-.87-1.5Z"
								stroke="currentColor"
								stroke-width="1.5"
								stroke-linecap="round"
								stroke-linejoin="round"
								fill="none"
							/>
						</svg>
					</span>
					<div>
						<p class="alert-title">Preview failed</p>
						<p class="alert-body">
							MiniJinja could not render your templates. Fix the syntax to re-enable preview.
						</p>
						<span class="alert-code">{previewError}</span>
					</div>
				</div>
			</div>
		{/if}

		<div class="preview-body" class:is-dim={previewing}>
			{#if previewView === 'full'}
				{#if previewBody}
					<pre class="full-text-body">{previewBody}</pre>
				{:else if fullTextOffAndMissing}
					<div class="full-text-cta">
						<p class="full-text-cta-label">Export full text is off.</p>
						<button type="button" class="full-text-cta-btn" onclick={onEnableFullText}
							>Turn on</button
						>
					</div>
				{:else if fullTextMissing}
					<p class="preview-empty">
						This sample has no prepared readable asset, so no full-text companion is rendered.
					</p>
				{:else if !previewing}
					<p class="preview-empty">Save your settings to see the rendered preview.</p>
				{/if}
			{:else if previewBody}
				<!-- eslint-disable-next-line svelte/no-at-html-tags -->
				<div class="md">{@html previewBodyHtml}</div>
				{#if previewMissingSummary}
					<p class="preview-hint">
						No summary rendered for this preview. Documents without a stored summary omit the
						default Summary row.
					</p>
				{/if}
			{:else if !previewing}
				<p class="preview-empty">Save your settings to see the rendered preview.</p>
			{/if}
		</div>
	</div>
</section>

<style>
	.section {
		margin-top: 28px;
	}
	.section-head {
		display: flex;
		align-items: baseline;
		justify-content: space-between;
		margin: 0 4px 12px;
		gap: 12px;
	}
	.section-title {
		font-size: 12px;
		font-weight: 550;
		color: var(--text-tertiary);
		text-transform: uppercase;
		letter-spacing: 0.08em;
		margin: 0;
	}
	.section-sub {
		font-size: 12.5px;
		color: var(--text-tertiary);
		margin: 0;
	}
	.preview-card {
		position: relative;
		isolation: isolate;
		background:
			linear-gradient(
				140deg,
				color-mix(in oklab, var(--obs-mark-from) var(--obs-card-tint), transparent) 0%,
				transparent 62%
			),
			var(--bg-elevated);
		border: 1px solid var(--border-hairline);
		border-radius: 14px;
		overflow: hidden;
		box-shadow: var(--shadow-1);
	}
	.preview-card::after {
		content: '';
		position: absolute;
		top: -52px;
		right: -56px;
		width: 240px;
		height: 240px;
		background-color: var(--obs-mark-from);
		-webkit-mask-image: url("data:image/svg+xml;utf8,<svg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 240 240' fill='none' stroke='white' stroke-width='1.4' stroke-linecap='round' stroke-linejoin='round'><polygon points='120,12 216,78 120,228 24,78'/><polyline points='24,78 120,108 216,78'/><line x1='120' y1='12' x2='120' y2='228'/><polyline points='63,45 120,108 177,45'/><polyline points='24,78 63,45 120,12 177,45 216,78'/></svg>");
		mask-image: url("data:image/svg+xml;utf8,<svg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 240 240' fill='none' stroke='white' stroke-width='1.4' stroke-linecap='round' stroke-linejoin='round'><polygon points='120,12 216,78 120,228 24,78'/><polyline points='24,78 120,108 216,78'/><line x1='120' y1='12' x2='120' y2='228'/><polyline points='63,45 120,108 177,45'/><polyline points='24,78 63,45 120,12 177,45 216,78'/></svg>");
		-webkit-mask-repeat: no-repeat;
		mask-repeat: no-repeat;
		-webkit-mask-size: contain;
		mask-size: contain;
		opacity: 0.025;
		transform: rotate(10deg);
		pointer-events: none;
		z-index: 0;
	}
	.preview-card > * {
		position: relative;
		z-index: 1;
	}
	.preview-head {
		padding: 12px 16px;
		display: flex;
		gap: 12px;
		align-items: center;
		flex-wrap: wrap;
		border-bottom: 1px solid var(--border-hairline);
		background: var(--bg-secondary);
	}
	.preview-spacer {
		flex: 1;
		min-width: 8px;
	}
	.seg {
		display: inline-flex;
		background: var(--fill-hover);
		border: 1px solid var(--border-hairline);
		border-radius: 8px;
		padding: 2px;
	}
	.seg-btn,
	.btn,
	.full-text-cta-btn {
		border: 0;
		cursor: pointer;
		font: inherit;
	}
	.seg-btn {
		padding: 4px 10px;
		font-size: 12px;
		border-radius: 6px;
		color: var(--text-secondary);
		background: transparent;
	}
	.seg-btn.is-active {
		background: var(--bg-elevated);
		color: var(--text-primary);
		box-shadow:
			0 1px 2px rgba(0, 0, 0, 0.05),
			0 0 0 1px var(--border-hairline);
		font-weight: 500;
	}
	.btn {
		display: inline-flex;
		align-items: center;
		gap: 6px;
		padding: 7px 12px;
		border-radius: 8px;
		font-size: 13px;
		font-weight: 500;
		border: 1px solid var(--border-hairline);
		background: var(--bg-elevated);
		color: var(--text-primary);
	}
	.btn-ghost {
		border-color: transparent;
		background: transparent;
		color: var(--text-secondary);
	}
	.rerender-btn {
		font-size: 12px;
		padding: 5px 10px;
	}
	.preview-path {
		padding: 10px 16px;
		border-bottom: 1px solid var(--border-hairline);
		background: var(--bg-elevated);
		font-family: var(--font-mono, ui-monospace, monospace);
		font-size: 11.5px;
		color: var(--text-tertiary);
		display: flex;
		align-items: center;
		gap: 8px;
	}
	.pp-label {
		color: var(--text-quaternary);
	}
	.pp-path {
		color: var(--text-primary);
	}
	.preview-body {
		padding: 22px 26px 26px;
		background: var(--bg-elevated);
		min-height: 280px;
		position: relative;
		transition: opacity 220ms ease;
	}
	.preview-body.is-dim {
		opacity: 0.5;
	}
	.preview-body pre {
		margin: 0;
		font-family: var(--font-mono, ui-monospace, monospace);
		font-size: 12px;
		line-height: 1.65;
		color: var(--text-primary);
		white-space: pre-wrap;
		word-break: break-word;
	}
	.full-text-body {
		color: var(--text-secondary);
	}
	.md :global(h1) {
		font-size: 22px;
		letter-spacing: -0.02em;
		font-weight: 600;
		margin: 0 0 12px;
		color: var(--text-primary);
	}
	.md :global(h2) {
		font-size: 15.5px;
		letter-spacing: -0.01em;
		font-weight: 600;
		margin: 22px 0 10px;
		padding-bottom: 4px;
		border-bottom: 1px solid var(--border-hairline);
		color: var(--text-primary);
	}
	.md :global(h3) {
		font-size: 14px;
		font-weight: 600;
		margin: 18px 0 8px;
		color: var(--text-primary);
	}
	.md :global(p) {
		font-size: 13.5px;
		color: var(--text-secondary);
		margin: 0 0 10px;
		line-height: 1.6;
	}
	.md :global(ul),
	.md :global(ol) {
		margin: 0 0 10px;
		padding-left: 18px;
	}
	.md :global(li) {
		font-size: 13.5px;
		color: var(--text-primary);
		line-height: 1.6;
		margin: 2px 0;
	}
	.md :global(li li) {
		color: var(--text-secondary);
		font-size: 13px;
	}
	.md :global(a) {
		color: var(--obs-accent-ink);
		text-decoration: underline;
		text-underline-offset: 3px;
		text-decoration-thickness: 1px;
	}
	.md :global(blockquote) {
		margin: 10px 0;
		padding: 4px 12px;
		border-left: 3px solid var(--obs-accent-soft);
		color: var(--text-secondary);
		font-size: 13.5px;
		line-height: 1.6;
	}
	.md :global(code) {
		font-family: var(--font-mono, ui-monospace, monospace);
		font-size: 11.5px;
		background: var(--fill-hover);
		padding: 1px 6px;
		border-radius: 4px;
		color: var(--text-primary);
	}
	.md :global(pre) {
		margin: 0 0 12px;
		padding: 12px 14px;
		background: var(--obs-editor-bg);
		border: 1px solid var(--border-hairline);
		border-radius: 8px;
		font-family: var(--font-mono, ui-monospace, monospace);
		font-size: 12px;
		line-height: 1.6;
		color: var(--text-primary);
		overflow-x: auto;
		white-space: pre;
		word-break: normal;
	}
	.md :global(pre code) {
		background: transparent;
		padding: 0;
		border-radius: 0;
		font-size: inherit;
	}
	.md :global(img) {
		max-width: 100%;
		border-radius: 8px;
		display: block;
		margin: 0 0 14px;
	}
	.md :global(hr) {
		border: 0;
		border-top: 1px solid var(--border-hairline);
		margin: 16px 0;
	}
	.md :global(table) {
		border-collapse: collapse;
		width: 100%;
		font-size: 13px;
		margin: 0 0 12px;
	}
	.md :global(th),
	.md :global(td) {
		border: 1px solid var(--border-hairline);
		padding: 6px 10px;
		text-align: left;
		color: var(--text-primary);
	}
	.md :global(th) {
		background: var(--fill-hover);
		font-weight: 600;
	}
	.preview-empty {
		font-size: 13px;
		color: var(--text-tertiary);
		font-style: italic;
		margin: 0;
	}
	.preview-hint {
		margin: 18px 0 0;
		padding: 10px 12px;
		border: 1px solid var(--border-hairline);
		border-radius: 8px;
		background: var(--fill-hover);
		color: var(--text-tertiary);
		font-size: 12.5px;
		line-height: 1.45;
	}
	.full-text-cta {
		display: flex;
		align-items: center;
		gap: 12px;
	}
	.full-text-cta-label {
		font-size: 13px;
		color: var(--text-secondary);
		margin: 0;
	}
	.full-text-cta-btn {
		height: 28px;
		padding: 0 14px;
		border-radius: 6px;
		background: var(--obs-accent);
		color: var(--text-on-color);
		font-size: 12px;
		font-weight: 500;
	}
	.alert-block {
		padding: 16px 22px 18px;
	}
	.alert {
		padding: 14px;
		border-radius: 12px;
		background: var(--obs-alert-bg);
		border: 1px solid var(--obs-alert-border);
		display: grid;
		grid-template-columns: auto 1fr;
		gap: 12px;
	}
	.alert-ico {
		width: 20px;
		height: 20px;
		color: var(--obs-alert-text);
	}
	.alert-ico svg {
		width: 100%;
		height: 100%;
	}
	.alert-title,
	.alert-body {
		margin: 0;
	}
	.alert-title {
		font-size: 13.5px;
		font-weight: 550;
		color: var(--text-primary);
	}
	.alert-body {
		font-size: 12.5px;
		color: var(--text-secondary);
		line-height: 1.5;
	}
	.alert-code {
		display: inline-block;
		margin-top: 10px;
		font-family: var(--font-mono, ui-monospace, monospace);
		font-size: 11.5px;
		color: var(--obs-alert-text);
		background: var(--bg-elevated);
		border-radius: 6px;
		padding: 6px 10px;
		border: 1px solid var(--obs-alert-border);
		word-break: break-all;
	}
</style>
