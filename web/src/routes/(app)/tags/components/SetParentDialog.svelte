<script lang="ts">
	import type { TagResponse } from '$lib/api/generated/types.gen';

	interface Props {
		options: TagResponse[];
		selectedParentId: string;
		onClose: () => void;
		onParentChange: (id: string) => void;
		onSubmit: () => void;
	}

	let { options, selectedParentId, onClose, onParentChange, onSubmit }: Props = $props();
</script>

<div
	class="cmd-backdrop"
	role="dialog"
	aria-modal="true"
	aria-label="Set parent tag"
	tabindex="-1"
	onclick={onClose}
	onkeydown={(event) => {
		if (event.key === 'Escape') onClose();
	}}
>
	<div
		class="cmd-card"
		role="none"
		onclick={(event) => event.stopPropagation()}
		onkeydown={() => {}}
	>
		<div class="cmd-body">
			<select
				class="cmd-select"
				value={selectedParentId}
				onchange={(event) => onParentChange(event.currentTarget.value)}
			>
				<option value="">None — top-level</option>
				{#each options as option (option.id)}
					<option value={option.id}>{option.name}</option>
				{/each}
			</select>
		</div>
		<div class="cmd-controls">
			<button type="button" class="cmd-secondary" onclick={onClose}>Cancel</button>
			<button type="button" class="cmd-action" onclick={onSubmit}>Save</button>
		</div>
	</div>
</div>

<style>
	.cmd-backdrop {
		position: fixed;
		inset: 0;
		z-index: 300;
		display: flex;
		align-items: flex-start;
		justify-content: center;
		background: var(--overlay-backdrop);
		backdrop-filter: blur(4px);
		-webkit-backdrop-filter: blur(4px);
		padding-top: 80px;
		box-sizing: border-box;
	}
	.cmd-card {
		width: 460px;
		max-width: calc(100vw - 32px);
		border-radius: 14px;
		background: var(--bg-elevated);
		box-shadow:
			0 24px 80px rgba(0, 0, 0, 0.22),
			0 0 0 0.5px rgba(0, 0, 0, 0.06);
		overflow: hidden;
	}
	.cmd-body {
		padding: 16px 16px 4px;
		display: flex;
		flex-direction: column;
		gap: 8px;
	}
	.cmd-controls {
		padding: 10px 16px 14px;
		display: flex;
		align-items: center;
		gap: 6px;
	}
	.cmd-select {
		width: 100%;
		height: 40px;
		border: none;
		border-radius: 10px;
		background: var(--bg-secondary);
		color: var(--text-primary);
		padding: 0 28px 0 12px;
		font: 500 14px/1.4 var(--font-sans);
		cursor: pointer;
		appearance: none;
		background-image: url("data:image/svg+xml,%3Csvg width='10' height='6' viewBox='0 0 10 6' fill='none' xmlns='http://www.w3.org/2000/svg'%3E%3Cpath d='M1 1l4 4 4-4' stroke='%2386868B' stroke-width='1.5' stroke-linecap='round' stroke-linejoin='round'/%3E%3C/svg%3E");
		background-repeat: no-repeat;
		background-position: right 10px center;
		outline: none;
		box-sizing: border-box;
		letter-spacing: -0.01em;
	}
	button {
		font: inherit;
		cursor: pointer;
	}
	.cmd-secondary,
	.cmd-action {
		border-radius: 980px;
		padding: 6px 14px;
		font-size: 13px;
		font-weight: 500;
		letter-spacing: -0.01em;
	}
	.cmd-secondary {
		border: 1px solid var(--border-primary);
		background: transparent;
		color: var(--text-secondary);
	}
	.cmd-action {
		margin-left: auto;
		border: 0;
		background: var(--accent);
		color: var(--text-on-color);
		padding: 6px 16px;
		font-weight: 600;
	}
</style>
