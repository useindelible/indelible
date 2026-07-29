<script lang="ts">
	import type { ObsidianSettingsDto } from '$lib/api';

	type TemplateKey =
		| 'properties_template'
		| 'page_title_template'
		| 'metadata_template'
		| 'highlight_header_template'
		| 'highlight_template'
		| 'sync_notification_template';

	interface TemplateEditor {
		key: TemplateKey;
		name: string;
		cap: string;
		placeholder?: string;
		span?: boolean;
		short?: boolean;
	}

	interface Props {
		settings: ObsidianSettingsDto;
		varsOpen: boolean;
		saveError: string | null;
		onToggleVars: () => void;
		onTemplateChange: (key: TemplateKey, value: string) => void;
	}

	let { settings, varsOpen, saveError, onToggleVars, onTemplateChange }: Props = $props();

	const variableGroups = [
		{
			label: 'Document',
			vars: [
				'{{title}}',
				'{{full_title}}',
				'{{author}}',
				'{{url}}',
				'{{category}}',
				'{{image_url}}',
				'{{summary}}',
				'{{document_tags}}'
			]
		},
		{
			label: 'Run context',
			vars: [
				'{{date}}',
				'{{time}}',
				'{{is_new_page}}',
				'{{has_new_highlights}}',
				'{{document_count}}'
			]
		},
		{
			label: 'Highlight',
			vars: [
				'{{highlight_text}}',
				'{{highlight_location}}',
				'{{highlight_location_url}}',
				'{{highlight_tags}}',
				'{{highlight_note}}',
				'{{color}}',
				'{{created_at}}'
			]
		}
	];

	const editors: TemplateEditor[] = [
		{
			key: 'properties_template',
			name: 'Properties / YAML',
			cap: 'optional · frontmatter',
			placeholder: '---\n# YAML frontmatter (optional)\n---'
		},
		{
			key: 'page_title_template',
			name: 'Page title',
			cap: 'optional first line',
			placeholder: '(empty — Obsidian shows the file title)'
		},
		{ key: 'metadata_template', name: 'Metadata', cap: 'metadata block', span: true },
		{
			key: 'highlight_header_template',
			name: 'Highlight header',
			cap: 'switches on first vs. incremental sync'
		},
		{ key: 'highlight_template', name: 'Highlight', cap: 'per highlight · location · tags · note' },
		{
			key: 'sync_notification_template',
			name: 'Sync notification',
			cap: 'appended to Indelible/Indelible Syncs.md',
			span: true,
			short: true
		}
	];

	function valueFor(key: TemplateKey): string {
		return settings[key] ?? '';
	}
</script>

