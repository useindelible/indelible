<script lang="ts">
	import type { TagResponse } from '$lib/api/generated/types.gen';
	import type { FilterCondition } from '$lib/utils/filter-expression';
	import type { LibraryFilterFieldDef } from '$lib/utils/library-filter-fields';
	import { filterCollections, type FilterCollection } from './filter-bar-model';

	interface Props {
		condition: FilterCondition;
		fieldDef: LibraryFilterFieldDef;
		collections: FilterCollection[];
		tagSuggestions: TagResponse[];
		pickerOpen: boolean;
		valueLabel: string;
		valuePlaceholder: boolean;
		onTogglePicker: () => void;
		onToggleBoolean: () => void;
		onValueChange: (value: FilterCondition['value']) => void;
		onToggleMultiValue: (value: string) => void;
		onSearchTags: (query: string) => void;
		onClose: () => void;
	}

	let {
		condition,
		fieldDef,
		collections,
		tagSuggestions,
		pickerOpen,
		valueLabel,
		valuePlaceholder,
		onTogglePicker,
		onToggleBoolean,
		onValueChange,
		onToggleMultiValue,
		onSearchTags,
		onClose
	}: Props = $props();

	const filteredCollections = $derived(filterCollections(collections, condition.value));
</script>

{#if fieldDef.valueType === 'boolean'}
	<button type="button" class="filter-value-btn" onclick={onToggleBoolean}>
		{valueLabel}
	</button>
{:else}
	<div class="value-picker-anchor">
		<button
			type="button"
			class="filter-value-btn"
			class:placeholder={valuePlaceholder}
			onclick={onTogglePicker}
		>
			{valueLabel}
			<svg viewBox="0 0 24 24" aria-hidden="true"><polyline points="6 9 12 15 18 9" /></svg>
		</button>

		{#if pickerOpen}
			<div class="value-picker">
				{#if fieldDef.valueType === 'select'}
					{#each fieldDef.options ?? [] as opt (opt.value)}
						{#if condition.op === 'in'}
							<button
								type="button"
								class="value-picker-item"
								class:selected={Array.isArray(condition.value) &&
									condition.value.includes(opt.value)}
								onclick={() => onToggleMultiValue(opt.value)}
							>
								{opt.label}
								{#if Array.isArray(condition.value) && condition.value.includes(opt.value)}
									<span class="check-icon">
										<svg viewBox="0 0 24 24" aria-hidden="true"
											><polyline points="20 6 9 17 4 12" /></svg
										>
									</span>
								{/if}
							</button>
						{:else}
							<button
								type="button"
								class="value-picker-item"
								class:selected={condition.value === opt.value}
								onclick={() => {
									onValueChange(opt.value);
									onClose();
								}}
							>
								{opt.label}
								{#if condition.value === opt.value}
									<span class="check-icon">
										<svg viewBox="0 0 24 24" aria-hidden="true"
											><polyline points="20 6 9 17 4 12" /></svg
										>
									</span>
								{/if}
							</button>
						{/if}
					{/each}
				{:else if fieldDef.valueType === 'text' && condition.field === 'tag'}
					<div class="value-input-wrap">
						<input
							type="text"
							class="value-text-input"
							placeholder="Search tags..."
							value={typeof condition.value === 'string' ? condition.value : ''}
							autofocus
							oninput={(event) => {
								const value = (event.target as HTMLInputElement).value;
								onValueChange(value);
								onSearchTags(value);
							}}
						/>
						{#if typeof condition.value === 'string' && condition.value}
							<button
								type="button"
								class="value-input-clear"
								onclick={() => {
									onValueChange('');
									onSearchTags('');
								}}
								aria-label="Clear"
							>
								<svg viewBox="0 0 24 24" aria-hidden="true">
									<line x1="18" y1="6" x2="6" y2="18" />
									<line x1="6" y1="6" x2="18" y2="18" />
								</svg>
							</button>
						{/if}
					</div>
					{#each tagSuggestions as tag (tag.id)}
						<button
							type="button"
							class="value-picker-item"
							class:selected={condition.value === tag.name}
							onclick={() => {
								onValueChange(tag.name);
								onClose();
							}}
						>
							{tag.name}
							{#if condition.value === tag.name}
								<span class="check-icon">
									<svg viewBox="0 0 24 24" aria-hidden="true"
										><polyline points="20 6 9 17 4 12" /></svg
									>
								</span>
							{/if}
						</button>
					{/each}
				{:else if fieldDef.valueType === 'text' && condition.field === 'collection'}
					<div class="value-input-wrap">
						<input
							type="text"
							class="value-text-input"
							placeholder="Search collections..."
							value={typeof condition.value === 'string' ? condition.value : ''}
							autofocus
							oninput={(event) => onValueChange((event.target as HTMLInputElement).value)}
						/>
						{#if typeof condition.value === 'string' && condition.value}
							<button
								type="button"
								class="value-input-clear"
								onclick={() => onValueChange('')}
								aria-label="Clear"
							>
								<svg viewBox="0 0 24 24" aria-hidden="true">
									<line x1="18" y1="6" x2="6" y2="18" />
									<line x1="6" y1="6" x2="18" y2="18" />
								</svg>
							</button>
						{/if}
					</div>
					{#each filteredCollections as collection (collection.id)}
						<button
							type="button"
							class="value-picker-item"
							class:selected={condition.value === collection.id}
							onclick={() => {
								onValueChange(collection.id);
								onClose();
							}}
						>
							{collection.name}
							{#if condition.value === collection.id}
								<span class="check-icon">
									<svg viewBox="0 0 24 24" aria-hidden="true"
										><polyline points="20 6 9 17 4 12" /></svg
									>
								</span>
							{/if}
						</button>
					{/each}
				{:else if fieldDef.valueType === 'text'}
					<div class="value-input-wrap">
						<input
							type="text"
							class="value-text-input"
							placeholder="Enter value..."
							value={typeof condition.value === 'string' ? condition.value : ''}
							autofocus
							oninput={(event) => onValueChange((event.target as HTMLInputElement).value)}
							onkeydown={(event) => {
								if (event.key === 'Enter') onClose();
							}}
						/>
						{#if typeof condition.value === 'string' && condition.value}
							<button
								type="button"
								class="value-input-clear"
								onclick={() => onValueChange('')}
								aria-label="Clear"
							>
								<svg viewBox="0 0 24 24" aria-hidden="true">
									<line x1="18" y1="6" x2="6" y2="18" />
									<line x1="6" y1="6" x2="18" y2="18" />
								</svg>
							</button>
						{/if}
					</div>
				{:else if fieldDef.valueType === 'number'}
					<div class="value-input-wrap">
						<input
							type="number"
							class="value-text-input"
							placeholder="0"
							value={typeof condition.value === 'number' ? condition.value : 0}
							autofocus
							oninput={(event) => {
								onValueChange(parseInt((event.target as HTMLInputElement).value) || 0);
							}}
							onkeydown={(event) => {
								if (event.key === 'Enter') onClose();
							}}
						/>
						{#if typeof condition.value === 'number' && condition.value !== 0}
							<button
								type="button"
								class="value-input-clear"
								onclick={() => onValueChange(0)}
								aria-label="Clear"
							>
								<svg viewBox="0 0 24 24" aria-hidden="true">
									<line x1="18" y1="6" x2="6" y2="18" />
									<line x1="6" y1="6" x2="18" y2="18" />
								</svg>
							</button>
						{/if}
					</div>
				{:else if fieldDef.valueType === 'date'}
					<div class="value-input-wrap">
						<input
							type="date"
							class="value-text-input"
							value={typeof condition.value === 'string' ? condition.value : ''}
							oninput={(event) => onValueChange((event.target as HTMLInputElement).value)}
						/>
					</div>
				{/if}
			</div>
		{/if}
	</div>
{/if}

<style>
	.value-picker-anchor {
		position: relative;
	}

	.filter-value-btn {
		display: flex;
		align-items: center;
		gap: 5px;
		padding: 4px 10px;
		border-radius: 6px;
		font-size: 12.5px;
		font-weight: 500;
		letter-spacing: -0.01em;
		color: var(--accent);
		background: var(--fill-selected);
		cursor: pointer;
		white-space: nowrap;
		border: none;
		font-family: var(--font-sans);
	}

	.filter-value-btn:hover {
		background: var(--fill-selected-strong);
	}

	.filter-value-btn svg {
		width: 10px;
		height: 10px;
		stroke: currentColor;
		fill: none;
		stroke-width: 2;
		stroke-linecap: round;
		stroke-linejoin: round;
	}

	.filter-value-btn.placeholder {
		color: var(--text-quaternary);
		background: var(--fill-secondary);
		font-weight: 400;
	}

	.value-picker {
		position: absolute;
		top: calc(100% + 4px);
		left: 0;
		width: 200px;
		background: var(--bg-elevated);
		border: 0.5px solid var(--border-secondary);
		border-radius: var(--radius-lg);
		box-shadow: var(--shadow-3);
		z-index: 100;
		overflow: hidden;
		padding: 4px 0;
	}

	.value-picker-item {
		display: flex;
		align-items: center;
		gap: 8px;
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

	.value-picker-item:hover {
		background: var(--fill-hover);
	}

	.value-picker-item.selected {
		color: var(--accent);
	}

	.check-icon {
		margin-left: auto;
		width: 14px;
		height: 14px;
		color: var(--accent);
		display: flex;
		align-items: center;
		justify-content: center;
	}

	.check-icon svg {
		width: 14px;
		height: 14px;
		stroke: currentColor;
		fill: none;
		stroke-width: 2;
		stroke-linecap: round;
		stroke-linejoin: round;
	}

	.value-input-wrap {
		display: flex;
		align-items: center;
		gap: 4px;
		padding: 4px 8px;
		border-bottom: 0.5px solid var(--border-primary);
		margin-bottom: 4px;
	}

	.value-text-input {
		flex: 1;
		min-width: 0;
		height: 32px;
		border-radius: var(--radius-sm);
		background: var(--bg-secondary);
		border: none;
		padding: 0 10px;
		font-size: 13px;
		font-weight: 400;
		color: var(--text-primary);
		letter-spacing: -0.01em;
		font-family: var(--font-sans);
	}

	.value-text-input::placeholder {
		color: var(--text-tertiary);
	}

	.value-text-input:focus {
		outline: none;
	}

	.value-input-clear {
		flex-shrink: 0;
		width: 20px;
		height: 20px;
		display: flex;
		align-items: center;
		justify-content: center;
		border-radius: 4px;
		color: var(--text-tertiary);
		background: none;
		border: none;
		padding: 0;
		cursor: pointer;
	}

	.value-input-clear:hover {
		background: var(--fill-hover);
		color: var(--text-primary);
	}

	.value-input-clear svg {
		width: 11px;
		height: 11px;
		stroke: currentColor;
		fill: none;
		stroke-width: 2;
		stroke-linecap: round;
		stroke-linejoin: round;
	}
</style>
