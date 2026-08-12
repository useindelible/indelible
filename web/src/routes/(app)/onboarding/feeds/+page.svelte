<script lang="ts">
	import { goto } from '$app/navigation';
	import { resolve } from '$app/paths';
	import StepLayout from '$lib/components/onboarding/StepLayout.svelte';
	import { getOnboarding } from '$lib/stores/onboarding.svelte';
	import { subscribe } from '$lib/api';
	import { uploadOpml } from '$lib/api/feeds';

	const onboarding = getOnboarding();

	const suggestedFeeds = [
		{
			initials: 'TC',
			tone: 'success',
			name: 'TechCrunch',
			domain: 'techcrunch.com',
			url: 'https://techcrunch.com/feed/',
			selected: false
		},
		{
			initials: 'HN',
			tone: 'warning',
			name: 'Hacker News',
			domain: 'news.ycombinator.com',
			url: 'https://hnrss.org/frontpage',
			selected: false
		},
		{
			initials: 'TV',
			tone: 'danger',
			name: 'The Verge',
			domain: 'theverge.com',
			url: 'https://www.theverge.com/rss/index.xml',
			selected: false
		},
		{
			initials: 'AT',
			tone: 'warning',
			name: 'Ars Technica',
			domain: 'arstechnica.com',
			url: 'https://feeds.arstechnica.com/arstechnica/index',
			selected: false
		},
		{
			initials: 'ST',
			tone: 'accent',
			name: 'Stratechery',
			domain: 'stratechery.com',
			url: 'https://stratechery.com/feed/',
			selected: false
		},
		{
			initials: 'WD',
			tone: 'neutral',
			name: 'Wired',
			domain: 'wired.com',
			url: 'https://www.wired.com/feed/rss',
			selected: false
		},
		{
			initials: 'MT',
			tone: 'neutral',
			name: 'MIT Tech Review',
			domain: 'technologyreview.com',
			url: 'https://www.technologyreview.com/topnews.rss',
			selected: false
		}
	];

	let feedState = $state(suggestedFeeds.map((f) => ({ ...f })));
	let rssUrl = $state('');
	let subscribing = $state(false);
	let subscribeMessage = $state('');
	let subscribeError = $state('');
	let submitting = $state(false);
	let opmlMessage = $state('');
	let uploadingOpml = $state(false);

	const selectedUrls = $derived(feedState.filter((f) => f.selected).map((f) => f.url));

	function isValidFeedUrl(input: string): boolean {
		try {
			const url = new URL(input);
			return url.protocol === 'http:' || url.protocol === 'https:';
		} catch {
			return false;
		}
	}

	async function handleSubscribe() {
		const url = rssUrl.trim();
		if (!url) return;
		if (!isValidFeedUrl(url)) {
			subscribeError = 'Enter a valid URL starting with http:// or https://.';
			subscribeMessage = '';
			return;
		}

		subscribing = true;
		subscribeError = '';
		subscribeMessage = '';
		try {
			const { data, error: apiError, response } = await subscribe({ body: { url } });
			if (data) {
				subscribeMessage = data.is_new ? 'Subscribed successfully.' : 'Already subscribed.';
				rssUrl = '';
				return;
			}
			subscribeError = feedSubscriptionError(apiError, response);
		} catch {
			subscribeError = 'An unexpected error occurred.';
		} finally {
			subscribing = false;
		}
	}

	function feedSubscriptionError(apiError: unknown, response: Response | undefined): string {
		if (apiError && typeof apiError === 'object') {
			const error = apiError as Record<string, unknown>;
			if (typeof error.detail === 'string') return error.detail;
			if (typeof error.message === 'string') return error.message;
		}
		return response?.status === 422 ? 'Invalid feed URL.' : 'Failed to subscribe to feed.';
	}

	async function handleContinue() {
		submitting = true;
		try {
			const urls = [...selectedUrls];
			const completed = await onboarding.completeStep(3, { feed_urls: urls });
			if (completed) {
				goto(resolve('/onboarding/ai'));
			}
		} finally {
			submitting = false;
		}
	}

	async function handleSkip() {
		submitting = true;
		try {
			const completed = await onboarding.completeStep(3);
			if (completed) {
				goto(resolve('/onboarding/ai'));
			}
		} finally {
			submitting = false;
		}
	}

	async function handleOpml(event: Event) {
		const file = (event.currentTarget as HTMLInputElement).files?.[0];
		if (!file) return;
		uploadingOpml = true;
		opmlMessage = '';
		const result = await uploadOpml(file);
		opmlMessage = result.ok
			? `Imported ${result.data.created} feed${result.data.created === 1 ? '' : 's'}.`
			: result.error;
		uploadingOpml = false;
	}
