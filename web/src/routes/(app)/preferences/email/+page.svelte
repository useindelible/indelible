<script lang="ts">
	import { onMount } from 'svelte';
	import {
		createEmailAlias,
		listEmailAliases,
		listEmailSenders,
		unsubscribeEmailSender,
		updateEmailSender
	} from '$lib/api';
	import type {
		AliasDestinationDto,
		EmailAliasResponse,
		EmailSenderResponse,
		RenderDefaultDto
	} from '$lib/api';
	import { getAuth } from '$lib/stores/auth.svelte';
	import EmailHero from './components/EmailHero.svelte';
	import EmailInboxesSection from './components/EmailInboxesSection.svelte';
	import EmailSenderFilters from './components/EmailSenderFilters.svelte';
	import EmailSenderTable from './components/EmailSenderTable.svelte';
	import {
		extractErrorMessage,
		filterSenders,
		isValidLocalPart,
		primaryAlias,
		routingPatchValue,
		senderCounts,
		type SenderFilter
	} from './email-model';

	const auth = getAuth();

	let aliases = $state<EmailAliasResponse[]>([]);
	let senders = $state<EmailSenderResponse[]>([]);
	let loading = $state(true);
	let loadError = $state<string | null>(null);
	let composerOpen = $state(false);
	let composerDest = $state<AliasDestinationDto>('feed');
	let newLocalPart = $state('');
	let creating = $state(false);
	let createError = $state<string | null>(null);
	let copied = $state<string | null>(null);
	let activeFilter = $state<SenderFilter>('all');
	let search = $state('');
	let updatingSender = $state<string | null>(null);
	let unsubscribingSender = $state<string | null>(null);

	const feedPrimary = $derived(primaryAlias(aliases, 'feed'));
	const libraryPrimary = $derived(primaryAlias(aliases, 'library'));
	const feedAddress = $derived(feedPrimary?.address ?? auth.user?.ingest_email ?? '');
	const libraryAddress = $derived(libraryPrimary?.address ?? auth.user?.ingest_library_email ?? '');
	const composerAddress = $derived(composerDest === 'feed' ? feedAddress : libraryAddress);
	const totalDeliveries = $derived(senders.reduce((sum, sender) => sum + sender.delivery_count, 0));
	const totalBlocked = $derived(senders.filter((sender) => sender.blocked).length);
	const lastDelivery = $derived(
		senders
			.map((sender) => sender.last_seen_at)
			.filter(Boolean)
			.sort()
			.at(-1)
	);
	const counts = $derived(senderCounts(senders));
	const filteredSenders = $derived(filterSenders(senders, activeFilter, search));

	onMount(() => {
		void load();
	});

	async function load() {
		loading = true;
		loadError = null;
		try {
			const [aliasResult, senderResult] = await Promise.all([
				listEmailAliases(),
				listEmailSenders()
			]);
			if (aliasResult.data) {
				aliases = aliasResult.data.data;
			} else {
				loadError = extractErrorMessage(
					aliasResult.error,
					aliasResult.response,
					'Failed to load aliases'
				);
			}
			if (senderResult.data) {
				senders = senderResult.data.data;
			}
		} catch {
			loadError = 'An unexpected error occurred';
		} finally {
			loading = false;
		}
	}

	function openComposer(destination: AliasDestinationDto) {
		composerDest = destination;
		composerOpen = true;
		newLocalPart = '';
		createError = null;
	}

	function closeComposer() {
		composerOpen = false;
		newLocalPart = '';
		createError = null;
	}

	async function handleCreateAlias() {
		if (!composerOpen) return;
		const localPart = newLocalPart.trim().toLowerCase();
		if (!isValidLocalPart(localPart)) return;
		creating = true;
		createError = null;
		try {
			const { data, error, response } = await createEmailAlias({
				body: { destination: composerDest, local_part: localPart, is_default: true }
			});
			if (data) {
				await load();
				closeComposer();
			} else {
				createError = extractErrorMessage(error, response, 'Failed to create alias');
			}
		} catch {
			createError = 'An unexpected error occurred';
		} finally {
			creating = false;
		}
	}

	async function copyText(key: string, text: string) {
		try {
			await navigator.clipboard.writeText(text);
			copied = key;
			setTimeout(() => {
				if (copied === key) copied = null;
			}, 1800);
		} catch {
			// Clipboard is optional in non-browser test environments.
		}
	}

	async function handleToggleBlock(sender: EmailSenderResponse) {
		updatingSender = sender.id;
		try {
			const { data } = await updateEmailSender({
				path: { id: sender.id },
				body: { blocked: !sender.blocked }
			});
			if (data) senders = senders.map((current) => (current.id === sender.id ? data : current));
		} finally {
			updatingSender = null;
		}
	}

	async function handleRenderChange(sender: EmailSenderResponse, value: RenderDefaultDto) {
		updatingSender = sender.id;
		try {
			const { data } = await updateEmailSender({
				path: { id: sender.id },
				body: { render_default: value }
			});
			if (data) senders = senders.map((current) => (current.id === sender.id ? data : current));
		} finally {
			updatingSender = null;
		}
	}

	async function handleRoutingChange(sender: EmailSenderResponse, raw: string) {
		updatingSender = sender.id;
		try {
			const { data } = await updateEmailSender({
				path: { id: sender.id },
				body: { routing_default: routingPatchValue(raw) }
			});
			if (data) senders = senders.map((current) => (current.id === sender.id ? data : current));
		} finally {
			updatingSender = null;
		}
	}

	async function handleUnsubscribe(sender: EmailSenderResponse) {
		unsubscribingSender = sender.id;
		try {
			const { data } = await unsubscribeEmailSender({ path: { id: sender.id } });
			if (data) {
				senders = senders.map((current) =>
					current.id === sender.id
						? { ...current, blocked: true, blocked_at: data.blocked_at }
						: current
				);
			}
		} finally {
			unsubscribingSender = null;
		}
	}
