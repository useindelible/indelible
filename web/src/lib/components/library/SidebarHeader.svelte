<script lang="ts">
	import { t } from '$lib/i18n';
	interface Props {
		homeHref: string;
		popoverOpen: boolean;
		onHideSidebar: () => void;
		onAddClick: (e: MouseEvent) => void;
	}

	let { homeHref, popoverOpen, onHideSidebar, onAddClick }: Props = $props();
</script>

<div class="sidebar-header">
	<!-- eslint-disable-next-line svelte/no-navigation-without-resolve -- homeHref is resolved by the parent. -->
	<a href={homeHref} class="sidebar-logo">Indelible</a>
	<div class="sidebar-header-actions">
		<button
			type="button"
			class="sidebar-icon-btn"
			onclick={onHideSidebar}
			aria-label={$t('library_hide_sidebar')}
			title={$t('library_hide_sidebar')}
		>
			<svg
				viewBox="0 0 20 20"
				fill="none"
				stroke="currentColor"
				stroke-width="1.5"
				stroke-linecap="round"
				stroke-linejoin="round"
				aria-hidden="true"
			>
				<rect x="3" y="4" width="14" height="12" rx="1.5" />
				<line x1="8" y1="4" x2="8" y2="16" />
			</svg>
		</button>
		<button
			type="button"
			class="sidebar-add-btn"
			class:active={popoverOpen}
			aria-label={$t('library_add_to_library')}
			aria-expanded={popoverOpen}
			onclick={onAddClick}
		>
			<svg viewBox="0 0 24 24" aria-hidden="true">
				<line x1="12" y1="5" x2="12" y2="19" />
				<line x1="5" y1="12" x2="19" y2="12" />
			</svg>
		</button>
	</div>
</div>
