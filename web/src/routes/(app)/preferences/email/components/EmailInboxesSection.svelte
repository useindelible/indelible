<script lang="ts">
	import type { AliasDestinationDto, EmailAliasResponse } from '$lib/api';
	import EmailAliasComposer from './EmailAliasComposer.svelte';
	import EmailEnvelopeCard from './EmailEnvelopeCard.svelte';
	import { t } from '$lib/i18n';

	interface Props {
		feedAddress: string;
		libraryAddress: string;
		feedPrimary: EmailAliasResponse | null;
		libraryPrimary: EmailAliasResponse | null;
		composerOpen: boolean;
		composerDest: AliasDestinationDto;
		composerAddress: string;
		newLocalPart: string;
		creating: boolean;
		createError: string | null;
		copied: string | null;
		onCopy: (key: string, text: string) => void;
		onOpenComposer: (destination: AliasDestinationDto) => void;
		onComposerDestination: (destination: AliasDestinationDto) => void;
		onLocalPart: (value: string) => void;
		onCloseComposer: () => void;
		onCreateAlias: () => void;
	}

	let {
		feedAddress,
		libraryAddress,
		feedPrimary,
		libraryPrimary,
		composerOpen,
		composerDest,
		composerAddress,
		newLocalPart,
		creating,
		createError,
		copied,
		onCopy,
		onOpenComposer,
		onComposerDestination,
		onLocalPart,
		onCloseComposer,
		onCreateAlias
	}: Props = $props();

	let composerOpener: HTMLButtonElement | null = null;

	function openComposer(destination: AliasDestinationDto, opener: HTMLButtonElement) {
		composerOpener = opener;
		onOpenComposer(destination);
	}

	function closeComposer() {
		onCloseComposer();
		composerOpener?.focus();
		composerOpener = null;
	}
</script>

<section class="page-section" aria-labelledby="inboxes-heading">
	<div class="section-head">
		<div class="section-head-left">
			<div class="section-eyebrow">{$t('email_inboxes_eyebrow')}</div>
			<h2 class="section-title" id="inboxes-heading">
				{$t('email_inboxes_title')} <em>{$t('email_inboxes_title_emphasis')}</em>
			</h2>
			<p class="section-desc">{$t('email_inboxes_description')}</p>
		</div>
		<div class="section-meta">{$t('email_inboxes_meta')}</div>
	</div>

	<div class="inbox-grid">
		<EmailEnvelopeCard
			dest="feed"
			label={$t('email_feed_inbox')}
			headline={$t('email_feed_inbox_hint')}
			address={feedAddress}
			primary={feedPrimary}
			copied={copied === 'primary-feed'}
			{onCopy}
			onOpenComposer={openComposer}
		/>
		<EmailEnvelopeCard
			dest="library"
			label={$t('email_library_inbox')}
			headline={$t('email_library_inbox_hint')}
			address={libraryAddress}
			primary={libraryPrimary}
			copied={copied === 'primary-library'}
			{onCopy}
			onOpenComposer={openComposer}
		/>
	</div>

	<EmailAliasComposer
		open={composerOpen}
		destination={composerDest}
		address={composerAddress}
		localPart={newLocalPart}
		{creating}
		error={createError}
		onDestination={onComposerDestination}
		{onLocalPart}
		onClose={closeComposer}
		onCreate={onCreateAlias}
	/>
</section>

<style>
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

	.section-head-left {
		display: flex;
		flex-direction: column;
		gap: 4px;
		min-width: 0;
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

	.inbox-grid {
		display: grid;
		grid-template-columns: 1fr 1fr;
		gap: 18px;
	}

	@media (max-width: 760px) {
		.section-head {
			flex-direction: column;
		}

		.inbox-grid {
			grid-template-columns: 1fr;
		}
	}
</style>
