<script lang="ts">
	import { onMount } from 'svelte';
	import { getModalStore } from '$lib/stores/addItemModal.svelte';
	import SavePill from '$lib/components/settings/SavePill.svelte';
	import { listSubscriptions, retrySubscription, unsubscribe, updateSubscription } from '$lib/api';
	import type { OpmlImportResponse } from '$lib/api';
	import { uploadOpml } from '$lib/api/feeds';
	import FeedDeleteDialog from './components/FeedDeleteDialog.svelte';
	import FeedEditComposer from './components/FeedEditComposer.svelte';
	import FeedHero from './components/FeedHero.svelte';
	import FeedSubscriptionsPanel from './components/FeedSubscriptionsPanel.svelte';
	import OpmlImportCard from './components/OpmlImportCard.svelte';
	import {
		calculateFeedStats,
		changedSnapshotFeeds,
		filterFeeds,
		mapSubscription,
		parseFeedSnapshot,
		pollIntervalToMinutes,
		snapshotFeeds,
		type EditComposerState,
		type Feed,
		type FilterChip
	} from './feed-model';

	const modal = getModalStore();

	let editComposer = $state<EditComposerState | null>(null);
	let editSaving = $state(false);
	let editError = $state<string | null>(null);
	let deleteConfirmId = $state<string | null>(null);
	let openKebabId = $state<string | null>(null);

	let feeds = $state<Feed[]>([]);
	let loading = $state(true);
	let loadError = $state<string | null>(null);
	let activeFilter = $state<FilterChip>('all');
	let searchQuery = $state('');
	let savedSnapshot = $state('[]');
	let saving = $state(false);
	let showSaved = $state(false);

	let opmlUploading = $state(false);
	let opmlResult = $state<OpmlImportResponse | null>(null);
	let opmlError = $state<string | null>(null);

	const stats = $derived(calculateFeedStats(feeds));
	const filteredFeeds = $derived(filterFeeds(feeds, activeFilter, searchQuery));
	const editingFeed = $derived(
		editComposer ? (feeds.find((feed) => feed.id === editComposer?.feedId) ?? null) : null
	);
	const feedToDelete = $derived(
		deleteConfirmId ? (feeds.find((feed) => feed.id === deleteConfirmId) ?? null) : null
	);
	const isDirty = $derived(snapshotFeeds(feeds) !== savedSnapshot);

	async function loadFeeds() {
		loading = true;
		loadError = null;
		try {
			const { data } = await listSubscriptions();
			if (data) {
				feeds = (data.data ?? []).map((subscription) => mapSubscription(subscription));
				savedSnapshot = snapshotFeeds(feeds);
			} else {
				loadError = 'Failed to load subscriptions';
			}
		} catch {
			loadError = 'An unexpected error occurred';
		} finally {
			loading = false;
		}
	}

	onMount(() => {
		loadFeeds();
	});

	$effect(() => {
		if (modal.subscribedCount > 0) loadFeeds();
	});

	$effect(() => {
		if (editComposer === null) return;
		function onKey(event: KeyboardEvent) {
			if (event.key === 'Escape') {
				event.preventDefault();
				closeEditComposer();
			} else if (event.key === 'Enter' && (event.metaKey || event.ctrlKey) && !editSaving) {
				event.preventDefault();
				saveEditComposer();
			}
		}
		window.addEventListener('keydown', onKey);
		return () => window.removeEventListener('keydown', onKey);
	});

	$effect(() => {
		if (openKebabId === null) return;
		const clickHandler = () => closeKebab();
		const keyHandler = (event: KeyboardEvent) => {
			if (event.key === 'Escape') closeKebab();
		};
		document.addEventListener('click', clickHandler);
		document.addEventListener('keydown', keyHandler);
		return () => {
			document.removeEventListener('click', clickHandler);
			document.removeEventListener('keydown', keyHandler);
		};
	});

	function discardChanges() {
		const saved = parseFeedSnapshot(savedSnapshot);
		for (const entry of saved) {
			const feed = feeds.find((candidate) => candidate.id === entry.id);
			if (feed) {
				feed.autoSave = entry.autoSave;
				feed.autoSaveCollectionId = entry.autoSaveCollectionId;
			}
		}
	}

	async function saveChanges() {
		saving = true;
		const saved = parseFeedSnapshot(savedSnapshot);
		const changed = changedSnapshotFeeds(feeds, saved);

		try {
			const results = await Promise.all(
				changed.map((feed) => {
					const previous = saved.find((entry) => entry.id === feed.id)!;
					return updateSubscription({
						path: { id: feed.id },
						body: {
							...(previous.autoSave !== feed.autoSave ? { auto_save: feed.autoSave } : {}),
							...(previous.autoSaveCollectionId !== feed.autoSaveCollectionId
								? { auto_save_collection_id: feed.autoSaveCollectionId }
								: {})
						}
					});
				})
			);
			if (results.some((result) => result.error)) {
				loadError = 'Failed to save some changes';
			} else {
				savedSnapshot = snapshotFeeds(feeds);
				showSaved = true;
				setTimeout(() => {
					showSaved = false;
				}, 2000);
			}
		} catch {
			loadError = 'Failed to save changes';
		} finally {
			saving = false;
		}
	}

	async function toggleFeed(id: string) {
		const feed = feeds.find((candidate) => candidate.id === id);
		if (!feed) return;
		const newStatus = feed.enabled ? 'paused' : 'active';
		feed.enabled = !feed.enabled;
		feed.status = newStatus === 'paused' ? 'paused' : 'active';
		try {
			const { error } = await updateSubscription({ path: { id }, body: { status: newStatus } });
			if (error) restoreFeedToggle(feed);
		} catch {
			restoreFeedToggle(feed);
		}
	}

	function restoreFeedToggle(feed: Feed) {
		feed.enabled = !feed.enabled;
		feed.status = feed.enabled ? 'active' : 'paused';
	}

	function toggleAutoSave(id: string) {
		const feed = feeds.find((candidate) => candidate.id === id);
		if (feed) feed.autoSave = !feed.autoSave;
	}

	function requestDelete(id: string) {
		deleteConfirmId = id;
	}

	async function confirmDelete() {
		const id = deleteConfirmId;
		if (!id) return;
		deleteConfirmId = null;

		const removed = feeds.find((feed) => feed.id === id);
		if (!removed) return;
		feeds = feeds.filter((feed) => feed.id !== id);
		savedSnapshot = snapshotFeeds(feeds);
		try {
			const { error } = await unsubscribe({ path: { id } });
			if (error) restoreDeletedFeed(removed);
		} catch {
			restoreDeletedFeed(removed);
		}
	}

	function restoreDeletedFeed(feed: Feed) {
		feeds = [...feeds, feed];
		savedSnapshot = snapshotFeeds(feeds);
	}

	function openEditComposer(feedId: string) {
		const feed = feeds.find((candidate) => candidate.id === feedId);
		if (!feed) return;
		editError = null;
		editComposer = {
			feedId,
			title: feed.name,
			autoSaveCollectionId: feed.autoSaveCollectionId,
			pollInterval: feed.pollIntervalOverride ? String(feed.pollIntervalOverride) : 'default',
			autoSave: feed.autoSave
		};
	}

	function closeEditComposer() {
		editComposer = null;
		editError = null;
	}

	function updateEditComposer(patch: Partial<EditComposerState>) {
		if (!editComposer) return;
		editComposer = { ...editComposer, ...patch };
	}

	async function saveEditComposer() {
		if (!editComposer) return;
		editSaving = true;
		editError = null;
		const { feedId, title, autoSaveCollectionId, pollInterval, autoSave } = editComposer;
		const feed = feeds.find((candidate) => candidate.id === feedId);
		if (!feed) {
			editSaving = false;
			return;
		}

		const pollMinutes = pollIntervalToMinutes(pollInterval);
		try {
			const { data, error } = await updateSubscription({
				path: { id: feedId },
				body: {
					...(title !== feed.name ? { title } : {}),
					...(autoSaveCollectionId !== feed.autoSaveCollectionId
						? { auto_save_collection_id: autoSaveCollectionId }
						: {}),
					...(pollMinutes !== feed.pollIntervalOverride
						? { poll_interval_override_minutes: pollMinutes }
						: {}),
					...(autoSave !== feed.autoSave ? { auto_save: autoSave } : {})
				}
			});
			if (error) {
				editError = 'Failed to save changes';
			} else if (data) {
				Object.assign(feed, mapSubscription(data));
				savedSnapshot = snapshotFeeds(feeds);
				editComposer = null;
			}
		} catch {
			editError = 'Failed to save changes';
		} finally {
			editSaving = false;
		}
	}

	async function retryFeed(id: string) {
		const feed = feeds.find((candidate) => candidate.id === id);
		if (!feed) return;
		const prevStatus = feed.status;
		const prevError = feed.errorMessage;
		feed.status = 'active';
		feed.errorMessage = undefined;
		try {
			const { data, error } = await retrySubscription({ path: { id } });
			if (error) {
				feed.status = prevStatus;
				feed.errorMessage = prevError;
			} else if (data) {
				Object.assign(feed, mapSubscription(data));
			}
		} catch {
			feed.status = prevStatus;
			feed.errorMessage = prevError;
		}
	}

	async function handleOpmlUpload(file: File) {
		opmlUploading = true;
		opmlError = null;
		opmlResult = null;
		try {
			const result = await uploadOpml(file);
			if (result.ok) {
				opmlResult = result.data;
				await loadFeeds();
			} else {
				opmlError = result.error;
			}
		} catch {
			opmlError = 'An unexpected error occurred during upload';
		} finally {
			opmlUploading = false;
		}
	}

	function toggleKebab(id: string, event: MouseEvent) {
		event.stopPropagation();
		openKebabId = openKebabId === id ? null : id;
	}

	function closeKebab() {
		openKebabId = null;
	}

	function scrollToOpml() {
		document.getElementById('opml-import')?.scrollIntoView({ behavior: 'smooth', block: 'center' });
	}
