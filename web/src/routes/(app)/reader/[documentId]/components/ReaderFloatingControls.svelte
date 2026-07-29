<script lang="ts">
	import type { ViewTab } from '$lib/components/reader/ViewTabs.svelte';
	import TtsController from '$lib/components/reader/TtsController.svelte';
	import TypographyPopover from '$lib/components/reader/TypographyPopover.svelte';
	import FocusMode from '$lib/components/reader/FocusMode.svelte';

	type FocusState = 'idle' | 'selecting' | 'active' | 'paused' | 'completed';

	interface Props {
		documentId: string;
		activeTab: ViewTab;
		readableReady: boolean;
		ttsOpen: boolean;
		readerArticleBodyEl: HTMLDivElement | undefined;
		showTypography: boolean;
		aaButtonEl: HTMLButtonElement | undefined;
		focusModeState: FocusState;
		focusStartProgress: number;
		progress: number;
		focusHighlightsCreated: number;
		onTypographyClose: () => void;
		onFocusStart: () => void;
		onFocusPause: () => void;
		onFocusResume: () => void;
		onFocusComplete: () => void;
		onFocusExit: () => void;
	}

	let {
		documentId,
		activeTab,
		readableReady,
		ttsOpen,
		readerArticleBodyEl,
		showTypography,
		aaButtonEl,
		focusModeState,
		focusStartProgress,
		progress,
		focusHighlightsCreated,
		onTypographyClose,
		onFocusStart,
		onFocusPause,
		onFocusResume,
		onFocusComplete,
		onFocusExit
	}: Props = $props();
</script>

{#if ttsOpen && activeTab === 'reader' && readableReady}
	<TtsController {documentId} articleBodyEl={readerArticleBodyEl} />
{/if}

{#if showTypography && aaButtonEl}
	<TypographyPopover anchorEl={aaButtonEl} onClose={onTypographyClose} />
{/if}

{#if focusModeState !== 'idle'}
	<FocusMode
		focusState={focusModeState}
		startProgress={focusStartProgress}
		currentProgress={progress}
		highlightsCreated={focusHighlightsCreated}
		onStart={onFocusStart}
		onPause={onFocusPause}
		onResume={onFocusResume}
		onComplete={onFocusComplete}
		onExit={onFocusExit}
	/>
{/if}
