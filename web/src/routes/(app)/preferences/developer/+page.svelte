<script lang="ts">
	import { onMount, untrack } from 'svelte';
	import { SvelteSet } from 'svelte/reactivity';
	import { page } from '$app/state';
	import SavePill from '$lib/components/settings/SavePill.svelte';
	import { createApiToken, loadApiTokens, revokeApiToken } from '$lib/api/tokens';
	import {
		createWebhookEndpoint,
		deleteWebhookEndpoint,
		listWebhookDeliveries,
		listWebhookEndpoints,
		rotateWebhookSecret,
		testWebhookEndpoint,
		updateWebhookEndpoint,
		type WebhookDelivery,
		type WebhookEndpoint
	} from '$lib/api/webhooks';
	import type { ApiTokenResponse } from '$lib/api';
	import { t } from '$lib/i18n';
	import ApiTokenPanel from './components/ApiTokenPanel.svelte';
	import DeveloperHero from './components/DeveloperHero.svelte';
	import WebhookPanel from './components/WebhookPanel.svelte';
	import {
		allPermissionsSelected,
		issuePresetFromSearchParams,
		ISSUE_DEFAULTS,
		nextIssuePermissions,
		setResourceAccess,
		setsEqual,
		toggleAllPermissions,
		tokenRequest,
		type ExpiryOption,
		type PermissionKey,
		type ResourceAccessLevel,
		type ResourcePermissionKey
	} from './developer-model';

	let tokens = $state<ApiTokenResponse[]>([]);
	let tokensLoading = $state(true);
	let tokensError = $state<string | null>(null);
	let issueOpen = $state(false);
	let issueName = $state(ISSUE_DEFAULTS.name);
	let issuePermissions: Set<PermissionKey> = new SvelteSet(ISSUE_DEFAULTS.permissions);
	let issueExpiry = $state<ExpiryOption>(ISSUE_DEFAULTS.expiry);
	let creatingToken = $state(false);
	let issueError = $state<string | null>(null);
	let revealToken = $state<string | null>(null);
	let copied = $state(false);

	let endpoints = $state<WebhookEndpoint[]>([]);
	let expandedEndpoint = $state<string | null>(null);
	let deliveriesByEndpoint = $state<Record<string, WebhookDelivery[]>>({});
	let testEventByEndpoint = $state<Record<string, string>>({});
	let addOpen = $state(false);
	let addName = $state('');
	let addUrl = $state('');
	let addEvents: Set<string> = new SvelteSet();
	let addActive = $state(true);
	let creatingEndpoint = $state(false);
	let addError = $state<string | null>(null);
	let revealWebhookSecret = $state<{ name: string; raw_secret: string } | null>(null);
	let webhookSecretCopied = $state(false);

	let saving = $state(false);
	let showSaved = $state(false);
	let savedTimer: ReturnType<typeof setTimeout> | null = null;

	const isDirty = $derived(
		issueName !== ISSUE_DEFAULTS.name ||
			!setsEqual(issuePermissions, new SvelteSet(ISSUE_DEFAULTS.permissions)) ||
			issueExpiry !== ISSUE_DEFAULTS.expiry ||
			addEvents.size > 0 ||
			addName.length > 0 ||
			addUrl.length > 0 ||
			!addActive
	);

	const tokenCount = $derived(tokens.length);
	const endpointCount = $derived(endpoints.length);

	onMount(() => {
		void refreshTokens();
		void refreshEndpoints();
		const preset = issuePresetFromSearchParams(page.url.searchParams);
		if (preset) {
			replaceIssuePermissions(preset.permissions);
			issueName = $t('prefs_developer_obsidian_plugin');
			issueOpen = true;
		}
	});

	async function refreshTokens() {
		tokensLoading = true;
		tokensError = null;
		const res = await loadApiTokens();
		tokensLoading = false;
		if (res.success) tokens = res.data;
		else tokensError = res.error;
	}

	async function refreshEndpoints() {
		endpoints = await listWebhookEndpoints();
	}

	function replaceIssuePermissions(permissions: Iterable<PermissionKey>) {
		issuePermissions.clear();
		for (const permission of permissions) issuePermissions.add(permission);
	}

	function toggleIssuePermission(permission: PermissionKey) {
		replaceIssuePermissions(nextIssuePermissions(issuePermissions, permission));
	}

	function setIssueResourceAccess(resource: ResourcePermissionKey, level: ResourceAccessLevel) {
		replaceIssuePermissions(setResourceAccess(issuePermissions, resource, level));
	}

	function toggleIssueAllPermissions() {
		replaceIssuePermissions(toggleAllPermissions(issuePermissions));
	}

	async function submitIssueToken() {
		if (issuePermissions.size === 0) {
			issueError = $t('prefs_developer_error_permission_required');
			return;
		}
		if (issueName.trim().length === 0) {
			issueError = $t('prefs_developer_error_name_required');
			return;
		}

		creatingToken = true;
		issueError = null;
		const res = await createApiToken(tokenRequest(issueName, issuePermissions, issueExpiry));
		creatingToken = false;
		if (!res.success) {
			issueError = res.error;
			return;
		}

		revealToken = res.data.raw_token;
		issueOpen = false;
		await refreshTokens();
		discardIssueForm();
	}

	async function copyTokenToClipboard() {
		if (!revealToken) return;
		try {
			await navigator.clipboard.writeText(revealToken);
			copied = true;
			setTimeout(() => (copied = false), 1400);
		} catch {
			// Clipboard is optional in non-browser test environments.
		}
	}

	async function revoke(tokenId: string) {
		const res = await revokeApiToken(tokenId);
		if (res.success) await refreshTokens();
		else tokensError = res.error;
	}

	function toggleEvent(event: string) {
		if (addEvents.has(event)) addEvents.delete(event);
		else addEvents.add(event);
	}

	function toggleGroupAll(events: string[]) {
		const allSelected = events.every((event) => addEvents.has(event));
		for (const event of events) {
			if (allSelected) addEvents.delete(event);
			else addEvents.add(event);
		}
	}

	async function submitCreateEndpoint() {
		if (!addUrl.trim().toLowerCase().startsWith('https://')) {
			addError = $t('prefs_developer_error_https_required');
			return;
		}
		if (addEvents.size === 0) {
			addError = $t('prefs_developer_error_event_required');
			return;
		}

		creatingEndpoint = true;
		addError = null;
		const created = await createWebhookEndpoint({
			name: addName.trim() || addUrl.trim(),
			url: addUrl.trim(),
			events: Array.from(addEvents),
			is_active: addActive
		});
		revealWebhookSecret = { name: created.name, raw_secret: created.raw_secret };
		webhookSecretCopied = false;
		creatingEndpoint = false;
		discardAddForm();
		addOpen = false;
		await refreshEndpoints();
	}

	async function toggleExpanded(id: string) {
		expandedEndpoint = expandedEndpoint === id ? null : id;
		if (expandedEndpoint === id && !deliveriesByEndpoint[id]) {
			deliveriesByEndpoint = { ...deliveriesByEndpoint, [id]: await listWebhookDeliveries(id) };
		}
	}

	async function rotateSecret(id: string) {
		const rotated = await rotateWebhookSecret(id);
		revealWebhookSecret = { name: rotated.name, raw_secret: rotated.raw_secret };
		webhookSecretCopied = false;
		await refreshEndpoints();
	}

	async function sendTest(id: string) {
		const endpoint = endpoints.find((candidate) => candidate.id === id);
		if (!endpoint) return;
		const event = testEventByEndpoint[id] ?? endpoint.events[0] ?? 'library_entry.saved';
		await testWebhookEndpoint(id, event);
		deliveriesByEndpoint = { ...deliveriesByEndpoint, [id]: await listWebhookDeliveries(id) };
		await refreshEndpoints();
	}

	async function copyWebhookSecretToClipboard() {
		if (!revealWebhookSecret) return;
		try {
			await navigator.clipboard.writeText(revealWebhookSecret.raw_secret);
			webhookSecretCopied = true;
			setTimeout(() => (webhookSecretCopied = false), 1400);
		} catch {
			// Clipboard is optional in non-browser test environments.
		}
	}

	async function toggleEndpointActive(id: string, next: boolean) {
		await updateWebhookEndpoint(id, { is_active: next });
		await refreshEndpoints();
	}

	async function removeEndpoint(id: string) {
		await deleteWebhookEndpoint(id);
		if (expandedEndpoint === id) expandedEndpoint = null;
		await refreshEndpoints();
	}

	function setTestEvent(id: string, event: string) {
		testEventByEndpoint = { ...testEventByEndpoint, [id]: event };
	}

	function onSavePill() {
		if (!isDirty || saving) return;
		saving = true;
		setTimeout(() => {
			saving = false;
			showSaved = true;
			discardForms();
			if (savedTimer) clearTimeout(savedTimer);
			savedTimer = setTimeout(() => (showSaved = false), 1800);
		}, 700);
	}

	function onDiscardPill() {
		discardForms();
		showSaved = false;
		if (savedTimer) {
			clearTimeout(savedTimer);
			savedTimer = null;
		}
	}

	function discardForms() {
		discardIssueForm();
		discardAddForm();
	}

	function discardIssueForm() {
		issueName = ISSUE_DEFAULTS.name;
		replaceIssuePermissions(ISSUE_DEFAULTS.permissions);
		issueExpiry = ISSUE_DEFAULTS.expiry;
		issueError = null;
	}

	function closeIssueForm() {
		issueOpen = false;
		discardIssueForm();
	}

	function discardAddForm() {
		addName = '';
		addUrl = '';
		addEvents.clear();
		addActive = true;
	}

	$effect(() => {
		if (isDirty) {
			untrack(() => {
				if (showSaved) showSaved = false;
			});
		}
	});
