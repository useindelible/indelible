<script lang="ts">
	import { getAuth } from '$lib/stores/auth.svelte';
	import { getModalStore } from '$lib/stores/addItemModal.svelte';
	import AddPopover from '$lib/components/library/AddPopover.svelte';
	import SaveUrlModal from '$lib/components/library/SaveUrlModal.svelte';
	import UploadFileModal from '$lib/components/library/UploadFileModal.svelte';
	import EmailForwardingModal from '$lib/components/library/EmailForwardingModal.svelte';
	import AddRssFeedModal from '$lib/components/library/AddRssFeedModal.svelte';
	import XPostModal from '$lib/components/library/XPostModal.svelte';
	import YouTubeModal from '$lib/components/library/YouTubeModal.svelte';
	import {
		addDomainEventHandler,
		startDomainEventStream,
		stopDomainEventStream
	} from '$lib/realtime/domain-events';
	import { WEB_DOMAIN_EVENT_TYPES } from '$lib/realtime/event-types';
	import { getLibrary } from '$lib/stores/library.svelte';

	let { children } = $props();
	const auth = getAuth();
	const modal = getModalStore();
	const library = getLibrary();

	$effect(() => {
		const userId = auth.user?.id;
		if (!auth.loading && auth.isAuthenticated && userId) {
			const unsubscribeLibrary = addDomainEventHandler(library.handleDomainEvent);
			startDomainEventStream(userId, { eventTypes: WEB_DOMAIN_EVENT_TYPES });
			return () => {
				unsubscribeLibrary();
				stopDomainEventStream();
			};
		}
		stopDomainEventStream();
	});

	function handleKeydown(e: KeyboardEvent) {
		if ((e.metaKey || e.ctrlKey) && e.key === 'n') {
			e.preventDefault();
			modal.open('url');
			return;
		}
		const tag = (e.target as HTMLElement)?.tagName;
		if (tag === 'INPUT' || tag === 'TEXTAREA' || (e.target as HTMLElement)?.isContentEditable)
			return;
		if (e.metaKey || e.ctrlKey || e.altKey) return;
		if (e.key === 'a' || e.key === 'A') {
			e.preventDefault();
			modal.open('url');
		} else if (e.key === 'r' || e.key === 'R') {
			e.preventDefault();
			modal.open('rss');
		}
	}
</script>

<svelte:window onkeydown={handleKeydown} />

{#if !auth.loading && auth.isAuthenticated}
	{@render children()}
	<AddPopover />
	{#if modal.active === 'url'}<SaveUrlModal />{/if}
	{#if modal.active === 'upload'}<UploadFileModal />{/if}
	{#if modal.active === 'email'}<EmailForwardingModal />{/if}
	{#if modal.active === 'rss'}<AddRssFeedModal />{/if}
	{#if modal.active === 'x'}<XPostModal />{/if}
	{#if modal.active === 'youtube'}<YouTubeModal />{/if}
{/if}
