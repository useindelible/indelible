<script lang="ts">
	import FeedRow from './FeedRow.svelte';
	import type { Feed } from '../feed-model';

	interface Props {
		feeds: Feed[];
		openKebabId: string | null;
		onToggleAutoSave: (id: string) => void;
		onToggleFeed: (id: string) => void;
		onToggleMenu: (id: string, event: MouseEvent) => void;
		onCloseMenu: () => void;
		onEdit: (id: string) => void;
		onRetry: (id: string) => void;
		onDelete: (id: string) => void;
	}

	let {
		feeds,
		openKebabId,
		onToggleAutoSave,
		onToggleFeed,
		onToggleMenu,
		onCloseMenu,
		onEdit,
		onRetry,
		onDelete
	}: Props = $props();
</script>

<div class="feeds-table-wrap">
	<table class="feeds">
		<thead>
			<tr>
				<th class="feed-col">Feed</th>
				<th>Status</th>
				<th>Auto-save</th>
				<th>Schedule</th>
				<th>Last updated</th>
				<th class="right action-col"></th>
			</tr>
		</thead>
		<tbody>
			{#each feeds as feed (feed.id)}
				<FeedRow
					{feed}
					menuOpen={openKebabId === feed.id}
					{onToggleAutoSave}
					{onToggleFeed}
					{onToggleMenu}
					{onCloseMenu}
					{onEdit}
					{onRetry}
					{onDelete}
				/>
			{/each}
		</tbody>
	</table>
</div>

<style>
	.feeds-table-wrap {
		background: var(--bg-elevated);
		border-radius: 14px;
		box-shadow: var(--feed-card-shadow);
		/* Stays visible so the kebab menu can escape the card; narrow widths
		   fold columns instead of scrolling for the same reason. */
		overflow: visible;
		container-type: inline-size;
		container-name: feeds-card;
	}

	table.feeds {
		width: 100%;
		border-collapse: collapse;
		font-size: 13px;
		font-family: var(--font-sans);
	}

	th {
		font-size: 11px;
		font-weight: 600;
		letter-spacing: 0.06em;
		text-transform: uppercase;
		color: var(--text-tertiary);
		background: var(--feed-table-head-bg);
		text-align: left;
		padding: 11px 14px;
		border-bottom: 0.5px solid var(--border-primary);
		white-space: nowrap;
	}

	th.right {
		text-align: right;
	}

	th:first-child {
		border-top-left-radius: 14px;
	}

	th:last-child {
		border-top-right-radius: 14px;
	}

	.feed-col {
		width: 32%;
	}

	.action-col {
		width: 40px;
	}

	/* Fold columns as the card narrows: Schedule and Last updated go first,
	   Auto-save follows on phones. Feed, Status, and actions always stay. */
	@container feeds-card (max-width: 739px) {
		th {
			padding: 10px 8px;
		}

		th:first-child {
			padding-left: 14px;
		}

		th:last-child {
			padding-right: 14px;
		}

		th:nth-child(4),
		th:nth-child(5) {
			display: none;
		}
	}

	@container feeds-card (max-width: 439px) {
		th:nth-child(3) {
			display: none;
		}
	}
</style>