</script>

<div class="page">
	<DeveloperHero {tokenCount} {endpointCount} />

	<div class="settings-body">
		<ApiTokenPanel
			{tokens}
			loading={tokensLoading}
			error={tokensError}
			{tokenCount}
			{issueOpen}
			{issueName}
			{issuePermissions}
			{issueExpiry}
			{creatingToken}
			{issueError}
			{revealToken}
			{copied}
			allPermissionsSelected={allPermissionsSelected(issuePermissions)}
			onOpenIssue={() => {
				issueOpen = true;
				revealToken = null;
			}}
			onCloseIssue={closeIssueForm}
			onIssueName={(name) => (issueName = name)}
			onSetIssueResourceAccess={setIssueResourceAccess}
			onToggleIssuePermission={toggleIssuePermission}
			onToggleIssueAllPermissions={toggleIssueAllPermissions}
			onIssueExpiry={(expiry) => (issueExpiry = expiry)}
			onSubmitIssueToken={submitIssueToken}
			onCopyToken={copyTokenToClipboard}
			onDismissToken={() => {
				revealToken = null;
				copied = false;
			}}
			onRevokeToken={revoke}
		/>

		<WebhookPanel
			{endpoints}
			{endpointCount}
			{expandedEndpoint}
			{deliveriesByEndpoint}
			{testEventByEndpoint}
			{addOpen}
			{addName}
			{addUrl}
			{addEvents}
			{addActive}
			{creatingEndpoint}
			{addError}
			{revealWebhookSecret}
			{webhookSecretCopied}
			onOpenAdd={() => {
				addOpen = true;
				addError = null;
			}}
			onCloseAdd={() => (addOpen = false)}
			onAddName={(name) => (addName = name)}
			onAddUrl={(url) => (addUrl = url)}
			onToggleEvent={toggleEvent}
			onToggleGroup={toggleGroupAll}
			onAddActive={(active) => (addActive = active)}
			onCreateEndpoint={submitCreateEndpoint}
			onToggleExpanded={toggleExpanded}
			onRotateSecret={rotateSecret}
			onSendTest={sendTest}
			onToggleActive={toggleEndpointActive}
			onDelete={removeEndpoint}
			onSetTestEvent={setTestEvent}
			onCopyWebhookSecret={copyWebhookSecretToClipboard}
			onDismissWebhookSecret={() => {
				revealWebhookSecret = null;
				webhookSecretCopied = false;
			}}
		/>

		<SavePill
			isDirty={isDirty && !showSaved}
			{saving}
			{showSaved}
			onSave={onSavePill}
			onDiscard={onDiscardPill}
		/>
	</div>
</div>

<style>
	.page {
		display: flex;
		flex-direction: column;
	}

	.settings-body {
		padding: 36px 56px 16px;
		flex: 1;
		display: flex;
		flex-direction: column;
		max-width: 920px;
		width: 100%;
		align-self: center;
		margin: 0 auto;
	}

	@media (max-width: 720px) {
		.settings-body {
			padding: 24px 20px 16px;
		}
	}
</style>
