<script lang="ts">
	interface Props {
		hints: string[];
		tipsVisible: boolean;
		onTipsVisibleChange: (visible: boolean) => void;
	}

	let { hints, tipsVisible, onTipsVisibleChange }: Props = $props();
</script>

<div class="search-hints">
	Syntax:
	{#each hints as hint (hint)}
		<code>{hint}</code>
	{/each}
	<code>!tag:</code>
	<div
		class="tips-trigger"
		role="button"
		tabindex="0"
		aria-label="Search tips"
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
				<span class="tips-popover-title">Search Tips</span>
				<ul class="tips-popover-list">
					<li><code>tag:research</code> Filter by tag</li>
					<li><code>type:pdf</code> Filter by content type</li>
					<li><code>sender:news@example.com</code> Filter by sender</li>
					<li><code>sender_domain:example.com</code> Filter by sender domain</li>
					<li><code>list:weekly.example.com</code> Filter by List-ID</li>
					<li><code>subject:"weekly brief"</code> Filter by email subject</li>
					<li><code>collection:reading-list</code> Filter by collection</li>
					<li><code>before:2026-01-01</code> Date range</li>
					<li><code>is:read</code> / <code>is:unread</code> Reading status</li>
					<li>
						<code>has:highlights</code> / <code>has:notes</code> /
						<code>has:unsubscribe</code> Presence filters
					</li>
					<li><code>is:blocked</code> Filter blocked email senders</li>
					<li><code>!tag:read</code> Negation with <code>!</code> or <code>-</code></li>
				</ul>
			</div>
		{/if}
	</div>
</div>
