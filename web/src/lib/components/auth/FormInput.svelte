<script lang="ts">
	import type { HTMLInputAttributes } from 'svelte/elements';
	import { t } from '$lib/i18n';

	interface Props extends HTMLInputAttributes {
		label: string;
		error?: string;
		value: string;
		revealable?: boolean;
	}

	let {
		label,
		error,
		value = $bindable(),
		revealable = false,
		type = 'text',
		...rest
	}: Props = $props();

	let revealed = $state(false);
	const effectiveType = $derived(revealable && revealed ? 'text' : type);
</script>

<div class="form-field">
	<label class="form-label">
		<span class="form-label-text">{label}</span>
		<div class="input-wrap" class:has-toggle={revealable}>
			<input
				class="form-input"
				class:form-input-error={!!error}
				bind:value
				type={effectiveType}
				{...rest}
			/>
			{#if revealable}
				<button
					type="button"
					class="toggle-visibility"
					aria-label={revealed ? $t('auth_hide_password') : $t('auth_show_password')}
					onclick={() => (revealed = !revealed)}
				>
					{#if revealed}
						<svg
							width="20"
							height="20"
							viewBox="0 0 24 24"
							fill="none"
							stroke="currentColor"
							stroke-width="1.5"
							stroke-linecap="round"
							stroke-linejoin="round"
							aria-hidden="true"
						>
							<path
								d="M17.94 17.94A10.94 10.94 0 0 1 12 20c-7 0-11-8-11-8a19.78 19.78 0 0 1 4.06-5.94"
							/>
							<path
								d="M9.9 4.24A10.94 10.94 0 0 1 12 4c7 0 11 8 11 8a19.76 19.76 0 0 1-4.21 5.17"
							/>
							<path d="M1 1l22 22" />
							<path d="M14.12 14.12a3 3 0 1 1-4.24-4.24" />
						</svg>
					{:else}
						<svg
							width="20"
							height="20"
							viewBox="0 0 24 24"
							fill="none"
							stroke="currentColor"
							stroke-width="1.5"
							stroke-linecap="round"
							stroke-linejoin="round"
							aria-hidden="true"
						>
							<path d="M1 12s4-8 11-8 11 8 11 8-4 8-11 8S1 12 1 12z" />
							<circle cx="12" cy="12" r="3" />
						</svg>
					{/if}
				</button>
			{/if}
		</div>
	</label>
	{#if error}
		<p class="form-error" role="alert">{error}</p>
	{/if}
</div>

<style>
	.form-field {
		margin-bottom: 16px;
	}

	.form-label {
		display: block;
	}

	.form-label-text {
		display: block;
		font-family: var(--font-sans);
		font-size: 13px;
		font-weight: 500;
		letter-spacing: -0.01em;
		color: var(--text-secondary);
		margin-bottom: 6px;
	}

	.input-wrap {
		position: relative;
	}

	.form-input {
		display: block;
		width: 100%;
		padding: 10px 12px;
		font-family: var(--font-sans);
		font-size: 15px;
		letter-spacing: -0.01em;
		line-height: 1.5;
		color: var(--text-primary);
		background: var(--bg-primary);
		border: 1px solid var(--border-secondary);
		border-radius: var(--radius-sm);
		outline: none;
		transition:
			border-color 0.15s ease,
			box-shadow 0.15s ease;
		box-sizing: border-box;
	}

	.input-wrap.has-toggle .form-input {
		padding-right: 44px;
	}

	.form-input:focus {
		border-color: var(--accent);
		box-shadow: 0 0 0 3px var(--fill-selected);
	}

	.form-input::placeholder {
		color: var(--text-tertiary);
	}

	.form-input-error {
		border-color: var(--destructive);
	}

	.form-input-error:focus {
		border-color: var(--destructive);
		box-shadow: 0 0 0 3px rgba(255, 59, 48, 0.12);
	}

	.toggle-visibility {
		position: absolute;
		right: 8px;
		top: 50%;
		transform: translateY(-50%);
		display: inline-flex;
		align-items: center;
		justify-content: center;
		padding: 4px;
		background: none;
		border: none;
		color: var(--text-tertiary);
		cursor: pointer;
		border-radius: var(--radius-xs, 4px);
		transition: color 0.15s ease;
	}

	.toggle-visibility:hover {
		color: var(--text-secondary);
	}

	.toggle-visibility:focus-visible {
		outline: 2px solid var(--accent);
		outline-offset: 1px;
	}

	.form-error {
		margin: 6px 0 0;
		font-family: var(--font-sans);
		font-size: 12px;
		font-weight: 400;
		letter-spacing: -0.005em;
		line-height: 1.4;
		color: var(--destructive);
	}
</style>
