<script lang="ts">
	import type { SmartListResponse, TriageModeDto } from '$lib/api/generated/types.gen';
	import { t } from '$lib/i18n';
	import type { GroupBy } from '$lib/stores/library.svelte';

	interface Props {
		smartList?: SmartListResponse;
		allSmartLists: SmartListResponse[];
		groupBy: GroupBy;
		triageMode?: TriageModeDto;
		onGroupByChange: (gb: GroupBy) => void;
		onSwitchView: (id: string) => void;
		onClearView: () => void;
		onNewView: () => void;
		onRenameView: () => void;
		onDeleteView: () => void;
		onEditFilter: () => void;
		showCountBadge: boolean;
		onToggleCountBadge: () => void;
		onMarkAllSeen: () => void;
		onArchiveAll: () => void;
		onClose: () => void;
	}

	let {
		smartList,
		allSmartLists,
		groupBy,
		triageMode = 'focus',
		onGroupByChange,
		onSwitchView,
		onClearView,
		onNewView,
		onRenameView,
		onDeleteView,
		onEditFilter,
		showCountBadge,
		onToggleCountBadge,
		onMarkAllSeen,
		onArchiveAll,
		onClose
	}: Props = $props();

	const triageGroupDescription = $derived(
		$t(
			triageMode === 'manual'
				? 'library_view_triage_manual_description'
				: 'library_view_triage_focus_description'
		)
	);

	let confirmDelete = $state(false);

	function handleDelete() {
		if (confirmDelete) {
			onDeleteView();
		} else {
			confirmDelete = true;
		}
	}

	$effect(() => {
		function handleClickOutside(e: MouseEvent) {
			const target = e.target as HTMLElement;
			if (
				!target.closest('.vp') &&
				!target.closest('.view-dropdown-btn') &&
				!target.closest('.content-type-trigger')
			) {
				onClose();
			}
		}

		function handleKeydown(e: KeyboardEvent) {
			if (e.key === 'Escape') onClose();
		}

		document.addEventListener('mousedown', handleClickOutside);
		document.addEventListener('keydown', handleKeydown);

		return () => {
			document.removeEventListener('mousedown', handleClickOutside);
			document.removeEventListener('keydown', handleKeydown);
		};
	});
</script>

