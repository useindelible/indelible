<!--
  Theme toggle.

  A Svelte island rather than an inline script because it owns real state: the
  button's label and pressed state have to track the theme, including when the
  docs (which write the same key) changed it in another tab.

  The theme itself is applied before this hydrates — see PRE_PAINT_SNIPPET —
  so this component never causes a flash; it only reflects and flips.
-->
<script lang="ts">
	import { currentTheme, cycleTheme, THEME_KEY, STARLIGHT_KEY } from '../../lib/theme';
	import { THEMES, DEFAULT_THEME } from '../../data/themes';

	let theme = $state<string>(DEFAULT_THEME);

	/** The palette the button will move to, which is what the label announces. */
	const next = $derived(
		THEMES[(THEMES.findIndex((t) => t.id === theme) + 1) % THEMES.length],
	);
	const label = $derived(`Switch to ${next.label.toLowerCase()} theme`);
	const isDark = $derived(
		(THEMES.find((t) => t.id === theme) ?? THEMES[0]).base === 'dark',
	);

	$effect(() => {
		theme = currentTheme();

		// The docs write the same keys, so a change made there in another tab
		// should be reflected here rather than silently disagreeing.
		const onStorage = (event: StorageEvent) => {
			if (event.key === THEME_KEY || event.key === STARLIGHT_KEY) {
				theme = currentTheme();
			}
		};
		addEventListener('storage', onStorage);
		return () => removeEventListener('storage', onStorage);
	});
</script>

<button
	class="tgl"
	type="button"
	aria-label={label}
	title={label}
	onclick={() => (theme = cycleTheme())}
>
	<svg
		class="i-sun"
		class:on={isDark}
		viewBox="0 0 24 24"
		width="15"
		height="15"
		fill="none"
		stroke="currentColor"
		stroke-width="1.6"
		stroke-linecap="round"
		aria-hidden="true"
	>
		<circle cx="12" cy="12" r="4.2" />
		<path
			d="M12 2.6v2.2M12 19.2v2.2M2.6 12h2.2M19.2 12h2.2M5.3 5.3l1.6 1.6M17.1 17.1l1.6 1.6M18.7 5.3l-1.6 1.6M6.9 17.1l-1.6 1.6"
		/>
	</svg>

	<svg
		class="i-moon"
		class:on={!isDark}
		viewBox="0 0 24 24"
		width="15"
		height="15"
		fill="none"
		stroke="currentColor"
		stroke-width="1.6"
		stroke-linecap="round"
		stroke-linejoin="round"
		aria-hidden="true"
	>
		<path d="M20 14.2A8.2 8.2 0 1 1 9.8 4a6.6 6.6 0 0 0 10.2 10.2Z" />
	</svg>
</button>

<style>
	.tgl {
		position: relative;
		width: 38px;
		height: 38px;
		display: grid;
		place-items: center;
		border: 1px solid var(--line);
		border-radius: 999px;
		background: transparent;
		color: var(--dim);
		transition:
			color var(--d-micro) var(--e-spring),
			border-color var(--d-micro) var(--e-spring),
			transform var(--d-micro) var(--e-spring);
	}

	.tgl:hover {
		color: var(--ink);
		border-color: var(--line-2);
	}

	.tgl:active {
		transform: scale(0.92);
	}

	svg {
		position: absolute;
		opacity: 0;
		transform: rotate(40deg) scale(0.6);
		transition:
			opacity 0.3s var(--e-out),
			transform 0.45s var(--e-spring);
	}

	svg.on {
		opacity: 1;
		transform: none;
	}

	.i-moon:not(.on) {
		transform: rotate(-40deg) scale(0.6);
	}
</style>
