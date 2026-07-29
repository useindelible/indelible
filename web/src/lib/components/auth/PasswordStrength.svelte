<script lang="ts">
	interface Props {
		password: string;
	}

	let { password }: Props = $props();

	const strength = $derived.by(() => {
		if (!password) return { level: 0, label: '' };
		let score = 0;
		if (password.length >= 8) score++;
		if (password.length >= 12) score++;
		if (/[A-Z]/.test(password) && /[a-z]/.test(password)) score++;
		if (/\d/.test(password)) score++;
		if (/[^A-Za-z0-9]/.test(password)) score++;
		if (score <= 1) return { level: 1, label: 'Weak' };
		if (score <= 3) return { level: 2, label: 'Medium' };
		return { level: 3, label: 'Strong' };
	});
</script>

<div class="password-strength" aria-live="polite">
	<div
		class="strength-bar"
		role="progressbar"
		aria-valuemin="0"
		aria-valuemax="3"
		aria-valuenow={strength.level}
		aria-label="Password strength"
	>
		{#each [1, 2, 3] as segment (segment)}
			<span
				class="strength-segment"
				class:filled={strength.level >= segment}
				class:weak={strength.level === 1}
				class:medium={strength.level === 2}
				class:strong={strength.level === 3}
			></span>
		{/each}
	</div>
	{#if strength.label}
		<span
			class="strength-label"
			class:weak={strength.level === 1}
			class:medium={strength.level === 2}
			class:strong={strength.level === 3}
		>
			{strength.label}
		</span>
	{/if}
</div>

<style>
	.password-strength {
		margin-top: 8px;
	}

	.strength-bar {
		display: flex;
		gap: 4px;
	}

	.strength-segment {
		flex: 1;
		height: 4px;
		border-radius: 2px;
		background: var(--fill-hover);
		transition: background-color 0.2s ease;
	}

	.strength-segment.filled.weak {
		background: var(--destructive);
	}

	.strength-segment.filled.medium {
		background: var(--warning);
	}

	.strength-segment.filled.strong {
		background: var(--success);
	}

	.strength-label {
		display: block;
		margin-top: 4px;
		font-family: var(--font-sans);
		font-size: 12px;
		font-weight: 500;
		letter-spacing: -0.005em;
	}

	.strength-label.weak {
		color: var(--destructive);
	}

	.strength-label.medium {
		color: var(--warning);
	}

	.strength-label.strong {
		color: var(--success);
	}
</style>