<div class="vp">
	{#if allSmartLists.length > 0 || smartList}
		<!-- VIEWS -->
		<div class="vp-section">{$t('library_view_views')}</div>
		<button
			type="button"
			class="vp-item"
			class:active={!smartList}
			onclick={() => {
				if (smartList) {
					onClearView();
					onClose();
				}
			}}
		>
			<span class="vp-dot" style:background={!smartList ? 'var(--accent)' : 'var(--text-tertiary)'}
			></span>
			{$t('common_library')}
			{#if !smartList}
				<span class="vp-check">
					<svg viewBox="0 0 24 24" aria-hidden="true"><polyline points="20 6 9 17 4 12" /></svg>
				</span>
			{/if}
		</button>
		{#each allSmartLists as sl (sl.id)}
			<button
				type="button"
				class="vp-item"
				class:active={sl.id === smartList?.id}
				onclick={() => {
					if (sl.id !== smartList?.id) onSwitchView(sl.id);
				}}
			>
				<span
					class="vp-dot"
					style:background={sl.id === smartList?.id ? 'var(--accent)' : 'var(--text-tertiary)'}
				></span>
				{sl.name}
				{#if sl.id === smartList?.id}
					<span class="vp-check">
						<svg viewBox="0 0 24 24" aria-hidden="true"><polyline points="20 6 9 17 4 12" /></svg>
					</span>
				{/if}
			</button>
		{/each}
		<button type="button" class="vp-item create" onclick={onNewView}>
			<span class="vp-icon">
				<svg viewBox="0 0 24 24" aria-hidden="true">
					<line x1="12" y1="5" x2="12" y2="19" />
					<line x1="5" y1="12" x2="19" y2="12" />
				</svg>
			</span>
			{$t('library_view_new_from_filters')}
		</button>

		<div class="vp-divider"></div>
	{/if}

	<!-- GROUP BY -->
	<div class="vp-section">{$t('library_view_group_by')}</div>

	<button
		type="button"
		class="vp-groupby-item"
		class:active={groupBy === 'triage'}
		onclick={() => onGroupByChange('triage')}
	>
		<div class="vp-groupby-icon">
			<svg viewBox="0 0 24 24" aria-hidden="true"><path d="M3 6h18M3 12h12M3 18h7" /></svg>
		</div>
		<div class="vp-groupby-text">
			<span class="vp-groupby-name">{$t('library_view_triage')}</span>
			<span class="vp-groupby-desc">{triageGroupDescription}</span>
		</div>
		<div class="vp-groupby-radio"><div class="vp-groupby-radio-dot"></div></div>
	</button>

	<button
		type="button"
		class="vp-groupby-item"
		class:active={groupBy === 'read_status'}
		onclick={() => onGroupByChange('read_status')}
	>
		<div class="vp-groupby-icon">
			<svg viewBox="0 0 24 24" aria-hidden="true">
				<path d="M1 12s4-8 11-8 11 8 11 8-4 8-11 8-11-8-11-8z" />
				<circle cx="12" cy="12" r="3" />
			</svg>
		</div>
		<div class="vp-groupby-text">
			<span class="vp-groupby-name">{$t('library_view_read_status')}</span>
			<span class="vp-groupby-desc">{$t('library_view_read_status_description')}</span>
		</div>
		<div class="vp-groupby-radio"><div class="vp-groupby-radio-dot"></div></div>
	</button>

	<button
		type="button"
		class="vp-groupby-item"
		class:active={groupBy === 'none'}
		onclick={() => onGroupByChange('none')}
	>
		<div class="vp-groupby-icon">
			<svg viewBox="0 0 24 24" aria-hidden="true">
				<line x1="8" y1="6" x2="21" y2="6" />
				<line x1="8" y1="12" x2="21" y2="12" />
				<line x1="8" y1="18" x2="21" y2="18" />
				<line x1="3" y1="6" x2="3.01" y2="6" />
				<line x1="3" y1="12" x2="3.01" y2="12" />
				<line x1="3" y1="18" x2="3.01" y2="18" />
			</svg>
		</div>
		<div class="vp-groupby-text">
			<span class="vp-groupby-name">{$t('common_none')}</span>
			<span class="vp-groupby-desc">{$t('library_view_flat_description')}</span>
		</div>
		<div class="vp-groupby-radio"><div class="vp-groupby-radio-dot"></div></div>
	</button>

	<div class="vp-divider"></div>

	<!-- DISPLAY -->
	<div class="vp-section">{$t('common_display')}</div>

	<div class="vp-toggle-row">
		<span>{$t('library_view_count_badge')}</span>
		<!-- svelte-ignore a11y_click_events_have_key_events -->
		<!-- svelte-ignore a11y_no_static_element_interactions -->
		<div
			class="vp-toggle"
			class:on={showCountBadge}
			class:off={!showCountBadge}
			onclick={onToggleCountBadge}
		>
			<div class="vp-toggle-knob"></div>
		</div>
	</div>

	<div class="vp-divider"></div>

	{#if smartList}
		<!-- FILTER — smart lists only -->
		<div class="vp-section">{$t('common_filter')}</div>
		<button type="button" class="vp-item" onclick={onEditFilter}>
			<span class="vp-icon">
				<svg viewBox="0 0 24 24" aria-hidden="true">
					<polygon points="22 3 2 3 10 12.46 10 19 14 21 14 12.46 22 3" />
				</svg>
			</span>
			{$t('library_view_edit_filter')}
		</button>
		<button
			type="button"
			class="vp-item"
			onclick={() => {
				onRenameView();
				onClose();
			}}
		>
			<span class="vp-icon">
				<svg viewBox="0 0 24 24" aria-hidden="true">
					<path d="M11 4H4a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2v-7" />
					<path d="M18.5 2.5a2.121 2.121 0 0 1 3 3L12 15l-4 1 1-4 9.5-9.5z" />
				</svg>
			</span>
			{$t('library_view_rename')}
		</button>

		<div class="vp-divider"></div>
	{/if}

	<!-- ACTIONS -->
	<div class="vp-section">{$t('common_actions')}</div>
	<button
		type="button"
		class="vp-item"
		onclick={() => {
			onMarkAllSeen();
			onClose();
		}}
	>
		<span class="vp-icon">
			<svg viewBox="0 0 24 24" aria-hidden="true">
				<polyline points="9 11 12 14 22 4" />
				<path d="M21 12v7a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h11" />
			</svg>
		</span>
		{$t('library_view_mark_all_seen')}
	</button>
	<button
		type="button"
		class="vp-item"
		onclick={() => {
			onArchiveAll();
			onClose();
		}}
	>
		<span class="vp-icon">
			<svg viewBox="0 0 24 24" aria-hidden="true">
				<polyline points="21 8 21 21 3 21 3 8" />
				<rect x="1" y="3" width="22" height="5" rx="1" />
			</svg>
		</span>
		{$t('library_view_archive_all')}
	</button>

	{#if smartList}
		<div class="vp-divider"></div>
		<button type="button" class="vp-item danger" onclick={handleDelete}>
			<span class="vp-icon">
				<svg viewBox="0 0 24 24" aria-hidden="true">
					<polyline points="3 6 5 6 21 6" />
					<path
						d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2"
					/>
				</svg>
			</span>
			{$t(confirmDelete ? 'library_view_confirm_delete' : 'library_view_delete')}
		</button>
	{/if}
</div>

<style>
	.vp {
		position: absolute;
		top: 60px;
		left: 16px;
		width: 280px;
		background: var(--bg-elevated);
		border: 0.5px solid var(--border-secondary);
		border-radius: var(--radius-lg);
		box-shadow: var(--shadow-3);
		z-index: 100;
		padding: 6px 0;
		max-height: calc(100vh - 100px);
		overflow-y: auto;
	}

	.vp-section {
		font-size: 10px;
		font-weight: 600;
		letter-spacing: 0.06em;
		text-transform: uppercase;
		color: var(--text-quaternary);
		padding: 6px 12px 4px;
	}

	.vp-divider {
		height: 0.5px;
		background: var(--border-primary);
		margin: 6px 0;
	}

	.vp-item {
		display: flex;
		align-items: center;
		gap: 9px;
		padding: 6px 12px;
		margin: 1px 4px;
		border-radius: 6px;
		font-size: 13px;
		font-weight: 400;
		letter-spacing: -0.01em;
		color: var(--text-primary);
		cursor: pointer;
		background: none;
		border: none;
		font-family: var(--font-sans);
		text-align: left;
		width: calc(100% - 8px);
	}

	.vp-item:hover {
		background: var(--fill-hover);
	}

	.vp-item.active {
		color: var(--accent);
		font-weight: 500;
		background: var(--fill-selected);
	}

	.vp-item.danger {
		color: var(--destructive);
	}

	.vp-item.danger:hover {
		background: var(--fill-danger);
	}

	.vp-item.create {
		color: var(--accent);
		font-weight: 500;
	}

	.vp-dot {
		width: 7px;
		height: 7px;
		border-radius: var(--radius-circle);
		flex-shrink: 0;
	}

	.vp-icon {
		width: 15px;
		height: 15px;
		display: flex;
		align-items: center;
		justify-content: center;
		flex-shrink: 0;
	}

	.vp-icon svg {
		width: 15px;
		height: 15px;
		stroke: currentColor;
		fill: none;
		stroke-width: 1.5;
		stroke-linecap: round;
		stroke-linejoin: round;
	}

	.vp-check {
		margin-left: auto;
		width: 14px;
		height: 14px;
		display: flex;
		align-items: center;
		justify-content: center;
	}

	.vp-check svg {
		width: 14px;
		height: 14px;
		stroke: var(--accent);
		fill: none;
		stroke-width: 2;
		stroke-linecap: round;
		stroke-linejoin: round;
	}

	/* Group by */
	.vp-groupby-item {
		display: flex;
		align-items: center;
		gap: 10px;
		padding: 7px 12px;
		margin: 1px 4px;
		border-radius: 7px;
		cursor: pointer;
		background: none;
		border: none;
		font-family: var(--font-sans);
		text-align: left;
		width: calc(100% - 8px);
	}

	.vp-groupby-item:hover {
		background: var(--fill-hover);
	}

	.vp-groupby-item.active {
		background: var(--fill-selected);
	}

	.vp-groupby-icon {
		width: 28px;
		height: 28px;
		border-radius: 7px;
		background: var(--fill-secondary);
		display: flex;
		align-items: center;
		justify-content: center;
		flex-shrink: 0;
		color: var(--text-secondary);
	}

	.vp-groupby-item.active .vp-groupby-icon {
		background: var(--fill-selected-strong);
		color: var(--accent);
	}

	.vp-groupby-icon svg {
		width: 14px;
		height: 14px;
		stroke: currentColor;
		fill: none;
		stroke-width: 1.5;
		stroke-linecap: round;
		stroke-linejoin: round;
	}

	.vp-groupby-text {
		display: flex;
		flex-direction: column;
		gap: 1px;
		flex: 1;
		min-width: 0;
	}

	.vp-groupby-name {
		font-size: 13px;
		font-weight: 500;
		letter-spacing: -0.01em;
		color: var(--text-primary);
		line-height: 1.3;
	}

	.vp-groupby-item.active .vp-groupby-name {
		color: var(--accent);
	}

	.vp-groupby-desc {
		font-size: 11px;
		font-weight: 400;
		letter-spacing: -0.01em;
		color: var(--text-tertiary);
		line-height: 1.3;
	}

	.vp-groupby-radio {
		width: 15px;
		height: 15px;
		border-radius: 50%;
		border: 1.5px solid var(--text-quaternary);
		flex-shrink: 0;
		display: flex;
		align-items: center;
		justify-content: center;
	}

	.vp-groupby-item.active .vp-groupby-radio {
		border-color: var(--accent);
		background: var(--accent);
	}

	.vp-groupby-radio-dot {
		width: 5px;
		height: 5px;
		border-radius: 50%;
		background: white;
		display: none;
	}

	.vp-groupby-item.active .vp-groupby-radio-dot {
		display: block;
	}

	/* Display toggles */
	.vp-toggle-row {
		display: flex;
		align-items: center;
		justify-content: space-between;
		padding: 6px 12px;
		margin: 1px 4px;
		border-radius: 6px;
		font-size: 13px;
		font-weight: 400;
		letter-spacing: -0.01em;
		color: var(--text-primary);
	}

	.vp-toggle {
		width: 34px;
		height: 20px;
		border-radius: 10px;
		position: relative;
		cursor: pointer;
		flex-shrink: 0;
	}

	.vp-toggle.on {
		background: var(--accent);
	}

	.vp-toggle.off {
		background: var(--fill-secondary);
	}

	.vp-toggle-knob {
		width: 16px;
		height: 16px;
		border-radius: 50%;
		position: absolute;
		top: 2px;
		box-shadow: 0 1px 3px rgba(0, 0, 0, 0.3);
	}

	.vp-toggle.on .vp-toggle-knob {
		right: 2px;
		background: white;
	}

	.vp-toggle.off .vp-toggle-knob {
		left: 2px;
		background: var(--text-tertiary);
	}
</style>
