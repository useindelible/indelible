<script lang="ts">
	import type { RecentSearchResponse } from '$lib/api/generated/types.gen';
	import { t } from '$lib/i18n';

	interface Props {
		recentLoading: boolean;
		recentSearches: RecentSearchResponse[];
		onRecentClick: (query: string) => void;
		onClearAll: () => void | Promise<void>;
		onDeleteRecent: (id: string) => void | Promise<void>;
	}

	let { recentLoading, recentSearches, onRecentClick, onClearAll, onDeleteRecent }: Props =
		$props();
</script>

<div class="empty-state">
	{#if recentLoading}
		<div class="empty-loading">
			<div class="skeleton-line wide"></div>
			<div class="skeleton-line"></div>
			<div class="skeleton-line"></div>
		</div>
	{:else if recentSearches.length > 0}
		<div class="recent-section">
			<div class="recent-header">
				<span class="recent-title">{$t('search_recent_title')}</span>
				<button type="button" class="recent-clear-btn" onclick={onClearAll}
					>{$t('search_recent_clear_all')}</button
				>
			</div>
			<ul class="recent-list" role="list">
				{#each recentSearches as recent (recent.id)}
					<li class="recent-item">
						<button
							type="button"
							class="recent-query-btn"
							onclick={() => onRecentClick(recent.query)}
						>
							<svg class="recent-icon" viewBox="0 0 24 24" aria-hidden="true">
								<circle cx="12" cy="12" r="10" />
								<polyline points="12 6 12 12 16 14" />
							</svg>
							<span class="recent-query-text">{recent.query}</span>
						</button>
						<button
							type="button"
							class="recent-delete-btn"
							onclick={() => onDeleteRecent(recent.id)}
							aria-label={$t('search_recent_remove')}
						>
							<svg viewBox="0 0 24 24"
								><line x1="6" y1="6" x2="18" y2="18" /><line x1="18" y1="6" x2="6" y2="18" /></svg
							>
						</button>
					</li>
				{/each}
			</ul>
		</div>
	{:else}
		<div class="empty-prompt">
			<div class="empty-icon" aria-hidden="true">
				<svg viewBox="0 0 24 24"
					><circle cx="11" cy="11" r="8" /><line x1="21" y1="21" x2="16.65" y2="16.65" /></svg
				>
			</div>
			<p class="empty-title">{$t('search_recent_empty_title')}</p>
			<p class="empty-subtitle">{$t('search_recent_empty_body')}</p>
		</div>
	{/if}
</div>
