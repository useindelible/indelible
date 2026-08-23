<script lang="ts">
	import { t } from '$lib/i18n';
	import type { FilterCondition } from '$lib/utils/filter-expression';
	import type { LibraryFilterFieldDef } from '$lib/utils/library-filter-fields';

	interface Props {
		condition: FilterCondition;
		fieldDef: LibraryFilterFieldDef;
		pickerOpen: boolean;
		onTogglePicker: () => void;
		onOperatorChange: (op: string) => void;
	}

	let { condition, fieldDef, pickerOpen, onTogglePicker, onOperatorChange }: Props = $props();

	const operatorLabel = $derived(
		fieldDef.ops.find((operator) => operator.value === condition.op)?.labelKey
	);
</script>

{#if fieldDef.ops.length > 0}
	<div class="op-picker-anchor">
		<button type="button" class="filter-op-btn" onclick={onTogglePicker}>
			{operatorLabel ? $t(operatorLabel) : condition.op}
		</button>

		{#if pickerOpen}
			<div class="op-picker">
				{#each fieldDef.ops as op (op.value)}
					<button
						type="button"
						class="op-picker-item"
						class:selected={condition.op === op.value}
						onclick={() => onOperatorChange(op.value)}
					>
						{$t(op.labelKey)}
					</button>
				{/each}
			</div>
		{/if}
	</div>
{/if}

<style>
	.op-picker-anchor {
		position: relative;
	}

	.filter-op-btn {
		font-size: 12.5px;
		font-weight: 400;
		letter-spacing: -0.01em;
		color: var(--text-tertiary);
		white-space: nowrap;
		background: none;
		border: none;
		padding: 4px 6px;
		border-radius: 6px;
		cursor: pointer;
		font-family: var(--font-sans);
	}

	.filter-op-btn:hover {
		background: var(--fill-hover);
	}

	.op-picker {
		position: absolute;
		top: calc(100% + 4px);
		left: 0;
		width: 160px;
		background: var(--bg-elevated);
		border: 0.5px solid var(--border-secondary);
		border-radius: var(--radius-lg);
		box-shadow: var(--shadow-3);
		z-index: 100;
		overflow: hidden;
		padding: 4px 0;
	}

	.op-picker-item {
		display: flex;
		align-items: center;
		padding: 7px 12px;
		font-size: 13px;
		font-weight: 400;
		color: var(--text-primary);
		cursor: pointer;
		letter-spacing: -0.01em;
		width: 100%;
		background: none;
		border: none;
		font-family: var(--font-sans);
		text-align: left;
	}

	.op-picker-item:hover {
		background: var(--fill-hover);
	}

	.op-picker-item.selected {
		color: var(--accent);
	}
</style>
