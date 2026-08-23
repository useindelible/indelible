<script lang="ts">
	import { t } from '$lib/i18n';
	import type { DashboardConfigItem } from '../dashboard-model';

	interface Props {
		sections: DashboardConfigItem[];
		types: DashboardConfigItem[];
		sectionOver: number | null;
		typeOver: number | null;
		onClose: () => void;
		onToggleSection: (id: string) => void;
		onToggleType: (id: string) => void;
		onSectionDragStart: (id: string) => void;
		onSectionDragOver: (event: DragEvent, index: number) => void;
		onSectionDrop: (event: DragEvent, index: number) => void;
		onSectionDragEnd: () => void;
		onTypeDragStart: (id: string) => void;
		onTypeDragOver: (event: DragEvent, index: number) => void;
		onTypeDrop: (event: DragEvent, index: number) => void;
		onTypeDragEnd: () => void;
	}

	let {
		sections,
		types,
		sectionOver,
		typeOver,
		onClose,
		onToggleSection,
		onToggleType,
		onSectionDragStart,
		onSectionDragOver,
		onSectionDrop,
		onSectionDragEnd,
		onTypeDragStart,
		onTypeDragOver,
		onTypeDrop,
		onTypeDragEnd
	}: Props = $props();
</script>

<div
	class="configure-backdrop"
	role="dialog"
	aria-modal="true"
	aria-label={$t('dashboard_config_title')}
	tabindex="-1"
	onclick={(event) => {
		if (event.target === event.currentTarget) onClose();
	}}
	onkeydown={(event) => {
		if (event.key === 'Escape') onClose();
	}}
