<script lang="ts">
	import SettingsGroup from '$lib/components/settings/SettingsGroup.svelte';

	const shortcutColumns = [
		{
			label: 'Triage',
			rows: [
				['Move to Inbox', ['1']],
				['Move to Later', ['2']],
				['Move to Archive', ['3']],
				['Archive selected', ['A']]
			]
		},
		{
			label: 'Reading',
			rows: [
				['Open in reader', ['↵']],
				['Next item', ['J']],
				['Previous item', ['K']],
				['Mark unread', ['U']]
			]
		},
		{
			label: 'Global',
			rows: [
				['Search', ['⌘', 'K']],
				['Refresh feeds', ['R']],
				['Toggle dark mode', ['⌘', '⌥', 'D']],
				['Help', ['?']]
			]
		}
	];
</script>

<SettingsGroup title="Keyboard shortcuts" meta="Read-only · we'll add bindings in a later release">
	<div class="group-card">
		<div class="kbd-grid">
			{#each shortcutColumns as column (column.label)}
				<div class="kbd-col">
					<div class="kbd-col-label">{column.label}</div>
					{#each column.rows as row (row[0])}
						<div class="kbd-row">
							<span class="kbd-name">{row[0]}</span>
							<span class="kbd-keys">
								{#each row[1] as key (key)}
									<span class="kbd">{key}</span>
								{/each}
							</span>
						</div>
					{/each}
				</div>
			{/each}
		</div>
	</div>
</SettingsGroup>

<style>
	.group-card {
		background: var(--card-bg);
		border-radius: 14px;
		overflow: hidden;
		box-shadow: inset 0 0 0 0.5px var(--border-primary);
		/* Column count depends on the card's width, which two collapsible
		   sidebars decide — query the card, not the viewport. */
		container-type: inline-size;
		container-name: kbd-card;
	}

	.kbd-grid {
		display: grid;
		grid-template-columns: repeat(3, 1fr);
		column-gap: 24px;
		row-gap: 4px;
		padding: 6px;
	}

	.kbd-col {
		display: flex;
		flex-direction: column;
		min-width: 0;
	}

	.kbd-col-label {
		font-family: var(--font-sans);
		font-size: 10.5px;
		font-weight: 600;
		letter-spacing: 0.06em;
		text-transform: uppercase;
		color: var(--text-tertiary);
		padding: 6px 12px 8px;
		border-bottom: 0.5px solid var(--border-hairline);
		margin-bottom: 4px;
	}

	.kbd-row {
		display: flex;
		align-items: center;
		justify-content: space-between;
		padding: 8px 12px;
		gap: 10px;
		min-height: 32px;
	}

	.kbd-row + .kbd-row {
		border-top: 0.5px solid var(--border-hairline);
	}

	.kbd-name {
		font-family: var(--font-sans);
		font-size: 13px;
		color: var(--text-primary);
		flex: 1;
		min-width: 0;
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}

	.kbd-keys {
		display: inline-flex;
		gap: 3px;
		flex-shrink: 0;
	}

	.kbd {
		display: inline-flex;
		align-items: center;
		justify-content: center;
		min-width: 22px;
		height: 22px;
		padding: 0 6px;
		border-radius: 5px;
		background: var(--bg-elevated);
		color: var(--text-secondary);
		font-family: 'SF Mono', 'Fira Code', Menlo, ui-monospace, monospace;
		font-size: 11px;
		font-weight: 600;
		box-shadow: var(--shadow-1);
	}

	@container kbd-card (max-width: 659px) {
		.kbd-grid {
			grid-template-columns: 1fr 1fr;
			row-gap: 12px;
		}
	}

	@container kbd-card (max-width: 439px) {
		.kbd-grid {
			grid-template-columns: 1fr;
		}
	}
</style>
