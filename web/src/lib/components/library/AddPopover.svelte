<script lang="ts">
	import { t } from '$lib/i18n';
	import { getModalStore } from '$lib/stores/addItemModal.svelte';

	const modal = getModalStore();

	function handleWindowClick() {
		if (modal.popoverOpen) modal.closePopover();
	}
</script>

<svelte:window onclick={handleWindowClick} />

{#if modal.popoverOpen}
	<div class="add-popover" role="menu" aria-label={$t('library_add_to_library')}>
		<button type="button" class="popover-item" role="menuitem" onclick={() => modal.open('url')}>
			<span class="popover-label">
				<svg viewBox="0 0 24 24" aria-hidden="true">
					<path d="M10 13a5 5 0 0 0 7.54.54l3-3a5 5 0 0 0-7.07-7.07l-1.72 1.71" />
					<path d="M14 11a5 5 0 0 0-7.54-.54l-3 3a5 5 0 0 0 7.07 7.07l1.71-1.71" />
				</svg>
				{$t('library_save_url')}
			</span>
			<span class="popover-shortcut">A</span>
		</button>
		<button type="button" class="popover-item" role="menuitem" onclick={() => modal.open('upload')}>
			<span class="popover-label">
				<svg viewBox="0 0 24 24" aria-hidden="true">
					<path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z" />
					<polyline points="14 2 14 8 20 8" />
					<path d="M12 18v-6" />
					<path d="m9 15 3-3 3 3" />
				</svg>
				{$t('library_upload_file')}
			</span>
		</button>
		<div class="popover-divider" role="separator"></div>
		<button type="button" class="popover-item" role="menuitem" onclick={() => modal.open('email')}>
			<span class="popover-label">
				<svg viewBox="0 0 24 24" aria-hidden="true">
					<path d="M4 4h16c1.1 0 2 .9 2 2v12c0 1.1-.9 2-2 2H4c-1.1 0-2-.9-2-2V6c0-1.1.9-2 2-2z" />
					<polyline points="22,6 12,13 2,6" />
				</svg>
				{$t('library_email_forwarding')}
			</span>
		</button>
		<button type="button" class="popover-item" role="menuitem" onclick={() => modal.open('rss')}>
			<span class="popover-label">
				<svg viewBox="0 0 24 24" aria-hidden="true">
					<path d="M4 11a9 9 0 0 1 9 9" />
					<path d="M4 4a16 16 0 0 1 16 16" />
					<circle cx="5" cy="19" r="1" fill="currentColor" stroke="none" />
				</svg>
				{$t('library_add_rss_feed')}
			</span>
			<span class="popover-shortcut">R</span>
		</button>
		<div class="popover-divider" role="separator"></div>
		<button type="button" class="popover-item" role="menuitem" onclick={() => modal.open('x')}>
			<span class="popover-label">
				<svg class="brand" viewBox="0 0 24 24" aria-hidden="true">
					<path
						d="M18.244 2.25h3.308l-7.227 8.26 8.502 11.24H16.17l-5.214-6.817L4.99 21.75H1.68l7.73-8.835L1.254 2.25H8.08l4.713 6.231zm-1.161 17.52h1.833L7.084 4.126H5.117z"
					/>
				</svg>
				{$t('library_x_post')}
			</span>
		</button>
		<button
			type="button"
			class="popover-item"
			role="menuitem"
			onclick={() => modal.open('youtube')}
		>
			<span class="popover-label">
				<svg class="brand" viewBox="0 0 24 24" aria-hidden="true">
					<path
						d="M23.498 6.186a3.016 3.016 0 0 0-2.122-2.136C19.505 3.545 12 3.545 12 3.545s-7.505 0-9.377.505A3.017 3.017 0 0 0 .502 6.186C0 8.07 0 12 0 12s0 3.93.502 5.814a3.016 3.016 0 0 0 2.122 2.136c1.871.505 9.376.505 9.376.505s7.505 0 9.377-.505a3.015 3.015 0 0 0 2.122-2.136C24 15.93 24 12 24 12s0-3.93-.502-5.814zM9.545 15.568V8.432L15.818 12z"
					/>
				</svg>
				YouTube
			</span>
		</button>
	</div>
{/if}

<style>
	.add-popover {
		position: fixed;
		top: 50px;
		left: 12px;
		width: 260px;
		background: var(--bg-elevated);
		border-radius: 12px;
		box-shadow:
			0 8px 40px rgba(0, 0, 0, 0.14),
			0 0 0 0.5px rgba(0, 0, 0, 0.1);
		padding: 4px 0;
		z-index: 1000;
	}

	[data-theme='dark'] .add-popover {
		box-shadow:
			0 8px 40px rgba(0, 0, 0, 0.4),
			0 0 0 0.5px rgba(255, 255, 255, 0.12);
	}

	.popover-item {
		display: flex;
		align-items: center;
		justify-content: space-between;
		width: 100%;
		padding: 8px 14px;
		cursor: pointer;
		font-family: var(--font-sans);
		font-size: 13px;
		font-weight: 400;
		color: var(--text-primary);
		background: none;
		border: none;
		text-align: left;
	}

	.popover-item:hover {
		background: var(--fill-selected-strong);
	}

	.popover-label {
		display: flex;
		align-items: center;
		gap: 8px;
	}

	.popover-label svg {
		width: 15px;
		height: 15px;
		stroke: currentColor;
		fill: none;
		stroke-width: 1.5;
		stroke-linecap: round;
		stroke-linejoin: round;
		flex-shrink: 0;
	}

	.popover-label svg.brand {
		stroke: none;
		fill: currentColor;
	}

	.popover-shortcut {
		font-size: 12px;
		font-weight: 400;
		color: var(--text-tertiary);
	}

	.popover-divider {
		height: 0.5px;
		background: var(--border-primary);
		margin: 4px 0;
	}
</style>