>
	<div class="configure-popover">
		<div class="popover-header">
			<span class="popover-title">{$t('dashboard_config_title')}</span>
			<button type="button" class="popover-close" aria-label={$t('common_close')} onclick={onClose}>
				<svg
					viewBox="0 0 24 24"
					fill="none"
					stroke="currentColor"
					stroke-width="2"
					stroke-linecap="round"
					aria-hidden="true"
				>
					<line x1="18" y1="6" x2="6" y2="18" />
					<line x1="6" y1="6" x2="18" y2="18" />
				</svg>
			</button>
		</div>
		<div class="popover-body">
			<div class="popover-section">
				<div class="popover-section-hdr">
					<span class="popover-section-lbl">{$t('dashboard_config_sections')}</span>
					<span class="popover-section-hint">{$t('dashboard_config_drag_to_reorder')}</span>
				</div>
				<div class="config-rows">
					{#each sections as section, i (section.id)}
						{@const label = $t(section.labelKey)}
						<div
							class="config-row"
							class:row-off={!section.on}
							class:drag-over={sectionOver === i}
							draggable="true"
							ondragstart={() => onSectionDragStart(section.id)}
							ondragover={(event) => onSectionDragOver(event, i)}
							ondrop={(event) => onSectionDrop(event, i)}
							ondragend={onSectionDragEnd}
							role="listitem"
						>
							<div class="drag-handle" aria-hidden="true">
								{#each [0, 1, 2, 3, 4, 5] as dot (dot)}
									<div class="drag-dot"></div>
								{/each}
							</div>
							<span class="config-row-lbl">{label}</span>
							<button
								type="button"
								class="cfg-check"
								class:on={section.on}
								aria-label={$t(section.on ? 'dashboard_config_hide' : 'dashboard_config_show', {
									values: { label }
								})}
								onclick={() => onToggleSection(section.id)}
							>
								{#if section.on}
									<svg
										viewBox="0 0 24 24"
										fill="none"
										stroke="currentColor"
										stroke-width="2.5"
										stroke-linecap="round"
										stroke-linejoin="round"
										aria-hidden="true"
										width="10"
										height="10"
									>
										<polyline points="20 6 9 17 4 12" />
									</svg>
								{/if}
							</button>
						</div>
					{/each}
				</div>
			</div>

			<div class="popover-section">
				<div class="popover-section-hdr">
					<span class="popover-section-lbl">{$t('dashboard_config_content_types')}</span>
				</div>
				<div class="ct-rows">
					{#each types as type, i (type.id)}
						{@const label = $t(type.labelKey)}
						<div
							class="ct-row"
							class:drag-over={typeOver === i}
							draggable="true"
							ondragstart={() => onTypeDragStart(type.id)}
							ondragover={(event) => onTypeDragOver(event, i)}
							ondrop={(event) => onTypeDrop(event, i)}
							ondragend={onTypeDragEnd}
							role="listitem"
						>
							<div class="ct-icon-tile" aria-hidden="true">{label.charAt(0)}</div>
							<span class="ct-row-lbl">{label}</span>
							<button
								type="button"
								class="cfg-check"
								class:on={type.on}
								aria-label={$t(type.on ? 'dashboard_config_hide' : 'dashboard_config_show', {
									values: { label }
								})}
								onclick={() => onToggleType(type.id)}
							>
								{#if type.on}
									<svg
										viewBox="0 0 24 24"
										fill="none"
										stroke="currentColor"
										stroke-width="2.5"
										stroke-linecap="round"
										stroke-linejoin="round"
										aria-hidden="true"
										width="10"
										height="10"
									>
										<polyline points="20 6 9 17 4 12" />
									</svg>
								{/if}
							</button>
						</div>
					{/each}
				</div>
			</div>
		</div>
		<div class="popover-footer">
			<button type="button" class="btn-cancel" onclick={onClose}>{$t('common_cancel')}</button>
			<button type="button" class="btn-apply" onclick={onClose}>{$t('common_apply')}</button>
		</div>
	</div>
</div>

<style>
	.configure-backdrop {
		position: fixed;
		inset: 0;
		left: 220px;
		background: var(--overlay-backdrop);
		backdrop-filter: blur(7px) saturate(70%);
		-webkit-backdrop-filter: blur(7px) saturate(70%);
		display: flex;
		align-items: flex-start;
		justify-content: flex-end;
		padding: 66px 26px 0 0;
		z-index: 200;
	}

	.configure-popover {
		width: 288px;
		background: var(--bg-elevated);
		border-radius: 12px;
		box-shadow: var(--shadow-3);
		display: flex;
		flex-direction: column;
		overflow: hidden;
		max-height: calc(100vh - 90px);
	}

	.popover-header,
	.popover-footer {
		display: flex;
		align-items: center;
		flex-shrink: 0;
	}

	.popover-header {
		justify-content: space-between;
		padding: 11px 13px 9px;
		border-bottom: 0.5px solid var(--border-primary);
	}

	.popover-title {
		font-family: var(--font-sans);
		font-size: 13px;
		font-weight: 600;
		color: var(--text-primary);
	}

	.popover-close {
		width: 20px;
		height: 20px;
		border-radius: 50%;
		background: var(--fill-hover);
		border: none;
		cursor: pointer;
		display: flex;
		align-items: center;
		justify-content: center;
		padding: 0;
	}

	.popover-close svg {
		width: 9px;
		height: 9px;
		stroke: var(--text-secondary);
		stroke-width: 2;
	}

	.popover-body {
		overflow-y: auto;
		flex: 1;
	}

	.popover-section {
		padding: 9px 12px 8px;
	}

	.popover-section + .popover-section {
		border-top: 0.5px solid var(--border-primary);
	}

	.popover-section-hdr {
		display: flex;
		align-items: baseline;
		justify-content: space-between;
		margin-bottom: 5px;
	}

	.popover-section-lbl,
	.popover-section-hint {
		font-family: var(--font-sans);
		font-size: 10px;
	}

	.popover-section-lbl {
		font-weight: 600;
		letter-spacing: 0.07em;
		text-transform: uppercase;
		color: var(--text-tertiary);
	}

	.popover-section-hint {
		font-weight: 400;
		color: var(--text-quaternary);
	}

	.config-rows,
	.ct-rows {
		display: flex;
		flex-direction: column;
	}

	.config-row,
	.ct-row {
		display: flex;
		align-items: center;
		padding: 5px;
		border-radius: 6px;
		transition: background 100ms ease;
		cursor: default;
	}

	.config-row {
		gap: 7px;
	}

	.ct-row {
		gap: 9px;
	}

	.config-row:hover,
	.ct-row:hover {
		background: var(--fill-hover);
	}

	.config-row.row-off .config-row-lbl {
		color: var(--text-tertiary);
	}

	.config-row.drag-over,
	.ct-row.drag-over {
		position: relative;
	}

	.config-row.drag-over::before,
	.ct-row.drag-over::before {
		content: '';
		position: absolute;
		left: 0;
		right: 0;
		top: -1px;
		height: 2px;
		background: var(--accent);
		border-radius: 1px;
	}

	.drag-handle {
		display: grid;
		grid-template-columns: 1fr 1fr;
		gap: 2.5px;
		width: 9px;
		cursor: grab;
		opacity: 0.3;
		flex-shrink: 0;
	}

	.config-row:hover .drag-handle {
		opacity: 0.5;
	}

	.drag-dot {
		width: 2.5px;
		height: 2.5px;
		border-radius: 50%;
		background: var(--text-primary);
	}

	.config-row-lbl,
	.ct-row-lbl {
		font-family: var(--font-sans);
		font-size: 13px;
		font-weight: 400;
		color: var(--text-primary);
		flex: 1;
	}

	.ct-icon-tile {
		width: 20px;
		height: 20px;
		flex-shrink: 0;
		display: flex;
		align-items: center;
		justify-content: center;
		color: var(--text-secondary);
		font-size: 12px;
		font-weight: 600;
	}

	.cfg-check {
		width: 16px;
		height: 16px;
		border-radius: 4px;
		border: 1.5px solid var(--border-secondary);
		flex-shrink: 0;
		display: flex;
		align-items: center;
		justify-content: center;
		transition: all 100ms ease;
		background: transparent;
		cursor: pointer;
		padding: 0;
		color: var(--text-on-color);
		font-size: 11px;
		line-height: 1;
	}

	.cfg-check.on {
		background: var(--accent);
		border-color: var(--accent);
	}

	.popover-footer {
		justify-content: flex-end;
		gap: 6px;
		padding: 9px 12px;
		border-top: 0.5px solid var(--border-primary);
	}

	.btn-cancel,
	.btn-apply {
		padding: 5px 13px;
		border-radius: 980px;
		border: none;
		font-family: var(--font-sans);
		font-size: 12px;
		cursor: pointer;
	}

	.btn-cancel {
		background: var(--fill-hover);
		font-weight: 500;
		color: var(--text-primary);
	}

	.btn-apply {
		background: var(--accent);
		font-weight: 600;
		color: var(--text-on-color);
	}

	/* ---- Responsive ---- */

	/* The 220px inset assumes the docked desktop sidebar; below the desktop
	   breakpoint the sidebar is collapsed or hidden, so cover everything. */
	@media (max-width: 1099px) {
		.configure-backdrop {
			left: 0;
		}
	}

	/* Mobile: the popover becomes a full-screen takeover. */
	@media (max-width: 599px) {
		.configure-backdrop {
			padding: 0;
			align-items: stretch;
			justify-content: stretch;
		}

		.configure-popover {
			width: 100%;
			max-height: none;
			border-radius: 0;
		}

		.popover-header {
			padding: 14px 16px 12px;
		}

		.popover-section {
			padding: 12px 16px 10px;
		}

		.config-row,
		.ct-row {
			padding: 8px 5px;
		}

		.popover-footer {
			padding: 12px 16px;
		}

		.btn-cancel,
		.btn-apply {
			padding: 8px 18px;
			font-size: 13px;
		}
	}
</style>
