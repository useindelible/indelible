<script lang="ts">
	import SettingsGroup from '$lib/components/settings/SettingsGroup.svelte';

	interface LocaleOption {
		value: string;
		label: string;
	}

	interface Props {
		locale: string;
		locales: LocaleOption[];
		onLocaleChange: (value: string) => void;
	}

	let { locale, locales, onLocaleChange }: Props = $props();
</script>

<SettingsGroup title="Locale">
	<div class="group-card">
		<div class="row">
			<div class="label-block">
				<div class="label">Language</div>
				<div class="hint">Indelible's chrome — menus, settings, and notifications.</div>
			</div>
			<select
				class="select"
				value={locale}
				aria-label="Language"
				onchange={(event) => onLocaleChange(event.currentTarget.value)}
			>
				{#each locales as item (item.value)}
					<option value={item.value}>{item.label}</option>
				{/each}
			</select>
		</div>
	</div>
</SettingsGroup>

<style>
	.group-card {
		background: var(--card-bg);
		border-radius: 14px;
		overflow: hidden;
		box-shadow: inset 0 0 0 0.5px var(--border-primary);
		container-type: inline-size;
		container-name: settings-card;
	}

	.row {
		display: flex;
		align-items: center;
		gap: 16px;
		padding: 14px 18px;
		min-height: 52px;
	}

	.label-block {
		flex: 1;
		min-width: 0;
	}

	.label {
		font-family: var(--font-sans);
		font-size: 13px;
		font-weight: 500;
		color: var(--text-primary);
		margin-bottom: 2px;
	}

	.hint {
		font-family: var(--font-sans);
		font-size: 12px;
		color: var(--text-secondary);
		line-height: 1.4;
	}

	.select {
		appearance: none;
		-webkit-appearance: none;
		background-color: var(--input-bg);
		color: var(--text-primary);
		border: none;
		outline: none;
		border-radius: 8px;
		padding: 8px 12px;
		font-family: var(--font-sans);
		font-size: 13.5px;
		box-shadow: var(--input-shadow);
		transition: box-shadow 120ms;
		width: 240px;
		background-image: url("data:image/svg+xml;utf8,<svg xmlns='http://www.w3.org/2000/svg' width='12' height='12' viewBox='0 0 24 24' fill='none' stroke='%236E6E73' stroke-width='1.8' stroke-linecap='round' stroke-linejoin='round'><polyline points='6 9 12 15 18 9'/></svg>");
		background-repeat: no-repeat;
		background-position: right 12px center;
		cursor: pointer;
	}

	.select:focus {
		box-shadow:
			var(--input-shadow),
			0 0 0 3px var(--accent-soft);
	}

	/* A 240px select + label can't share a narrow row; wrap so the select
	   drops under the label at full width. */
	@container settings-card (max-width: 539px) {
		.row {
			flex-wrap: wrap;
		}

		.label-block {
			flex: 1 1 100%;
		}

		.select {
			width: 100%;
		}
	}
</style>
