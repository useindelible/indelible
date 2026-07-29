<script lang="ts">
	import type { DocumentListEntry } from '$lib/api';
	import type { ViewTab } from './ViewTabs.svelte';

	interface Props {
		item: DocumentListEntry;
		progress: number;
		onBack: () => void;
		onPrev?: () => void;
		onNext?: () => void;
		hasPrev?: boolean;
		hasNext?: boolean;
		aaButtonEl?: HTMLButtonElement;
		onAaClick: () => void;
		isFavorite?: boolean;
		onBookmark?: () => void;
		savedToLibrary?: boolean;
		savingToLibrary?: boolean;
		onSaveToLibrary?: () => void;
		detailPanelOpen?: boolean;
		onDetailPanelToggle?: () => void;
		bookMode?: boolean;
		subtitle?: string;
		progressLabel?: string;
		showLeftPanelToggle?: boolean;
		leftPanelOpen?: boolean;
		onLeftPanelToggle?: () => void;
		onBookmarkCreate?: () => void;
		availableTabs?: ViewTab[];
		activeTab?: ViewTab;
		onTabChange?: (tab: ViewTab) => void;
		ttsActive?: boolean;
		onTtsToggle?: () => void;
		onMenuClick?: () => void;
		menuAriaLabel?: string;
	}

	let {
		item,
		progress,
		onBack,
		onPrev,
		onNext,
		hasPrev = false,
		hasNext = false,
		aaButtonEl = $bindable(),
		onAaClick,
		isFavorite = false,
		onBookmark,
		savedToLibrary = true,
		savingToLibrary = false,
		onSaveToLibrary,
		detailPanelOpen = false,
		onDetailPanelToggle,
		bookMode = false,
		subtitle,
		progressLabel,
		showLeftPanelToggle = false,
		leftPanelOpen = false,
		onLeftPanelToggle,
		onBookmarkCreate,
		availableTabs,
		activeTab,
		onTabChange,
		ttsActive = false,
		onTtsToggle,
		onMenuClick,
		menuAriaLabel = 'Open navigation'
	}: Props = $props();

	const tabLabels: Record<ViewTab, string> = {
		reader: 'Reader',
		original: 'Original',
		pdf: 'PDF',
		screenshot: 'Screenshot'
	};

	const allTabs: ViewTab[] = ['reader', 'original', 'pdf', 'screenshot'];

	const activeTabLabel = $derived(activeTab ? (tabLabels[activeTab] ?? 'Reader') : 'Reader');
	const canSwitch = $derived((availableTabs?.length ?? 0) > 1);

	let showViewDropdown = $state(false);
	let viewPillWrapperEl = $state<HTMLDivElement | undefined>(undefined);

	$effect(() => {
		if (!showViewDropdown) return;
		function handleClickOutside(e: MouseEvent) {
			if (!viewPillWrapperEl?.contains(e.target as Node)) {
				showViewDropdown = false;
			}
		}
		document.addEventListener('click', handleClickOutside, true);
		return () => document.removeEventListener('click', handleClickOutside, true);
	});
</script>

