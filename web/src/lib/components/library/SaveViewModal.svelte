<script lang="ts">
	interface Props {
		initialName?: string;
		onClose: () => void;
		onSaved: (name: string) => void;
	}

	let { initialName = '', onClose, onSaved }: Props = $props();

	let name = $state(initialName);

	function handleSave() {
		const trimmed = name.trim();
		if (trimmed) {
			onSaved(trimmed);
		}
	}

	function handleBackdropClick(e: MouseEvent) {
		if (e.target === e.currentTarget) {
			onClose();
		}
	}

	function handleKeydown(e: KeyboardEvent) {
		if (e.key === 'Escape') {
			onClose();
		} else if (e.key === 'Enter' && name.trim()) {
			handleSave();
		}
	}

	$effect(() => {
		document.addEventListener('keydown', handleKeydown);
		return () => document.removeEventListener('keydown', handleKeydown);
	});
</script>

<!-- svelte-ignore a11y_no_static_element_interactions -->
<div class="cmd-backdrop" onmousedown={handleBackdropClick}>
	<!-- svelte-ignore a11y_click_events_have_key_events -->
	<!-- svelte-ignore a11y_no_static_element_interactions -->
	<div class="cmd-card" onclick={(e) => e.stopPropagation()}>
		<div class="cmd-input-zone">
			<div class="cmd-input-wrap">
				<svg class="cmd-icon" viewBox="0 0 24 24" aria-hidden="true">
					<path d="M19 21l-7-5-7 5V5a2 2 0 0 1 2-2h10a2 2 0 0 1 2 2z" />
				</svg>
				<!-- svelte-ignore a11y_autofocus -->
				<input class="cmd-input" placeholder="Name this view..." bind:value={name} autofocus />
			</div>
		</div>
		<div class="cmd-controls">
			<button type="button" class="cmd-secondary" onclick={onClose}>Cancel</button>
			<button type="button" class="cmd-action" onclick={handleSave} disabled={!name.trim()}>
				Save
			</button>
		</div>
	</div>
</div>

<style>
	.cmd-backdrop {
		position: absolute;
		inset: 0;
		background: var(--overlay-backdrop);
		backdrop-filter: blur(4px);
		-webkit-backdrop-filter: blur(4px);
		display: flex;
		align-items: flex-start;
		justify-content: center;
		padding-top: 80px;
		z-index: 300;
	}

	.cmd-card {
		width: 460px;
		background: var(--bg-elevated);
		border-radius: var(--radius-xl);
		box-shadow:
			0 24px 80px rgba(0, 0, 0, 0.55),
			0 0 0 0.5px var(--border-secondary);
	}

	.cmd-input-zone {
		padding: 8px 8px 0;
	}

	.cmd-input-wrap {
		position: relative;
	}

	.cmd-icon {
		position: absolute;
		left: 14px;
		top: 50%;
		transform: translateY(-50%);
		width: 16px;
		height: 16px;
		stroke: var(--text-tertiary);
		fill: none;
		stroke-width: 1.5;
		stroke-linecap: round;
		stroke-linejoin: round;
		pointer-events: none;
	}

	.cmd-input {
		width: 100%;
		height: 48px;
		border-radius: var(--radius-md);
		background: var(--bg-secondary);
		border: none;
		padding: 0 16px 0 40px;
		font-size: 15px;
		font-weight: 400;
		color: var(--text-primary);
		letter-spacing: -0.01em;
		font-family: var(--font-sans);
	}

	.cmd-input::placeholder {
		color: var(--text-tertiary);
	}

	.cmd-input:focus {
		outline: none;
	}

	.cmd-controls {
		display: flex;
		align-items: center;
		gap: 6px;
		padding: 10px 16px 14px;
	}

	.cmd-secondary {
		padding: 6px 16px;
		border-radius: var(--radius-full);
		border: 1px solid var(--border-primary);
		background: transparent;
		font-size: 13px;
		font-weight: 500;
		color: var(--text-secondary);
		cursor: pointer;
		letter-spacing: -0.01em;
		font-family: var(--font-sans);
	}

	.cmd-secondary:hover {
		background: var(--fill-hover);
	}

	.cmd-action {
		margin-left: auto;
		padding: 6px 16px;
		border-radius: var(--radius-full);
		border: none;
		font-size: 13px;
		font-weight: 600;
		color: var(--text-on-color);
		background: var(--accent);
		cursor: pointer;
		letter-spacing: -0.01em;
		font-family: var(--font-sans);
	}

	.cmd-action:hover {
		opacity: 0.88;
	}

	.cmd-action:disabled {
		opacity: 0.4;
		cursor: not-allowed;
	}
</style>
