<script lang="ts">
	import { extractDomain, type EditComposerState, type Feed } from '../feed-model';
	import { t } from '$lib/i18n';

	interface Props {
		composer: EditComposerState | null;
		feed: Feed | null;
		saving: boolean;
		error: string | null;
		onClose: () => void;
		onSave: () => void;
		onChange: (patch: Partial<EditComposerState>) => void;
	}

	let { composer, feed, saving, error, onClose, onSave, onChange }: Props = $props();
</script>

<section
	class="edit-feed-composer"
	class:open={composer !== null}
	aria-label={$t('feed_management_edit_feed')}
	aria-hidden={composer === null}
>
	{#if composer && feed}
		<div class="composer-inner">
			<div class="composer-header">
				<div>
					<div class="composer-eyebrow">
						{$t('feed_management_edit').toUpperCase()} · {extractDomain(feed.inputUrl)}
					</div>
					<div class="composer-title">
						{$t('feed_management_edit_named', { values: { name: feed.name } })}
					</div>
				</div>
				<button
					type="button"
					class="composer-close"
					onclick={onClose}
					aria-label={$t('common_close')}
				>
					<svg viewBox="0 0 24 24"><path d="M6 6l12 12M18 6L6 18" /></svg>
				</button>
			</div>

			<div class="composer-grid">
				<div class="composer-field span-full">
					<span class="composer-label">{$t('feed_management_source')}</span>
					<div class="composer-source-readonly">
						<svg viewBox="0 0 24 24" aria-hidden="true">
							<path d="M9 17H7a5 5 0 0 1 0-10h2" />
							<path d="M15 7h2a5 5 0 0 1 0 10h-2" />
							<path d="M8 12h8" />
						</svg>
						<span>{feed.inputUrl}</span>
						<span class="source-tag">RSS</span>
					</div>
				</div>

				<div class="composer-field span-full">
					<label class="composer-label" for="edit-feed-title">{$t('common_title')}</label>
					<input
						id="edit-feed-title"
						type="text"
						class="composer-input"
						placeholder={$t('feed_management_title_placeholder')}
						autocomplete="off"
						spellcheck="false"
						value={composer.title}
						oninput={(event) => onChange({ title: event.currentTarget.value })}
					/>
					<span class="composer-hint">{$t('feed_management_title_hint')}</span>
				</div>

				<div class="composer-field">
					<label class="composer-label" for="edit-feed-collection"
						>{$t('feed_management_send_collection')}</label
					>
					<select
						id="edit-feed-collection"
						class="composer-input"
						value={composer.autoSaveCollectionId ?? ''}
						onchange={(event) =>
							onChange({ autoSaveCollectionId: event.currentTarget.value || null })}
					>
						<option value="">{$t('library_triage_inbox')}</option>
					</select>
				</div>

				<div class="composer-field">
					<label class="composer-label" for="edit-feed-schedule"
						>{$t('feed_management_polling_schedule')}</label
					>
					<select
						id="edit-feed-schedule"
						class="composer-input"
						value={composer.pollInterval}
						onchange={(event) => onChange({ pollInterval: event.currentTarget.value })}
					>
						<option value="default">{$t('feed_management_schedule_default')}</option>
						<option value="15"
							>{$t('feed_management_schedule_minutes', { values: { minutes: 15 } })}</option
						>
						<option value="30"
							>{$t('feed_management_schedule_minutes', { values: { minutes: 30 } })}</option
						>
						<option value="60"
							>{$t('feed_management_schedule_hours', { values: { hours: 1 } })}</option
						>
						<option value="240"
							>{$t('feed_management_schedule_hours', { values: { hours: 4 } })}</option
						>
						<option value="1440"
							>{$t('feed_management_schedule_days', { values: { days: 1 } })}</option
						>
					</select>
				</div>

				<div class="composer-field span-full">
					<span class="composer-label">{$t('feed_management_auto_save_new')}</span>
					<div class="auto-save-row">
						<button
							type="button"
							class="toggle"
							class:on={composer.autoSave}
							role="switch"
							aria-checked={composer.autoSave}
							aria-label={composer.autoSave
								? $t('feed_management_disable_auto_save_short')
								: $t('feed_management_enable_auto_save_short')}
							onclick={() => onChange({ autoSave: !composer.autoSave })}
						>
							<span class="toggle-track"></span>
						</button>
						<span class="composer-hint">{$t('feed_management_auto_save_hint')}</span>
					</div>
				</div>
			</div>

			{#if error}
				<div class="composer-error">{error}</div>
			{/if}

			<div class="composer-actions">
				<div class="left">
					<svg viewBox="0 0 24 24" aria-hidden="true">
						<circle cx="12" cy="12" r="10" />
						<path d="M12 8v4M12 16h.01" />
					</svg>
					<span>{$t('feed_management_keyboard_hint')}</span>
				</div>
				<div class="right">
					<button type="button" class="composer-btn ghost" onclick={onClose}
						>{$t('common_cancel')}</button
					>
					<button type="button" class="composer-btn primary" disabled={saving} onclick={onSave}>
						<svg viewBox="0 0 24 24" aria-hidden="true">
							<path d="M5 12l4 4 10-10" />
						</svg>
						<span>{saving ? $t('common_saving') : $t('feed_management_save_changes')}</span>
					</button>
				</div>
			</div>
		</div>
	{/if}
</section>

<style>
	.edit-feed-composer {
		background: var(--bg-elevated);
		border-radius: 16px;
		box-shadow: var(--shadow-1);
		overflow: hidden;
		position: relative;
		max-height: 0;
		margin-bottom: 0;
		opacity: 0;
		transform: translateY(-4px);
		transition:
			max-height 320ms cubic-bezier(0.2, 0, 0, 1),
			opacity 200ms ease,
			margin-bottom 260ms cubic-bezier(0.2, 0, 0, 1),
			transform 220ms ease;
		pointer-events: none;
	}

	.edit-feed-composer.open {
		max-height: 600px;
		margin-bottom: 16px;
		opacity: 1;
		transform: translateY(0);
		pointer-events: auto;
	}

	.edit-feed-composer::before {
		content: '';
		position: absolute;
		left: 0;
		top: 0;
		bottom: 0;
		width: 3px;
		background: linear-gradient(180deg, var(--feed-amber), var(--feed-amber-strong));
	}

	.composer-inner {
		padding: 18px 22px 16px 24px;
	}

	.composer-header,
	.composer-actions,
	.auto-save-row {
		display: flex;
		align-items: center;
	}

	.composer-header {
		align-items: flex-start;
		justify-content: space-between;
		gap: 12px;
		margin-bottom: 14px;
	}

	.composer-eyebrow {
		font-size: 10.5px;
		font-weight: 600;
		letter-spacing: 0.1em;
		text-transform: uppercase;
		color: var(--feed-amber);
		margin-bottom: 4px;
	}

	.composer-title {
		font-size: 16px;
		font-weight: 600;
		letter-spacing: -0.012em;
		color: var(--text-primary);
	}

	.composer-close {
		background: transparent;
		border: 0;
		cursor: pointer;
		width: 28px;
		height: 28px;
		border-radius: 8px;
		color: var(--text-tertiary);
		display: inline-flex;
		align-items: center;
		justify-content: center;
		transition:
			background 140ms,
			color 140ms;
	}

	.composer-close:hover {
		background: var(--fill-hover);
		color: var(--text-primary);
	}

	.composer-close svg,
	.composer-source-readonly svg,
	.composer-actions .left svg,
	.composer-btn svg {
		stroke: currentColor;
		fill: none;
		stroke-linecap: round;
		stroke-linejoin: round;
	}

	.composer-close svg {
		width: 14px;
		height: 14px;
		stroke-width: 1.9;
	}

	.composer-grid {
		display: grid;
		grid-template-columns: 1fr 180px 180px;
		gap: 12px 14px;
		align-items: end;
	}

	.composer-field {
		display: flex;
		flex-direction: column;
		gap: 6px;
		min-width: 0;
	}

	.span-full {
		grid-column: 1 / -1;
	}

	.composer-label {
		font-size: 11.5px;
		font-weight: 600;
		letter-spacing: -0.005em;
		color: var(--text-secondary);
		display: flex;
		align-items: center;
		gap: 6px;
	}

	.composer-input {
		appearance: none;
		-webkit-appearance: none;
		font: inherit;
		font-size: 13.5px;
		letter-spacing: -0.005em;
		color: var(--text-primary);
		background: var(--bg-secondary);
		border: 0;
		border-radius: 10px;
		padding: 10px 12px;
		box-shadow: inset 0 0 0 0.5px var(--border-primary);
		outline: none;
		transition:
			box-shadow 150ms,
			background 150ms;
		width: 100%;
		height: 38px;
		box-sizing: border-box;
	}

	.composer-input:focus {
		box-shadow: inset 0 0 0 1.5px var(--feed-amber);
		background: var(--bg-elevated);
	}

	select.composer-input {
		padding-right: 30px;
		background-image: url("data:image/svg+xml;utf8,<svg xmlns='http://www.w3.org/2000/svg' width='12' height='12' viewBox='0 0 24 24' fill='none' stroke='%236E6E73' stroke-width='1.8' stroke-linecap='round' stroke-linejoin='round'><polyline points='6 9 12 15 18 9'/></svg>");
		background-repeat: no-repeat;
		background-position: right 10px center;
		cursor: pointer;
	}

	.composer-source-readonly {
		display: flex;
		align-items: center;
		gap: 8px;
		background: var(--bg-secondary);
		border-radius: 10px;
		padding: 9px 12px;
		box-shadow: inset 0 0 0 0.5px var(--border-primary);
		font-size: 12.5px;
		color: var(--text-secondary);
		letter-spacing: -0.005em;
		height: 38px;
		box-sizing: border-box;
		min-width: 0;
	}

	.composer-source-readonly svg {
		width: 13px;
		height: 13px;
		stroke: var(--text-tertiary);
		stroke-width: 1.8;
		flex-shrink: 0;
	}

	.composer-source-readonly > span:first-of-type {
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
		font-family: ui-monospace, 'SF Mono', Menlo, monospace;
		font-size: 12px;
	}

	.source-tag {
		margin-left: auto;
		font-family: inherit;
		font-size: 10.5px;
		text-transform: uppercase;
		letter-spacing: 0.06em;
		color: var(--text-tertiary);
		font-weight: 600;
		flex-shrink: 0;
		padding-left: 8px;
	}

	.composer-hint {
		font-size: 11.5px;
		color: var(--text-tertiary);
		letter-spacing: -0.003em;
	}

	.auto-save-row {
		gap: 10px;
		height: 38px;
	}

	.auto-save-row .composer-hint {
		white-space: nowrap;
	}

	.toggle {
		display: inline-flex;
		align-items: center;
		cursor: pointer;
		flex-shrink: 0;
		background: none;
		border: 0;
		padding: 0;
	}

	.toggle-track {
		width: 32px;
		height: 19px;
		border-radius: 980px;
		background: var(--bg-tertiary);
		box-shadow: inset 0 0 0 0.5px var(--border-primary);
		position: relative;
		transition: background 160ms;
		display: block;
	}

	.toggle-track::after {
		content: '';
		position: absolute;
		left: 2px;
		top: 2px;
		width: 15px;
		height: 15px;
		border-radius: 50%;
		background: var(--text-on-color);
		box-shadow: var(--feed-toggle-thumb-shadow);
		transition: left 180ms;
	}

	.toggle.on .toggle-track {
		background: var(--feed-amber);
	}

	.toggle.on .toggle-track::after {
		left: 15px;
	}

	.composer-error {
		font-size: 12px;
		color: var(--destructive);
		margin-top: 8px;
		letter-spacing: -0.003em;
	}

	.composer-actions {
		justify-content: space-between;
		gap: 8px;
		margin-top: 16px;
		padding-top: 14px;
		border-top: 0.5px solid var(--border-primary);
	}

	.left {
		font-size: 11.5px;
		color: var(--text-tertiary);
		letter-spacing: -0.003em;
		display: flex;
		align-items: center;
		gap: 6px;
	}

	.left svg {
		width: 12px;
		height: 12px;
		stroke-width: 1.8;
	}

	.right {
		display: flex;
		gap: 8px;
	}

	.composer-btn {
		appearance: none;
		border: 0;
		font: inherit;
		font-size: 13px;
		font-weight: 600;
		letter-spacing: -0.005em;
		padding: 8px 14px;
		border-radius: 980px;
		cursor: pointer;
		display: inline-flex;
		align-items: center;
		gap: 6px;
		transition:
			transform 120ms,
			box-shadow 140ms,
			background 140ms;
	}

	.composer-btn svg {
		width: 12px;
		height: 12px;
		stroke-width: 2;
	}

	.composer-btn.ghost {
		background: transparent;
		color: var(--text-secondary);
	}

	.composer-btn.ghost:hover {
		background: var(--fill-hover);
		color: var(--text-primary);
	}

	.composer-btn.primary {
		background: linear-gradient(180deg, var(--feed-amber), var(--feed-amber-strong));
		color: var(--text-on-color);
		box-shadow: var(--feed-amber-button-shadow);
	}

	.composer-btn.primary:hover:not(:disabled) {
		transform: translateY(-1px);
		box-shadow: var(--feed-amber-button-shadow-hover);
	}

	.composer-btn.primary:disabled {
		opacity: 0.85;
		cursor: progress;
		transform: none;
	}

	@media (max-width: 720px) {
		.composer-grid {
			grid-template-columns: 1fr;
		}
	}
</style>
