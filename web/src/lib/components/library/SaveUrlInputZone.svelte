<script lang="ts">
	import { t } from '$lib/i18n';
	interface Props {
		url: string;
		hasDuplicate: boolean;
		onSave: () => void;
		onClose: () => void;
	}

	let { url = $bindable(), hasDuplicate, onSave, onClose }: Props = $props();
</script>

<div class="cmd-input-zone">
	<div class="cmd-input-wrap">
		<svg class="cmd-icon" viewBox="0 0 24 24" aria-hidden="true">
			<path d="M10 13a5 5 0 0 0 7.54.54l3-3a5 5 0 0 0-7.07-7.07l-1.72 1.71" />
			<path d="M14 11a5 5 0 0 0-7.54-.54l-3 3a5 5 0 0 0 7.07 7.07l1.71-1.71" />
		</svg>
		<input
			class="cmd-input"
			class:has-value={hasDuplicate}
			type="text"
			placeholder={$t('library_save_url_paste')}
			bind:value={url}
			onkeydown={(event) => {
				if (event.key === 'Enter') onSave();
				if (event.key === 'Escape') onClose();
			}}
			aria-label={$t('library_url')}
		/>
	</div>
</div>

<style>
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
		border-radius: 10px;
		background: var(--bg-secondary);
		border: none;
		padding: 0 16px 0 40px;
		font-family: var(--font-sans);
		font-size: 15px;
		color: var(--text-primary);
		outline: none;
		letter-spacing: -0.01em;
		box-sizing: border-box;
	}

	.cmd-input::placeholder {
		color: var(--text-tertiary);
	}

	.cmd-input.has-value {
		font-size: 13px;
		color: var(--accent);
	}
</style>
