<script lang="ts">
	import { goto } from '$app/navigation';
	import { resolve } from '$app/paths';
	import { t, type MessageKey } from '$lib/i18n';
	import { getLibrary } from '$lib/stores/library.svelte';

	const lib = getLibrary();

	let open = $state(false);
	let wrapperEl = $state<HTMLDivElement | undefined>(undefined);

	type LibraryRoute =
		| '/library'
		| '/library/articles'
		| '/library/books'
		| '/library/emails'
		| '/library/pdfs'
		| '/library/tweets'
		| '/library/videos'
		| '/library/podcasts';

	type ContentOption = {
		labelKey: MessageKey;
		type: string | undefined;
		href: LibraryRoute;
	};

	const options: ContentOption[] = [
		{ labelKey: 'common_all', type: undefined, href: '/library' },
		{ labelKey: 'library_nav_articles', type: 'articles', href: '/library/articles' },
		{ labelKey: 'library_nav_books', type: 'books', href: '/library/books' },
		{ labelKey: 'library_nav_emails', type: 'emails', href: '/library/emails' },
		{ labelKey: 'library_nav_pdfs', type: 'pdfs', href: '/library/pdfs' },
		{ labelKey: 'library_nav_tweets', type: 'tweets', href: '/library/tweets' },
		{ labelKey: 'library_nav_videos', type: 'videos', href: '/library/videos' }
	];

	const currentLabel = $derived(
		$t(options.find((option) => option.type === lib.activeType)?.labelKey ?? 'common_library')
	);

	$effect(() => {
		if (!open) return;

		function handleClickOutside(e: MouseEvent) {
			if (wrapperEl && !wrapperEl.contains(e.target as Node)) {
				open = false;
			}
		}

		function handleKeydown(e: KeyboardEvent) {
			if (e.key === 'Escape') open = false;
		}

		document.addEventListener('mousedown', handleClickOutside);
		document.addEventListener('keydown', handleKeydown);

		return () => {
			document.removeEventListener('mousedown', handleClickOutside);
			document.removeEventListener('keydown', handleKeydown);
		};
	});

	function select(opt: ContentOption) {
		lib.setActiveType(opt.type);
		goto(resolve(opt.href));
		open = false;
	}
</script>

<div bind:this={wrapperEl} class="content-type-wrapper">
	<button
		type="button"
		class="content-type-trigger"
		onclick={() => (open = !open)}
		aria-haspopup="listbox"
		aria-expanded={open}
	>
		<span class="trigger-label">{currentLabel}</span>
		<svg
			viewBox="0 0 24 24"
			fill="none"
			stroke="currentColor"
			stroke-width="2"
			aria-hidden="true"
			class="trigger-chevron"
		>
			<polyline points="6 9 12 15 18 9" />
		</svg>
	</button>

	{#if open}
		<div class="content-type-popover" role="listbox" aria-label={$t('library_content_type_filter')}>
			{#each options as opt (opt.href)}
				<button
					type="button"
					class="content-type-option"
					role="option"
					aria-selected={lib.activeType === opt.type}
					class:selected={lib.activeType === opt.type}
					onclick={() => select(opt)}
				>
					{$t(opt.labelKey)}
					{#if lib.activeType === opt.type}
						<svg
							viewBox="0 0 24 24"
							fill="none"
							stroke="currentColor"
							stroke-width="2.5"
							aria-hidden="true"
						>
							<polyline points="20 6 9 17 4 12" />
						</svg>
					{/if}
				</button>
			{/each}
		</div>
	{/if}
</div>

<style>
	.content-type-wrapper {
		position: relative;
	}

	.content-type-trigger {
		display: flex;
		align-items: baseline;
		gap: 6px;
		background: none;
		border: none;
		cursor: pointer;
		padding: 0;
	}

	.content-type-trigger:hover {
		background: none;
	}

	.trigger-label {
		font-family: var(--font-sans);
		font-size: 28px;
		font-weight: 700;
		letter-spacing: -0.03em;
		line-height: 1.18;
		color: var(--text-primary);
	}

	.trigger-chevron {
		width: 16px;
		height: 16px;
		color: var(--text-tertiary);
		flex-shrink: 0;
	}

	.content-type-popover {
		position: absolute;
		left: 0;
		top: calc(100% + 6px);
		z-index: 100;
		width: 180px;
		background: var(--bg-elevated);
		backdrop-filter: blur(40px) saturate(200%);
		-webkit-backdrop-filter: blur(40px) saturate(200%);
		border: 0.5px solid var(--border-primary);
		border-radius: var(--radius-xl);
		box-shadow: var(--shadow-3);
		padding: 4px;
		animation: popover-open 0.15s ease-out;
	}

	@keyframes popover-open {
		from {
			opacity: 0;
			transform: scale(0.96) translateY(-4px);
		}
		to {
			opacity: 1;
			transform: scale(1) translateY(0);
		}
	}

	.content-type-option {
		display: flex;
		align-items: center;
		justify-content: space-between;
		width: 100%;
		padding: 8px 10px;
		border-radius: var(--radius-sm);
		background: none;
		border: none;
		font-family: var(--font-sans);
		font-size: 13px;
		font-weight: 400;
		letter-spacing: -0.01em;
		color: var(--text-primary);
		cursor: pointer;
		text-align: left;
		transition: background 0.1s ease;
	}

	.content-type-option:hover {
		background: var(--fill-hover);
	}

	.content-type-option.selected {
		font-weight: 500;
	}

	.content-type-option svg {
		width: 14px;
		height: 14px;
		color: var(--accent);
		flex-shrink: 0;
	}
</style>