</script>

<StepLayout
	title="Subscribe to your favorite sources"
	description="Follow publications and blogs via RSS. Toggle on the ones you want."
	currentStep={3}
	showSkip
	{submitting}
	onContinue={handleContinue}
	onSkip={handleSkip}
>
	<div class="feeds-content">
		<p class="section-label">Suggested feeds</p>

		<div class="feed-list">
			{#each feedState as feed (feed.url)}
				<label class="feed-row">
					<div class="feed-avatar tone-{feed.tone}" aria-hidden="true">
						{feed.initials}
					</div>
					<div class="feed-info">
						<span class="feed-name">{feed.name}</span>
						<span class="feed-domain">{feed.domain}</span>
					</div>
					<input
						type="checkbox"
						class="toggle"
						bind:checked={feed.selected}
						aria-label="Subscribe to {feed.name}"
					/>
				</label>
			{/each}
		</div>
		<p class="selection-summary" role="status" aria-live="polite">
			{#if submitting}
				Saving your feed choices…
			{:else if selectedUrls.length === 0}
				No suggested feeds selected.
			{:else}
				{selectedUrls.length} suggested {selectedUrls.length === 1 ? 'feed' : 'feeds'} selected.
			{/if}
		</p>

		<p class="section-label manual-label">Add feed manually</p>
		<form
			class="manual-row"
			onsubmit={(event) => {
				event.preventDefault();
				handleSubscribe();
			}}
		>
			<input
				type="url"
				class="url-input"
				bind:value={rssUrl}
				placeholder="https://example.com/feed.xml"
				aria-label="RSS feed URL"
				disabled={subscribing}
			/>
			<button type="submit" class="subscribe-btn" disabled={subscribing || !rssUrl.trim()}>
				{subscribing ? 'Subscribing…' : 'Subscribe'}
			</button>
		</form>
		{#if subscribeMessage}<p class="form-success" role="status">{subscribeMessage}</p>{/if}
		{#if subscribeError}<p class="form-error" role="alert">{subscribeError}</p>{/if}
		<p class="section-label manual-label">Import subscriptions</p>
		<label class="opml-zone">
			<input
				class="opml-input"
				type="file"
				accept=".opml,.xml,text/xml,application/xml"
				onchange={handleOpml}
				disabled={uploadingOpml}
			/>
			<strong>{uploadingOpml ? 'Importing…' : 'Drop an OPML file or choose one'}</strong>
			<span>Imports feeds from Feedly, Inoreader, NetNewsWire, and other readers.</span>
		</label>
		{#if opmlMessage}<p class="form-error">{opmlMessage}</p>{/if}

		{#if onboarding.error}
			<p class="form-error">{onboarding.error}</p>
		{/if}
	</div>
</StepLayout>

<style>
	.feeds-content {
		display: flex;
		flex-direction: column;
	}

	.section-label {
		font-family: var(--font-sans);
		font-size: 13px;
		font-weight: 600;
		letter-spacing: -0.01em;
		color: var(--text-secondary);
		margin: 0 0 8px;
	}

	.feed-list {
		display: grid;
		grid-template-columns: 1fr 1fr;
		gap: 2px;
	}

	.selection-summary {
		margin: 8px 0 0;
		font-family: var(--font-sans);
		font-size: 12px;
		line-height: 1.4;
		color: var(--text-secondary);
	}

	.feed-row {
		display: flex;
		align-items: center;
		gap: 12px;
		padding: 12px 14px;
		border-radius: 10px;
		cursor: pointer;
		background: var(--fill-secondary);
		transition: background 0.12s ease;
	}

	:global([data-theme='dark']) .feed-row {
		background: var(--fill-secondary);
	}

	.feed-row:hover {
		background: var(--fill-hover);
	}

	:global([data-theme='dark']) .feed-row:hover {
		background: var(--fill-hover);
	}

	.feed-avatar {
		width: 32px;
		height: 32px;
		border-radius: 8px;
		display: flex;
		align-items: center;
		justify-content: center;
		flex-shrink: 0;
		font-family: var(--font-sans);
		font-size: 11px;
		font-weight: 700;
		letter-spacing: -0.01em;
	}

	.tone-success {
		background: var(--fill-success);
		color: var(--success);
	}

	.tone-warning {
		background: var(--fill-warning);
		color: var(--warning);
	}

	.tone-danger {
		background: var(--fill-danger);
		color: var(--destructive);
	}

	.tone-accent {
		background: var(--fill-selected);
		color: var(--accent);
	}

	.tone-neutral {
		background: var(--fill-secondary);
		color: var(--text-secondary);
	}

	.feed-info {
		flex: 1;
		display: flex;
		flex-direction: column;
		gap: 1px;
		min-width: 0;
		overflow: hidden;
	}

	.feed-name {
		font-family: var(--font-sans);
		font-size: 13px;
		font-weight: 600;
		letter-spacing: -0.01em;
		color: var(--text-primary);
		white-space: nowrap;
		overflow: hidden;
		text-overflow: ellipsis;
	}

	.feed-domain {
		font-family: var(--font-sans);
		font-size: 11px;
		font-weight: 400;
		letter-spacing: -0.005em;
		color: var(--text-secondary);
		white-space: nowrap;
		overflow: hidden;
		text-overflow: ellipsis;
	}

	.toggle {
		-webkit-appearance: none;
		appearance: none;
		position: relative;
		width: 44px;
		height: 26px;
		border-radius: 13px;
		background: var(--border-secondary);
		cursor: pointer;
		flex-shrink: 0;
		transition: background 0.2s ease;
	}

	.toggle::after {
		content: '';
		position: absolute;
		top: 3px;
		left: 3px;
		width: 20px;
		height: 20px;
		border-radius: 50%;
		background: var(--text-on-color);
		box-shadow: 0 1px 3px rgba(0, 0, 0, 0.2);
		transition: transform 0.2s ease;
	}

	.toggle:checked {
		background: var(--accent);
	}

	.toggle:checked::after {
		transform: translateX(18px);
	}

	.manual-row {
		display: flex;
		border-radius: var(--radius-sm);
		overflow: hidden;
		border: 1px solid var(--border-secondary);
	}

	.manual-label {
		margin-top: 20px;
	}

	.url-input {
		flex: 1;
		padding: 0 14px;
		height: 40px;
		font-family: var(--font-sans);
		font-size: 15px;
		letter-spacing: -0.01em;
		color: var(--text-primary);
		background: var(--bg-elevated);
		border: none;
		outline: none;
	}

	.url-input::placeholder {
		color: var(--text-secondary);
	}

	.subscribe-btn {
		height: 40px;
		padding: 0 16px;
		background: var(--accent);
		color: var(--text-on-color);
		border: none;
		font-family: var(--font-sans);
		font-size: 15px;
		font-weight: 500;
		letter-spacing: -0.01em;
		cursor: pointer;
		transition: background 0.15s ease;
		flex-shrink: 0;
	}

	.subscribe-btn:hover:not(:disabled) {
		background: var(--accent-hover);
	}

	.subscribe-btn:disabled {
		opacity: 0.5;
		cursor: not-allowed;
	}

	.opml-zone {
		display: flex;
		flex-direction: column;
		align-items: center;
		justify-content: center;
		gap: 4px;
		min-height: 82px;
		padding: 16px;
		border: 1px dashed var(--border-secondary);
		border-radius: 10px;
		background: var(--fill-selected);
		color: var(--text-primary);
		text-align: center;
		cursor: pointer;
		transition:
			background 0.15s ease,
			border-color 0.15s ease;
	}

	.opml-zone:hover {
		border-color: var(--accent);
		background: var(--fill-selected-strong);
	}

	.opml-zone strong {
		font-size: 13px;
		font-weight: 600;
	}

	.opml-zone span {
		color: var(--text-secondary);
		font-size: 12px;
		line-height: 1.4;
	}

	.opml-input {
		position: absolute;
		width: 1px;
		height: 1px;
		padding: 0;
		margin: -1px;
		overflow: hidden;
		clip: rect(0, 0, 0, 0);
		white-space: nowrap;
		border: 0;
	}

	.opml-zone:focus-within {
		outline: 2px solid var(--accent);
		outline-offset: 2px;
	}

	.form-error {
		margin: 10px 0 0;
		font-family: var(--font-sans);
		font-size: 13px;
		line-height: 1.4;
		color: var(--destructive);
	}

	.form-success {
		margin: 10px 0 0;
		font-family: var(--font-sans);
		font-size: 13px;
		line-height: 1.4;
		color: var(--success);
	}
</style>
