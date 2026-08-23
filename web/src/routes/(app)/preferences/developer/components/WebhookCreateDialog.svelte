<script lang="ts">
	import { groupCount, isGroupAllSelected, type WebhookEventGroup } from '../developer-model';
	import { t, type MessageKey } from '$lib/i18n';

	interface Props {
		open: boolean;
		name: string;
		url: string;
		events: Set<string>;
		active: boolean;
		creating: boolean;
		error: string | null;
		eventGroups: WebhookEventGroup[];
		onClose: () => void;
		onName: (name: string) => void;
		onUrl: (url: string) => void;
		onToggleEvent: (event: string) => void;
		onToggleGroup: (events: string[]) => void;
		onActive: (active: boolean) => void;
		onSubmit: () => void;
	}

	let {
		open,
		name,
		url,
		events,
		active,
		creating,
		error,
		eventGroups,
		onClose,
		onName,
		onUrl,
		onToggleEvent,
		onToggleGroup,
		onActive,
		onSubmit
	}: Props = $props();

	const EVENT_GROUP_LABEL_KEYS: Record<string, MessageKey> = {
		library_entry: 'prefs_developer_event_group_library_entry',
		highlight: 'prefs_developer_event_group_highlight',
		feed: 'prefs_developer_event_group_feed',
		taxonomy: 'prefs_developer_event_group_taxonomy',
		lifecycle: 'prefs_developer_event_group_lifecycle'
	};

	function eventGroupLabel(group: WebhookEventGroup): string {
		const key = EVENT_GROUP_LABEL_KEYS[group.key];
		return key ? $t(key) : group.name;
	}
</script>

