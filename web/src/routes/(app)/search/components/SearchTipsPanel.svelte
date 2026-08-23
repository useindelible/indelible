<script lang="ts">
	import { t } from '$lib/i18n';

	interface Props {
		hints: string[];
		tipsVisible: boolean;
		onTipsVisibleChange: (visible: boolean) => void;
	}

	let { hints, tipsVisible, onTipsVisibleChange }: Props = $props();
</script>

<div class="search-hints">
	{$t('search_syntax')}
	{#each hints as hint (hint)}
		<code>{hint}</code>
	{/each}
	<code>!tag:</code>
	<div
		class="tips-trigger"
		role="button"
		tabindex="0"
		aria-label={$t('search_tips')}
		onmouseenter={() => onTipsVisibleChange(true)}
		onmouseleave={() => onTipsVisibleChange(false)}
		onfocus={() => onTipsVisibleChange(true)}
		onblur={() => onTipsVisibleChange(false)}
	>
		<svg viewBox="0 0 24 24" aria-hidden="true"
			><circle cx="12" cy="12" r="10" /><path d="M12 16v-4" /><path d="M12 8h.01" /></svg
		>
		{#if tipsVisible}
			<div class="tips-popover" role="tooltip">
				<span class="tips-popover-title">{$t('search_tips')}</span>
				<ul class="tips-popover-list">
					<li><code>tag:research</code> {$t('search_tip_tag')}</li>
					<li><code>type:pdf</code> {$t('search_tip_content_type')}</li>
					<li><code>sender:news@example.com</code> {$t('search_tip_sender')}</li>
					<li><code>sender_domain:example.com</code> {$t('search_tip_sender_domain')}</li>
					<li><code>list:weekly.example.com</code> {$t('search_tip_list_id')}</li>
					<li><code>subject:"weekly brief"</code> {$t('search_tip_subject')}</li>
					<li><code>collection:reading-list</code> {$t('search_tip_collection')}</li>
					<li><code>before:2026-01-01</code> {$t('search_tip_date_range')}</li>
					<li><code>is:read</code> / <code>is:unread</code> {$t('search_tip_reading_status')}</li>
					<li>
						<code>has:highlights</code> / <code>has:notes</code> /
						<code>has:unsubscribe</code>
						{$t('search_tip_presence')}
					</li>
					<li><code>is:blocked</code> {$t('search_tip_blocked_senders')}</li>
					<li><code>!tag:read</code> {$t('search_tip_negation')}</li>
				</ul>
			</div>
		{/if}
	</div>
</div>