<div class="reader-toolbar">
	<div class="toolbar-left">
		{#if onMenuClick}
			<button
				type="button"
				class="toolbar-btn menu-btn"
				aria-label={menuAriaLabel}
				onclick={onMenuClick}
			>
				<svg
					viewBox="0 0 24 24"
					fill="none"
					stroke="currentColor"
					stroke-width="1.7"
					stroke-linecap="round"
					stroke-linejoin="round"
					aria-hidden="true"
				>
					<line x1="3" y1="6" x2="21" y2="6" />
					<line x1="3" y1="12" x2="21" y2="12" />
					<line x1="3" y1="18" x2="21" y2="18" />
				</svg>
			</button>
		{/if}
		{#if showLeftPanelToggle}
			<button
				type="button"
				class="toolbar-btn"
				class:accent-text={leftPanelOpen}
				aria-label={leftPanelOpen ? 'Hide sidebar' : 'Show sidebar'}
				onclick={onLeftPanelToggle}
			>
				<svg
					viewBox="0 0 24 24"
					fill="none"
					stroke="currentColor"
					stroke-width="1.5"
					stroke-linecap="round"
					stroke-linejoin="round"
				>
					<rect x="3" y="3" width="18" height="18" rx="2" />
					<line x1="9" y1="3" x2="9" y2="21" />
				</svg>
			</button>
		{/if}
		<button type="button" class="toolbar-btn" aria-label="Back to library" onclick={onBack}>
			<svg
				viewBox="0 0 24 24"
				fill="none"
				stroke="currentColor"
				stroke-width="2"
				stroke-linecap="round"
				stroke-linejoin="round"
			>
				<polyline points="15 18 9 12 15 6" />
			</svg>
		</button>
		{#if !bookMode}
			<button
				type="button"
				class="toolbar-btn pn-btn"
				aria-label="Previous item"
				disabled={!hasPrev}
				onclick={onPrev}
			>
				<svg
					viewBox="0 0 24 24"
					fill="none"
					stroke="currentColor"
					stroke-width="2"
					stroke-linecap="round"
					stroke-linejoin="round"
				>
					<polyline points="18 15 12 9 6 15" />
				</svg>
			</button>
			<button
				type="button"
				class="toolbar-btn pn-btn"
				aria-label="Next item"
				disabled={!hasNext}
				onclick={onNext}
			>
				<svg
					viewBox="0 0 24 24"
					fill="none"
					stroke="currentColor"
					stroke-width="2"
					stroke-linecap="round"
					stroke-linejoin="round"
				>
					<polyline points="6 9 12 15 18 9" />
				</svg>
			</button>
		{/if}

		{#if availableTabs && availableTabs.length > 0}
			<div class="toolbar-view-divider" aria-hidden="true"></div>
			<div class="view-pill-wrapper" bind:this={viewPillWrapperEl}>
				<button
					type="button"
					class="view-pill"
					class:no-switch={!canSwitch}
					aria-label="Switch view"
					aria-haspopup="listbox"
					aria-expanded={showViewDropdown}
					onclick={() => {
						if (canSwitch) showViewDropdown = !showViewDropdown;
					}}
				>
					<span class="view-dot" aria-hidden="true"></span>
					<span>{activeTabLabel}</span>
					{#if canSwitch}
						<svg
							class="view-chev"
							viewBox="0 0 24 24"
							fill="none"
							stroke="currentColor"
							stroke-width="2"
							stroke-linecap="round"
							stroke-linejoin="round"
							aria-hidden="true"
						>
							<polyline points="6 9 12 15 18 9" />
						</svg>
					{/if}
				</button>

				{#if showViewDropdown}
					<div class="view-dropdown-menu" role="listbox" aria-label="View options">
						{#each allTabs as tab (tab)}
							{@const isAvailable = availableTabs.includes(tab)}
							{@const isActive = tab === activeTab}
							<button
								type="button"
								role="option"
								aria-selected={isActive}
								class="view-dropdown-item"
								class:active={isActive}
								class:disabled={!isAvailable}
								onclick={() => {
									if (isAvailable) {
										onTabChange?.(tab);
										showViewDropdown = false;
									}
								}}
							>
								{#if isActive}
									<svg
										class="view-check"
										viewBox="0 0 24 24"
										fill="none"
										stroke="currentColor"
										stroke-width="2.5"
										stroke-linecap="round"
										stroke-linejoin="round"
										aria-hidden="true"
									>
										<polyline points="20 6 9 17 4 12" />
									</svg>
								{:else}
									<span class="view-check-gap" aria-hidden="true"></span>
								{/if}
								{tabLabels[tab]}
							</button>
						{/each}
					</div>
				{/if}
			</div>
		{/if}
	</div>

	<div class="toolbar-center" title={item.title}>
		<span class="toolbar-title">{item.title}</span>
		{#if subtitle}
			<span class="toolbar-sep"></span>
			<span class="toolbar-chapter">{subtitle}</span>
		{/if}
	</div>

	<div class="toolbar-right">
		{#if progressLabel}
			<div class="toolbar-progress-pill">
				<span class="toolbar-progress-pill-text">{progressLabel}</span>
			</div>
		{:else if progress > 0}
			<span class="toolbar-progress">{progress}%</span>
		{/if}

		<button
			type="button"
			class="toolbar-btn aa-btn"
			aria-label="Typography settings"
			bind:this={aaButtonEl}
			onclick={onAaClick}
		>
			Aa
		</button>

		{#if onBookmarkCreate}
			<button
				type="button"
				class="toolbar-btn bookmark-active"
				aria-label="Add bookmark"
				onclick={onBookmarkCreate}
			>
				<svg
					viewBox="0 0 24 24"
					fill="currentColor"
					stroke="currentColor"
					stroke-width="1.5"
					stroke-linecap="round"
					stroke-linejoin="round"
				>
					<path d="M19 21l-7-5-7 5V5a2 2 0 012-2h10a2 2 0 012 2z" />
				</svg>
			</button>
		{:else if onBookmark}
			<button
				type="button"
				class="toolbar-btn"
				class:bookmark-active={isFavorite}
				aria-label={isFavorite ? 'Remove bookmark' : 'Bookmark'}
				onclick={onBookmark}
			>
				<svg
					viewBox="0 0 24 24"
					fill={isFavorite ? 'currentColor' : 'none'}
					stroke="currentColor"
					stroke-width="1.5"
					stroke-linecap="round"
					stroke-linejoin="round"
				>
					<path d="M19 21l-7-5-7 5V5a2 2 0 012-2h10a2 2 0 012 2z" />
				</svg>
			</button>
		{/if}

		{#if onDetailPanelToggle}
			<button
				type="button"
				class="toolbar-btn"
				class:accent-text={detailPanelOpen}
				aria-label={detailPanelOpen ? 'Hide info panel' : 'Show info panel'}
				onclick={onDetailPanelToggle}
			>
				<svg
					viewBox="0 0 24 24"
					fill="none"
					stroke="currentColor"
					stroke-width="1.5"
					stroke-linecap="round"
					stroke-linejoin="round"
				>
					<rect x="3" y="3" width="18" height="18" rx="2" />
					<line x1="15" y1="3" x2="15" y2="21" />
				</svg>
			</button>
		{/if}

		{#if onTtsToggle !== undefined}
			<button
				type="button"
				class="toolbar-btn"
				class:tts-active={ttsActive}
				aria-label={ttsActive ? 'Close listen mode' : 'Listen to article'}
				aria-pressed={ttsActive}
				onclick={onTtsToggle}
			>
				<svg
					viewBox="0 0 24 24"
					fill="none"
					stroke="currentColor"
					stroke-width="1.8"
					stroke-linecap="round"
					stroke-linejoin="round"
				>
					<path d="M11 5L6 9H2v6h4l5 4V5z" />
					<path d="M15.54 8.46a5 5 0 010 7.07" />
					<path d="M19.07 4.93a10 10 0 010 14.14" />
				</svg>
			</button>
		{/if}

		{#if !savedToLibrary && onSaveToLibrary}
			<div class="toolbar-right-divider"></div>
			<button
				type="button"
				class="toolbar-save-btn"
				disabled={savingToLibrary}
				onclick={onSaveToLibrary}
			>
				{savingToLibrary ? 'Saving...' : 'Save'}
			</button>
		{/if}
	</div>
</div>

<style>
	.reader-toolbar {
		height: 44px;
		min-height: 44px;
		display: flex;
		align-items: center;
		padding: 0 16px;
		border-bottom: 0.5px solid var(--border-primary);
		background: var(--bg-content);
		position: relative;
		z-index: 5;
		flex-shrink: 0;
	}

	.toolbar-left,
	.toolbar-right {
		display: flex;
		align-items: center;
		gap: 4px;
	}

	.toolbar-center {
		flex: 1;
		display: flex;
		align-items: center;
		justify-content: center;
		gap: 0;
		font-size: 13px;
		font-weight: 500;
		color: var(--text-secondary);
		letter-spacing: -0.01em;
		white-space: nowrap;
		overflow: hidden;
		text-overflow: ellipsis;
		padding: 0 12px;
		font-family: var(--font-sans);
	}

	.toolbar-title {
		overflow: hidden;
		text-overflow: ellipsis;
		flex-shrink: 1;
		min-width: 0;
	}

	.toolbar-sep {
		width: 3px;
		height: 3px;
		border-radius: 50%;
		background: var(--text-tertiary);
		flex-shrink: 0;
		margin: 0 8px;
	}

	.toolbar-chapter {
		color: var(--text-tertiary);
		font-weight: 400;
		overflow: hidden;
		text-overflow: ellipsis;
		flex-shrink: 1;
		min-width: 0;
	}

	.toolbar-btn {
		width: 32px;
		height: 32px;
		display: flex;
		align-items: center;
		justify-content: center;
		border-radius: 7px;
		cursor: pointer;
		color: var(--text-primary);
		transition: background 120ms ease;
		border: none;
		background: transparent;
		font-family: var(--font-sans);
	}

	.toolbar-btn:hover:not(:disabled) {
		background: var(--fill-hover);
	}

	.toolbar-btn:disabled {
		opacity: 0.3;
		cursor: default;
	}

	.toolbar-btn :global(svg) {
		width: 16px;
		height: 16px;
	}

	.toolbar-btn.aa-btn {
		font-size: 14px;
		font-weight: 600;
		letter-spacing: -0.02em;
		width: auto;
		padding: 0 4px;
	}

	.toolbar-right-divider {
		width: 0.5px;
		height: 20px;
		background: var(--border-secondary);
		margin: 0 4px;
		flex-shrink: 0;
	}

	.toolbar-save-btn {
		height: 28px;
		border: none;
		border-radius: var(--radius-full);
		background: var(--accent);
		color: var(--text-on-color);
		cursor: pointer;
		font-family: var(--font-sans);
		font-size: 12.5px;
		font-weight: 600;
		letter-spacing: -0.01em;
		padding: 0 14px;
		margin: 0 2px;
		box-shadow: 0 1px 6px var(--accent-glow);
		transition: background 150ms ease;
	}

	.toolbar-save-btn:hover:not(:disabled) {
		background: var(--accent-hover);
	}

	.toolbar-save-btn:disabled {
		opacity: 0.55;
		cursor: default;
	}

	.toolbar-btn.accent-text {
		color: var(--accent);
	}

	.toolbar-btn.bookmark-active {
		color: var(--warning);
	}

	.toolbar-btn.tts-active {
		color: var(--accent);
		background: var(--fill-selected);
	}

	.toolbar-btn.tts-active:hover:not(:disabled) {
		background: var(--fill-selected-strong);
	}

	.toolbar-progress {
		font-size: 12px;
		font-weight: 500;
		color: var(--text-tertiary);
		letter-spacing: -0.005em;
		padding: 0 4px;
		white-space: nowrap;
		font-family: var(--font-sans);
	}

	.toolbar-progress-pill {
		padding: 3px 10px;
		border-radius: 980px;
		background: var(--fill-hover);
	}

	.toolbar-progress-pill-text {
		font-size: 11px;
		font-weight: 500;
		color: var(--text-secondary);
		white-space: nowrap;
		font-family: var(--font-sans);
	}

	/* ---- View switcher pill ---- */

	.toolbar-view-divider {
		width: 1px;
		height: 20px;
		background: var(--border-primary);
		margin: 0 4px;
		flex-shrink: 0;
	}

	.view-pill-wrapper {
		position: relative;
	}

	.view-pill {
		display: inline-flex;
		align-items: center;
		gap: 5px;
		padding: 4px 8px 4px 10px;
		border-radius: 7px;
		font-size: 12px;
		font-weight: 500;
		color: var(--text-primary);
		background: var(--seg-bg);
		border: none;
		cursor: pointer;
		letter-spacing: -0.01em;
		line-height: 1.45;
		transition: background 120ms ease;
		font-family: var(--font-sans);
		white-space: nowrap;
	}

	.view-pill:hover {
		background: var(--fill-hover);
	}

	.view-pill.no-switch {
		cursor: default;
		padding-right: 10px;
	}

	.view-pill.no-switch:hover {
		background: var(--seg-bg);
	}

	.view-dot {
		width: 6px;
		height: 6px;
		border-radius: 50%;
		background: var(--accent);
		flex-shrink: 0;
	}

	.view-chev {
		width: 11px;
		height: 11px;
		color: var(--text-secondary);
		flex-shrink: 0;
	}

	/* ---- View dropdown menu ---- */

	.view-dropdown-menu {
		position: absolute;
		top: calc(100% + 4px);
		left: 0;
		background: var(--bg-elevated, var(--bg-primary));
		border: 1px solid var(--border-primary);
		border-radius: 10px;
		padding: 4px;
		box-shadow: var(--shadow-3, 0 8px 24px rgba(0, 0, 0, 0.12));
		z-index: 50;
		min-width: 140px;
	}

	.view-dropdown-item {
		display: flex;
		align-items: center;
		gap: 8px;
		width: 100%;
		padding: 7px 10px;
		border-radius: 7px;
		border: none;
		background: transparent;
		font-family: var(--font-sans);
		font-size: 13px;
		font-weight: 500;
		color: var(--text-primary);
		cursor: pointer;
		text-align: left;
		letter-spacing: -0.01em;
		transition: background 100ms ease;
	}

	.view-dropdown-item:hover {
		background: var(--fill-hover);
	}

	.view-dropdown-item.active {
		color: var(--accent);
	}

	.view-dropdown-item.disabled {
		color: var(--text-tertiary);
		opacity: 0.5;
		cursor: default;
	}

	.view-dropdown-item.disabled:hover {
		background: transparent;
	}

	.view-check {
		width: 14px;
		height: 14px;
		flex-shrink: 0;
		color: var(--accent);
	}

	.view-check-gap {
		width: 14px;
		flex-shrink: 0;
	}

	/* ---- Responsive ---- */

	.menu-btn {
		display: none;
	}

	@media (max-width: 599px) {
		.reader-toolbar {
			padding: 0 8px;
			height: 50px;
			min-height: 50px;
		}

		.menu-btn {
			display: flex;
		}

		/* The compact bar keeps: menu, back, view pill | Aa, bookmark, details, listen.
		   Title, prev/next stepping, and the progress readout yield to the article. */
		.pn-btn,
		.toolbar-center,
		.toolbar-progress {
			display: none;
		}

		.toolbar-left {
			flex: 1;
			min-width: 0;
		}
	}
</style>