<div class="add-form" class:open inert={!open} aria-hidden={!open}>
	<div class="add-form-inner">
		<div class="form-head">
			<div class="form-title">{$t('prefs_developer_add_webhook')}</div>
			<button type="button" class="close" onclick={onClose} aria-label={$t('common_close')}>
				<svg viewBox="0 0 24 24" aria-hidden="true"><path d="M6 6l12 12M18 6l-12 12" /></svg>
			</button>
		</div>

		<div class="form-row">
			<label class="lab" for="dev-add-name">
				{$t('prefs_developer_name')}<span class="help">{$t('prefs_developer_internal_label')}</span>
			</label>
			<input
				id="dev-add-name"
				class="input"
				type="text"
				placeholder={$t('prefs_developer_webhook_name_placeholder')}
				value={name}
				oninput={(event) => onName(event.currentTarget.value)}
			/>
		</div>

		<div class="form-row">
			<label class="lab" for="dev-add-url">
				URL<span class="help">{$t('prefs_developer_https_hint')}</span>
			</label>
			<input
				id="dev-add-url"
				class="input mono"
				type="text"
				placeholder="https://example.com/hooks/indelible"
				value={url}
				oninput={(event) => onUrl(event.currentTarget.value)}
			/>
		</div>

		<div class="form-row">
			<div class="lab">
				{$t('prefs_developer_events')}<span class="help"
					>{$t('prefs_developer_events_signing_hint')}</span
				>
			</div>
			<div class="event-picker">
				{#each eventGroups as group (group.key)}
					{@const allOn = isGroupAllSelected(group.events, events)}
					<div class="event-group">
						<div class="group-head">
							<span class="group-name">{eventGroupLabel(group)}</span>
							<button type="button" class="all-toggle" onclick={() => onToggleGroup(group.events)}>
								{$t(allOn ? 'common_clear' : 'prefs_developer_select_all')}
							</button>
							<span class="group-count">
								{groupCount(group.events, events)} / {group.events.length}
							</span>
						</div>
						<div class="group-body">
							{#each group.events as event (event)}
								{@const checked = events.has(event)}
								<button
									type="button"
									class="event-check"
									class:checked
									onclick={() => onToggleEvent(event)}
								>
									<span class="box">
										<svg viewBox="0 0 24 24" aria-hidden="true">
											<polyline points="20 6 9 17 4 12" />
										</svg>
									</span>
									<span class="event-name">{event}</span>
								</button>
							{/each}
						</div>
					</div>
				{/each}
			</div>
		</div>

		<div class="form-row">
			<div class="lab">
				{$t('prefs_developer_active')}<span class="help">{$t('prefs_developer_active_hint')}</span>
			</div>
			<button
				type="button"
				class="toggle"
				class:on={active}
				aria-pressed={active}
				aria-label={$t('prefs_developer_toggle_active')}
				onclick={() => onActive(!active)}
			></button>
		</div>

		{#if error}
			<div class="form-error" role="alert">{error}</div>
		{/if}

		<div class="form-foot">
			<button type="button" class="btn ghost" onclick={onClose}>{$t('common_cancel')}</button>
			<button type="button" class="btn primary" disabled={creating} onclick={onSubmit}>
				{creating ? $t('prefs_developer_creating') : $t('prefs_developer_create_endpoint')}
			</button>
		</div>
	</div>
</div>

<style>
	.add-form {
		background: var(--dev-card-strong);
		border-radius: 14px;
		box-shadow: inset 0 0 0 0.5px var(--border-primary);
		margin-top: 12px;
		overflow: hidden;
		max-height: 0;
		opacity: 0;
		transition:
			max-height 320ms ease,
			opacity 240ms ease;
	}

	.add-form.open {
		max-height: 1600px;
		opacity: 1;
	}

	.add-form-inner {
		padding: 22px 24px 24px;
		display: flex;
		flex-direction: column;
		gap: 22px;
	}

	.form-head,
	.form-foot,
	.group-head,
	.event-check {
		display: flex;
		align-items: center;
	}

	.form-head {
		justify-content: space-between;
	}

	.form-title {
		font-size: 14px;
		font-weight: 600;
		color: var(--text-primary);
		letter-spacing: -0.015em;
	}

	.close {
		width: 24px;
		height: 24px;
		border-radius: 6px;
		color: var(--text-tertiary);
		background: none;
		border: none;
		cursor: pointer;
		display: inline-flex;
		align-items: center;
		justify-content: center;
	}

	.close:hover {
		background: var(--fill-hover);
		color: var(--text-primary);
	}

	.close svg {
		width: 14px;
		height: 14px;
		stroke: currentColor;
		fill: none;
		stroke-width: 1.6;
		stroke-linecap: round;
		stroke-linejoin: round;
	}

	.form-row {
		display: grid;
		grid-template-columns: 140px minmax(0, 1fr);
		gap: 18px;
		align-items: flex-start;
	}

	.lab {
		font-size: 12.5px;
		font-weight: 500;
		color: var(--text-primary);
		padding-top: 10px;
	}

	.help {
		display: block;
		font-size: 11.5px;
		color: var(--text-tertiary);
		margin-top: 2px;
		font-weight: 400;
		letter-spacing: -0.005em;
	}

	.input {
		background: var(--bg-elevated);
		color: var(--text-primary);
		border: none;
		outline: none;
		border-radius: 8px;
		padding: 8px 12px;
		font-size: 13.5px;
		letter-spacing: -0.01em;
		box-shadow:
			inset 0 0 0 0.5px var(--border-primary),
			0 1px 0 rgba(0, 0, 0, 0.02);
		width: 100%;
		font-family: inherit;
	}

	.input:focus {
		box-shadow:
			inset 0 0 0 0.5px var(--border-primary),
			0 0 0 3px var(--dev-accent-soft);
	}

	.mono,
	.event-name,
	.group-count {
		font-family: 'SF Mono', 'Fira Code', Menlo, ui-monospace, monospace;
	}

	.event-picker {
		display: flex;
		flex-direction: column;
		gap: 14px;
	}

	.event-group {
		border-radius: 10px;
		background: var(--bg-elevated);
		box-shadow: inset 0 0 0 0.5px var(--border-primary);
		overflow: hidden;
	}

	.group-head {
		padding: 9px 14px;
		background: var(--bg-secondary);
		border-bottom: 0.5px solid var(--border-hairline);
		gap: 10px;
	}

	.group-name {
		font-size: 12px;
		font-weight: 600;
		color: var(--text-primary);
		letter-spacing: -0.01em;
	}

	.group-count {
		font-size: 11px;
		color: var(--text-tertiary);
		margin-left: auto;
	}

	.all-toggle {
		font-size: 11px;
		color: var(--dev-accent);
		font-weight: 500;
		background: none;
		border: none;
		cursor: pointer;
		padding: 0;
		font-family: inherit;
	}

	.group-body {
		display: grid;
		grid-template-columns: repeat(2, minmax(0, 1fr));
	}

	.event-check {
		gap: 10px;
		padding: 8px 14px;
		cursor: pointer;
		border-top: 0.5px solid var(--border-hairline);
		border-left: none;
		border-right: none;
		border-bottom: none;
		background: none;
		text-align: left;
		font: inherit;
		color: inherit;
		width: 100%;
	}

	.event-check:hover {
		background: var(--fill-hover);
	}

	.event-check:nth-child(-n + 2) {
		border-top: none;
	}

	.box {
		width: 14px;
		height: 14px;
		border-radius: 4px;
		box-shadow: inset 0 0 0 1px var(--border-primary);
		background: var(--bg-elevated);
		color: transparent;
		display: flex;
		align-items: center;
		justify-content: center;
		flex-shrink: 0;
		font-size: 10px;
	}

	.box svg {
		width: 9px;
		height: 9px;
		stroke: currentColor;
		fill: none;
		stroke-width: 2.5;
		stroke-linecap: round;
		stroke-linejoin: round;
	}

	.event-check.checked .box {
		background: var(--dev-accent);
		box-shadow: inset 0 0 0 1px var(--dev-accent);
		color: var(--text-on-color);
	}

	.event-name {
		font-size: 11.5px;
		color: var(--text-primary);
		letter-spacing: -0.01em;
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}

	.form-error {
		font-size: 12.5px;
		color: var(--destructive);
		padding: 8px 12px;
		border-radius: 8px;
		background: var(--dev-destructive-soft);
	}

	.form-foot {
		justify-content: flex-end;
		gap: 8px;
		padding: 16px 0 0;
		border-top: 0.5px solid var(--border-hairline);
		margin-top: 4px;
	}

	.btn {
		border: none;
		border-radius: 8px;
		padding: 7px 14px;
		font: inherit;
		font-size: 12.5px;
		font-weight: 500;
		cursor: pointer;
		letter-spacing: 0;
	}

	.btn.ghost {
		background: transparent;
		color: var(--text-primary);
		box-shadow: inset 0 0 0 0.5px var(--border-primary);
	}

	.btn.primary {
		background: var(--dev-accent);
		color: var(--text-on-color);
	}

	.btn[disabled] {
		opacity: 0.45;
		cursor: default;
	}

	.toggle {
		width: 40px;
		height: 24px;
		border-radius: 12px;
		background: var(--bg-tertiary);
		position: relative;
		cursor: pointer;
		flex-shrink: 0;
		transition: background 200ms ease;
		border: none;
		padding: 0;
	}

	.toggle::after {
		content: '';
		position: absolute;
		top: 3px;
		left: 3px;
		width: 18px;
		height: 18px;
		border-radius: 50%;
		background: var(--bg-elevated);
		box-shadow: var(--shadow-1);
		transition: left 200ms ease;
	}

	.toggle.on {
		background: var(--dev-accent);
	}

	.toggle.on::after {
		left: 19px;
	}

	@media (max-width: 720px) {
		.form-row,
		.group-body {
			grid-template-columns: 1fr;
		}

		.event-check:nth-child(2) {
			border-top: 0.5px solid var(--border-hairline);
		}
	}
</style>
