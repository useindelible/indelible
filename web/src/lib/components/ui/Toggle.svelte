<script lang="ts">
	interface Props {
		checked?: boolean;
		label?: string;
		disabled?: boolean;
	}

	let { checked = $bindable(false), label, disabled = false }: Props = $props();
</script>

<label class="toggle-wrapper" class:disabled>
	<input type="checkbox" class="toggle" bind:checked {disabled} aria-label={label} />
	{#if label}
		<span class="toggle-label">{label}</span>
	{/if}
</label>

<style>
	.toggle-wrapper {
		display: inline-flex;
		align-items: center;
		gap: 10px;
		cursor: pointer;
		user-select: none;
	}

	.toggle-wrapper.disabled {
		opacity: 0.5;
		cursor: not-allowed;
	}

	.toggle {
		-webkit-appearance: none;
		appearance: none;
		position: relative;
		width: 44px;
		height: 26px;
		border-radius: 13px;
		background: var(--border-secondary);
		cursor: pointer;
		flex-shrink: 0;
		transition: background 0.2s ease;
		margin: 0;
	}

	.toggle::after {
		content: '';
		position: absolute;
		top: 3px;
		left: 3px;
		width: 20px;
		height: 20px;
		border-radius: 50%;
		background: var(--text-on-color);
		box-shadow: 0 1px 3px rgba(0, 0, 0, 0.2);
		transition: transform 0.2s ease;
	}

	.toggle:checked {
		background: var(--accent);
	}

	.toggle:checked::after {
		transform: translateX(18px);
	}

	.toggle:disabled {
		cursor: not-allowed;
	}

	.toggle-label {
		font-family: var(--font-sans);
		font-size: 15px;
		letter-spacing: -0.01em;
		color: var(--text-primary);
	}
</style>