<section class="section">
	<div class="section-head">
		<h2 class="section-title">Templates</h2>
		<p class="section-sub">MiniJinja syntax · server-rendered on every sync</p>
	</div>

	<div class="vars" class:is-open={varsOpen}>
		<button type="button" class="vars-head" aria-expanded={varsOpen} onclick={onToggleVars}>
			<span class="chev" aria-hidden="true">›</span>
			Available variables
			<span class="vars-tag">21 total</span>
		</button>
		<div class="vars-body">
			<div>
				<div class="vars-body-inner">
					{#each variableGroups as group (group.label)}
						<div class="var-group">
							<div class="var-group-label">{group.label}</div>
							<div class="var-pills">
								{#each group.vars as variable (variable)}
									<span class="var-pill">{variable}</span>
								{/each}
							</div>
						</div>
					{/each}
				</div>
			</div>
		</div>
	</div>

	<div class="editors-grid">
		{#each editors as editor (editor.key)}
			<div class="editor" class:span-2={editor.span}>
				<div class="editor-head">
					<span class="editor-name">{editor.name}</span>
					<span class="editor-cap">{editor.cap}</span>
				</div>
				<div class="editor-body">
					<textarea
						class="editor-textarea"
						class:editor-textarea-short={editor.short}
						spellcheck="false"
						placeholder={editor.placeholder}
						value={valueFor(editor.key)}
						oninput={(event) => onTemplateChange(editor.key, event.currentTarget.value)}
					></textarea>
				</div>
			</div>
		{/each}
	</div>

	{#if saveError}
		<div class="alert-block">
			<div class="alert">
				<strong>Save failed</strong>
				<p>{saveError}</p>
			</div>
		</div>
	{/if}
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
	.section-sub {
		font-size: 12.5px;
		color: var(--text-tertiary);
		margin: 0;
	}
	.vars,
	.editor {
		background: var(--bg-elevated);
		border: 1px solid var(--border-hairline);
		box-shadow: var(--shadow-1);
		overflow: hidden;
	}
	.vars {
		border-radius: 14px;
		margin-bottom: 14px;
	}
	.vars-head {
		width: 100%;
		display: flex;
		align-items: center;
		gap: 10px;
		padding: 13px 18px;
		cursor: pointer;
		font-size: 13.5px;
		font-weight: 500;
		color: var(--text-primary);
		background: transparent;
		border: 0;
		text-align: left;
	}
	.chev {
		color: var(--text-tertiary);
		transition: transform 200ms ease;
		flex-shrink: 0;
	}
	.vars.is-open .chev {
		transform: rotate(90deg);
	}
	.vars-tag {
		margin-left: auto;
		font-size: 12px;
		color: var(--text-tertiary);
		font-weight: 400;
	}
	.vars-body {
		display: grid;
		grid-template-rows: 0fr;
		transition: grid-template-rows 280ms cubic-bezier(0.4, 0, 0.2, 1);
	}
	.vars.is-open .vars-body {
		grid-template-rows: 1fr;
	}
	.vars-body > div {
		overflow: hidden;
	}
	.vars-body-inner {
		padding: 14px 18px 18px;
		border-top: 1px solid var(--border-hairline);
	}
	.var-group + .var-group {
		margin-top: 14px;
	}
	.var-group-label {
		font-size: 11px;
		color: var(--text-tertiary);
		text-transform: uppercase;
		letter-spacing: 0;
		font-weight: 500;
		margin-bottom: 6px;
	}
	.var-pills {
		display: flex;
		flex-wrap: wrap;
		gap: 5px;
	}
	.var-pill {
		font-family: var(--font-mono, ui-monospace, monospace);
		font-size: 11.5px;
		padding: 3px 8px;
		border-radius: 6px;
		background: var(--fill-hover);
		border: 1px solid var(--border-hairline);
		color: var(--text-secondary);
	}
	.editors-grid {
		display: grid;
		grid-template-columns: repeat(2, minmax(0, 1fr));
		gap: 12px;
	}
	.editor {
		border-radius: 12px;
		display: flex;
		flex-direction: column;
	}
	.editor.span-2 {
		grid-column: span 2;
	}
	.editor-head {
		padding: 10px 14px 8px;
		display: flex;
		align-items: baseline;
		justify-content: space-between;
		gap: 10px;
		border-bottom: 1px solid var(--border-hairline);
		background: var(--bg-secondary);
	}
	.editor-name {
		font-size: 12.5px;
		font-weight: 550;
		letter-spacing: 0;
		color: var(--text-primary);
	}
	.editor-cap {
		font-size: 11.5px;
		color: var(--text-tertiary);
	}
	.editor-body {
		position: relative;
		flex: 1;
	}
	.editor-textarea {
		width: 100%;
		border: 0;
		background: var(--obs-editor-bg);
		color: var(--text-primary);
		padding: 12px 14px;
		font-family: var(--font-mono, ui-monospace, monospace);
		font-size: 12px;
		line-height: 1.6;
		resize: vertical;
		min-height: 120px;
		display: block;
		outline: 0;
		white-space: pre;
		overflow-x: auto;
	}
	.editor-textarea-short {
		min-height: 64px;
	}
	.alert-block {
		padding: 16px 0 0;
	}
	.alert {
		padding: 14px;
		border-radius: 12px;
		background: var(--obs-alert-bg);
		border: 1px solid var(--obs-alert-border);
		color: var(--text-secondary);
	}
	.alert strong {
		color: var(--text-primary);
	}
	.alert p {
		margin: 4px 0 0;
	}
	@media (max-width: 980px) {
		.editors-grid {
			grid-template-columns: 1fr;
		}
		.editor.span-2 {
			grid-column: auto;
		}
	}
</style>
