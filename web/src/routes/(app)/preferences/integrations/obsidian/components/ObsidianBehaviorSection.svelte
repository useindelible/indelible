<script lang="ts">
	import type { ObsidianSettingsDto } from '$lib/api';

	interface Props {
		settings: ObsidianSettingsDto;
		onChange: (patch: Partial<ObsidianSettingsDto>) => void;
	}

	let { settings, onChange }: Props = $props();
</script>

<section class="section">
	<div class="section-head">
		<h2 class="section-title">Behavior</h2>
		<p class="section-sub">How Indelible writes into your vault</p>
	</div>
	<div class="card card-stack">
		<div class="row">
			<div>
				<p class="row-title">Group in category folders</p>
				<p class="row-sub">
					PDFs use <code>books/</code>; videos and emails use <code>articles/</code>.
				</p>
			</div>
			<button
				type="button"
				class="toggle"
				class:is-on={settings.group_files_in_category_folders}
				aria-pressed={settings.group_files_in_category_folders}
				aria-label="Group in category folders"
				onclick={() =>
					onChange({
						group_files_in_category_folders: !settings.group_files_in_category_folders
					})}
			></button>
		</div>

		<div class="row-block">
			<div class="row-inner">
				<div>
					<p class="row-title">Export full Reader documents</p>
					<p class="row-sub">
						Write a generated companion file when Indelible has a prepared readable asset for the
						document. Note templates control the linked export note, not this companion body.
					</p>
				</div>
				<button
					type="button"
					class="toggle"
					class:is-on={settings.export_all_reader_documents}
					aria-pressed={settings.export_all_reader_documents}
					aria-label="Export full Reader documents"
					onclick={() =>
						onChange({
							export_all_reader_documents: !settings.export_all_reader_documents
						})}
				></button>
			</div>
			{#if settings.export_all_reader_documents}
				<div class="row-extra">
					Companion path pattern: <code>{'Indelible/{category}/{title} Full Text.md'}</code>.
					Documents without prepared readable text still sync their notes and highlights; they skip
					the full-text companion.
				</div>
			{/if}
		</div>

		<div class="row-block">
			<div class="row-inner">
				<div>
					<p class="row-title">Append sync notifications</p>
					<p class="row-sub">
						Add a timestamped line to <code>Indelible/Indelible Syncs.md</code> after each run.
					</p>
				</div>
				<button
					type="button"
					class="toggle"
					class:is-on={settings.sync_notifications}
					aria-pressed={settings.sync_notifications}
					aria-label="Append sync notifications"
					onclick={() => onChange({ sync_notifications: !settings.sync_notifications })}
				></button>
			</div>
			{#if settings.sync_notifications}
				<div class="row-extra">
					One bullet per sync, rendered with the <b>Sync notification</b> template below. Notifications
					are appended only after document writes succeed.
				</div>
			{/if}
		</div>
	</div>
</section>

<style>
	.section {
		margin-top: 28px;
	}
	.section-head {
		display: flex;
		align-items: baseline;
		justify-content: space-between;
		margin: 0 4px 12px;
		gap: 12px;
	}
	.section-title {
		font-size: 12px;
		font-weight: 550;
		color: var(--text-tertiary);
		text-transform: uppercase;
		letter-spacing: 0;
		margin: 0;
	}
	.section-sub,
	.row-sub {
		font-size: 12.5px;
		color: var(--text-tertiary);
		margin: 0;
	}
	.card {
		background: var(--bg-elevated);
		border: 1px solid var(--border-hairline);
		border-radius: 14px;
		box-shadow: var(--shadow-1);
		overflow: hidden;
	}
	.card-stack > * + * {
		border-top: 1px solid var(--border-hairline);
	}
	.row,
	.row-inner {
		display: grid;
		grid-template-columns: 1fr auto;
		gap: 24px;
		align-items: center;
	}
	.row {
		padding: 14px 22px;
	}
	.row-block {
		padding: 18px 22px;
	}
	.row-title {
		font-size: 14px;
		font-weight: 500;
		margin: 0;
		color: var(--text-primary);
	}
	.row-sub {
		margin-top: 4px;
		line-height: 1.45;
		max-width: 64ch;
	}
	code {
		font-family: var(--font-mono, ui-monospace, monospace);
		background: transparent;
		color: var(--text-primary);
		font-size: 12px;
	}
	.row-extra {
		font-size: 12.5px;
		color: var(--text-secondary);
		margin: 10px 0 0;
		background: var(--fill-hover);
		border: 1px solid var(--border-hairline);
		border-radius: 8px;
		padding: 10px 12px;
		line-height: 1.5;
	}
	.toggle {
		position: relative;
		width: 40px;
		height: 24px;
		border-radius: 999px;
		background: var(--border-secondary);
		flex-shrink: 0;
		cursor: pointer;
		border: 0;
		padding: 0;
	}
	.toggle::after {
		content: '';
		position: absolute;
		left: 2px;
		top: 2px;
		width: 20px;
		height: 20px;
		border-radius: 50%;
		background: var(--switch-thumb);
		box-shadow:
			0 1px 1px rgba(0, 0, 0, 0.06),
			0 2px 6px rgba(0, 0, 0, 0.18);
		transition: left 200ms cubic-bezier(0.32, 0.72, 0, 1);
	}
	.toggle.is-on {
		background: var(--success);
	}
	.toggle.is-on::after {
		left: 18px;
	}
</style>