</script>

<svelte:head>
	<link rel="preconnect" href="https://fonts.gstatic.com" crossorigin="anonymous" />
	<link
		href="https://fonts.googleapis.com/css2?family=Fraunces:ital,opsz,wght@0,9..144,400..700;1,9..144,400..700&family=DM+Sans:opsz,wght@9..40,400..700&family=IBM+Plex+Mono:wght@400;500;600&display=swap"
		rel="stylesheet"
	/>
</svelte:head>

<div class="email-page">
	<EmailHero senderCount={senders.length} {totalDeliveries} {totalBlocked} {lastDelivery} />

	<div class="body-area">
		{#if loadError}
			<p class="form-error">{loadError}</p>
		{/if}

		<EmailInboxesSection
			{feedAddress}
			{libraryAddress}
			{feedPrimary}
			{libraryPrimary}
			{composerOpen}
			{composerDest}
			{composerAddress}
			{newLocalPart}
			{creating}
			{createError}
			{copied}
			onCopy={copyText}
			onOpenComposer={openComposer}
			onComposerDestination={(destination) => (composerDest = destination)}
			onLocalPart={(value) => (newLocalPart = value)}
			onCloseComposer={closeComposer}
			onCreateAlias={handleCreateAlias}
		/>

		<section class="page-section" aria-labelledby="senders-heading">
			<div class="section-head">
				<div>
					<div class="section-eyebrow">Section ii · Register</div>
					<h2 class="section-title" id="senders-heading">The senders <em>register</em></h2>
					<p class="section-desc">
						Every address that has written to you. Set how each one is rendered, where each lands,
						block what you don&rsquo;t want, unsubscribe what you no longer need.
					</p>
				</div>
				<div class="section-meta">{senders.length} entries</div>
			</div>

			<EmailSenderFilters
				{activeFilter}
				{search}
				{counts}
				onFilter={(filter) => (activeFilter = filter)}
				onSearch={(value) => (search = value)}
			/>

			{#if loading}
				<div class="ledger-loading">Loading senders…</div>
			{:else}
				<EmailSenderTable
					senders={filteredSenders}
					totalSenders={senders.length}
					{updatingSender}
					{unsubscribingSender}
					onRenderChange={handleRenderChange}
					onRoutingChange={handleRoutingChange}
					onToggleBlock={handleToggleBlock}
					onUnsubscribe={handleUnsubscribe}
				/>
			{/if}
		</section>
	</div>
</div>

<style>
	.email-page {
		--canvas: #fbf8f4;
		--paper: #ffffff;
		--paper-soft: #f6efe0;
		--paper-deep: #ede3cc;
		--ink: #1a1612;
		--accent-strong: #8b1a1f;
		--accent-soft: rgba(176, 37, 43, 0.1);
		--accent-tint: rgba(176, 37, 43, 0.06);
		--accent-line: rgba(176, 37, 43, 0.32);
		--warning-soft: rgba(140, 83, 0, 0.12);
		--border-hairline: rgba(26, 22, 18, 0.06);
		--border-ledger: rgba(26, 22, 18, 0.08);
		--fill-selected: rgba(176, 37, 43, 0.1);
		--fill-selected-strong: rgba(176, 37, 43, 0.18);
		--envelope-bg: #ffffff;
		--envelope-edge: rgba(26, 22, 18, 0.08);
		--envelope-shadow: 0 1px 2px rgba(40, 30, 15, 0.05), 0 0 0 0.5px rgba(26, 22, 18, 0.08);
		--envelope-shadow-hover: 0 4px 14px rgba(80, 60, 30, 0.08), 0 0 0 0.5px rgba(26, 22, 18, 0.14);
		--perf-dot: rgba(26, 22, 18, 0.22);
		--stamp-fill: rgba(176, 37, 43, 0.06);
		--stamp-ink: #8b1a1f;
		--stamp-line: rgba(176, 37, 43, 0.45);
		--code-bg: rgba(26, 22, 18, 0.04);
		--table-head-bg: rgba(26, 22, 18, 0.02);
		--table-row-hover: rgba(176, 37, 43, 0.03);
		--table-row-blocked: rgba(26, 22, 18, 0.025);
		--chip-bg: rgba(26, 22, 18, 0.04);
		--chip-active-bg: rgba(176, 37, 43, 0.1);
		--chip-active-text: #8b1a1f;
		--chip-active-border: rgba(176, 37, 43, 0.3);
		--airmail-red: #b0252b;
		--airmail-navy: #1e2a55;
		--hero-from: #f2ede0;
		--hero-to: #e9dfc9;
		--hero-blob-a: rgba(176, 37, 43, 0.16);
		--hero-blob-b: rgba(20, 30, 80, 0.1);
		--hero-blob-c: rgba(214, 180, 120, 0.5);
		--font-display: 'Fraunces', 'Iowan Old Style', 'Apple Garamond', Georgia, serif;
		--font-body: 'DM Sans', -apple-system, BlinkMacSystemFont, 'Segoe UI', Helvetica, sans-serif;
		--font-mono: 'IBM Plex Mono', 'SF Mono', Menlo, Consolas, monospace;

		display: flex;
		flex-direction: column;
		width: 100%;
		min-height: 100%;
		font-family: var(--font-body);
		color: var(--text-primary);
		background: var(--canvas);
	}

	:global([data-theme='dark']) .email-page {
		--canvas: #100d0b;
		--paper: #211d18;
		--paper-soft: #1a1612;
		--paper-deep: #14110f;
		--ink: #f0eae0;
		--accent-strong: #f09ea1;
		--accent-soft: rgba(224, 124, 127, 0.14);
		--accent-tint: rgba(224, 124, 127, 0.08);
		--accent-line: rgba(224, 124, 127, 0.36);
		--warning-soft: rgba(255, 177, 67, 0.14);
		--border-hairline: rgba(240, 234, 224, 0.06);
		--border-ledger: rgba(240, 234, 224, 0.08);
		--fill-selected: rgba(224, 124, 127, 0.14);
		--fill-selected-strong: rgba(224, 124, 127, 0.24);
		--envelope-bg: #211d18;
		--envelope-edge: rgba(240, 234, 224, 0.08);
		--envelope-shadow: 0 2px 10px rgba(0, 0, 0, 0.45), 0 0 0 0.5px rgba(240, 234, 224, 0.07);
		--envelope-shadow-hover:
			0 8px 24px rgba(224, 124, 127, 0.22), 0 0 0 0.5px rgba(224, 124, 127, 0.36);
		--perf-dot: rgba(240, 234, 224, 0.22);
		--stamp-fill: rgba(224, 124, 127, 0.1);
		--stamp-ink: #f09ea1;
		--stamp-line: rgba(224, 124, 127, 0.55);
		--code-bg: rgba(240, 234, 224, 0.06);
		--table-head-bg: rgba(240, 234, 224, 0.03);
		--table-row-hover: rgba(224, 124, 127, 0.06);
		--table-row-blocked: rgba(240, 234, 224, 0.03);
		--chip-bg: rgba(240, 234, 224, 0.06);
		--chip-active-bg: rgba(224, 124, 127, 0.18);
		--chip-active-text: #f09ea1;
		--chip-active-border: rgba(224, 124, 127, 0.4);
		--airmail-red: #e07c7f;
		--airmail-navy: #6680d2;
		--hero-from: #1f1816;
		--hero-to: #2a1f1c;
		--hero-blob-a: rgba(224, 124, 127, 0.26);
		--hero-blob-b: rgba(102, 128, 210, 0.16);
		--hero-blob-c: rgba(160, 120, 70, 0.18);
	}

	.body-area {
		padding: 40px 56px 56px;
		flex: 1;
		display: flex;
		flex-direction: column;
		gap: 40px;
		max-width: 1080px;
		width: 100%;
		align-self: center;
		margin: 0 auto;
	}

	.page-section {
		display: flex;
		flex-direction: column;
		gap: 18px;
	}

	.section-head {
		display: flex;
		align-items: baseline;
		justify-content: space-between;
		padding-bottom: 14px;
		border-bottom: 0.5px solid var(--border-primary);
		gap: 24px;
	}

	.section-eyebrow {
		font-family: var(--font-mono);
		font-size: 9.5px;
		font-weight: 600;
		letter-spacing: 0.2em;
		text-transform: uppercase;
		color: var(--accent);
	}

	.section-title {
		font-family: var(--font-display);
		font-size: 26px;
		font-weight: 500;
		letter-spacing: -0.015em;
		color: var(--text-primary);
		line-height: 1.15;
		margin: 0;
	}

	.section-title em {
		font-style: italic;
		font-weight: 500;
		color: var(--accent);
	}

	.section-desc {
		font-size: 13.5px;
		color: var(--text-secondary);
		letter-spacing: -0.005em;
		line-height: 1.5;
		max-width: 520px;
		margin: 6px 0 0;
	}

	.section-meta {
		font-family: var(--font-mono);
		font-size: 10.5px;
		color: var(--text-tertiary);
		letter-spacing: 0.1em;
		text-transform: uppercase;
		white-space: nowrap;
	}

	.form-error {
		color: var(--destructive);
		font-size: 13px;
		margin: 0;
	}

	.ledger-loading {
		background: var(--paper);
		border-radius: var(--radius-lg);
		box-shadow: var(--envelope-shadow);
		padding: 24px;
		color: var(--text-tertiary);
		font-size: 13px;
		text-align: center;
	}

	@media (max-width: 720px) {
		.body-area {
			padding: 28px 20px 36px;
		}

		.section-head {
			flex-direction: column;
		}
	}
</style>
