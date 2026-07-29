<script lang="ts">
	export type ViewTab = 'reader' | 'original' | 'pdf' | 'screenshot';

	interface Props {
		activeTab: ViewTab;
		availableTabs: ViewTab[];
		onTabChange: (tab: ViewTab) => void;
	}

	let { activeTab, availableTabs, onTabChange }: Props = $props();

	const allTabs: { value: ViewTab; label: string }[] = [
		{ value: 'reader', label: 'Reader' },
		{ value: 'original', label: 'Original' },
		{ value: 'pdf', label: 'PDF' },
		{ value: 'screenshot', label: 'Screenshot' }
	];

	function isEnabled(tab: ViewTab): boolean {
		return availableTabs.includes(tab);
	}

	function handleKeydown(e: KeyboardEvent) {
		if (e.key !== 'ArrowLeft' && e.key !== 'ArrowRight') return;
		e.preventDefault();

		const enabledTabs = allTabs.filter((t) => isEnabled(t.value));
		const currentIndex = enabledTabs.findIndex((t) => t.value === activeTab);
		if (currentIndex === -1) return;

		let nextIndex: number;
		if (e.key === 'ArrowRight') {
			nextIndex = (currentIndex + 1) % enabledTabs.length;
		} else {
			nextIndex = (currentIndex - 1 + enabledTabs.length) % enabledTabs.length;
		}
		const next = enabledTabs[nextIndex];
		if (next) onTabChange(next.value);
	}
</script>

<div class="view-tabs">
	<div
		class="view-tabs-inner"
		role="tablist"
		tabindex="-1"
		aria-label="Content view"
		onkeydown={handleKeydown}
	>
		{#each allTabs as tab (tab.value)}
			{@const enabled = isEnabled(tab.value)}
			{@const active = tab.value === activeTab}
			<button
				type="button"
				role="tab"
				class="view-tab"
				class:active
				class:disabled={!enabled}
				aria-selected={active}
				aria-disabled={!enabled}
				tabindex={active ? 0 : -1}
				onclick={() => {
					if (enabled) onTabChange(tab.value);
				}}
			>
				{tab.label}
			</button>
		{/each}
	</div>
</div>

<style>
	.view-tabs {
		display: flex;
		align-items: center;
		padding: 10px 20px 0;
		gap: 0;
		flex-shrink: 0;
	}

	.view-tabs-inner {
		display: inline-flex;
		background: var(--seg-bg);
		border-radius: 8px;
		padding: 2px;
		gap: 1px;
	}

	.view-tab {
		padding: 5px 16px;
		border-radius: 7px;
		font-size: 13px;
		font-weight: 500;
		color: var(--text-secondary);
		cursor: pointer;
		transition: all 200ms ease;
		line-height: 1.45;
		letter-spacing: -0.01em;
		border: none;
		background: transparent;
		font-family: var(--font-sans);
	}

	.view-tab:hover:not(.disabled) {
		color: var(--text-primary);
	}

	.view-tab.active {
		background: var(--seg-on);
		color: var(--text-primary);
		box-shadow: var(--seg-shadow);
	}

	.view-tab.disabled {
		color: var(--text-tertiary);
		cursor: default;
		opacity: 0.5;
	}
</style>