</script>

<div class="page">
	<FeedHero {stats} onAddFeed={() => modal.open('rss')} onImportOpml={scrollToOpml} />

	<div class="body-area">
		{#if loadError}
			<div class="load-error">
				<p class="error-text">{loadError}</p>
				<button type="button" class="retry-btn" onclick={loadFeeds}>Retry</button>
			</div>
		{/if}

		<FeedEditComposer
			composer={editComposer}
			feed={editingFeed}
			saving={editSaving}
			error={editError}
			onClose={closeEditComposer}
			onSave={saveEditComposer}
			onChange={updateEditComposer}
		/>

		<FeedSubscriptionsPanel
			{loading}
			{feeds}
			{filteredFeeds}
			{stats}
			{activeFilter}
			{searchQuery}
			{openKebabId}
			onAddFeed={() => modal.open('rss')}
			onSearch={(query) => (searchQuery = query)}
			onFilter={(filter) => (activeFilter = filter)}
			onToggleAutoSave={toggleAutoSave}
			onToggleFeed={toggleFeed}
			onToggleMenu={toggleKebab}
			onCloseMenu={closeKebab}
			onEdit={openEditComposer}
			onRetry={retryFeed}
			onDelete={requestDelete}
		/>

		<OpmlImportCard
			uploading={opmlUploading}
			result={opmlResult}
			error={opmlError}
			onUpload={handleOpmlUpload}
		/>
	</div>

	<SavePill {isDirty} {saving} {showSaved} onSave={saveChanges} onDiscard={discardChanges} />
</div>

<FeedDeleteDialog
	open={deleteConfirmId !== null}
	feed={feedToDelete}
	onCancel={() => (deleteConfirmId = null)}
	onConfirm={confirmDelete}
/>

<style>
	.page {
		display: flex;
		flex-direction: column;
	}

	.body-area {
		padding: 32px 56px 16px;
		display: flex;
		flex-direction: column;
		max-width: 1080px;
		width: 100%;
		align-self: center;
		margin: 0 auto;
	}

	.load-error {
		display: flex;
		align-items: center;
		gap: 12px;
		padding: 16px;
		border-radius: 10px;
		background: var(--fill-danger);
		margin-bottom: 16px;
	}

	.error-text {
		font-size: 13px;
		color: var(--destructive);
		font-family: var(--font-sans);
		line-height: 1.4;
		flex: 1;
		margin: 0;
	}

	.retry-btn {
		display: inline-flex;
		padding: 5px 12px;
		border-radius: 6px;
		background: var(--accent);
		color: var(--text-on-color);
		font-family: var(--font-sans);
		font-size: 12px;
		font-weight: 500;
		border: none;
		cursor: pointer;
		flex-shrink: 0;
		transition: opacity 120ms ease;
	}

	.retry-btn:hover {
		opacity: 0.88;
	}

	@media (max-width: 760px) {
		.body-area {
			padding: 24px 20px 16px;
		}
	}
</style>
