import './client';

export { api, AUTH_REDIRECT_SUPPRESSION_HEADER, getApiBaseUrl } from './client';
export { itemTypeCounts, trashCount } from './compat-counts';
export type {
	DocumentAssetListResponse,
	DocumentListEntry,
	DocumentUpdateBody,
	LibraryQueryBody,
	LibraryTriageRequest
} from './compat-types';
export * from './extension-auth';
export * from './mila';
export {
	addEntryToCollection,
	authorizeIntegration,
	changeEmail,
	changePassword,
	clearRecentSearches,
	completeStep,
	createCollection,
	createEmailAlias,
	createPromptPreset,
	createSmartList,
	createTag,
	createToken,
	createWebhookEndpoint,
	deleteAccount,
	deleteCollection,
	deleteHighlight,
	deleteIntegration,
	deleteNote,
	deletePromptPreset,
	deleteRecentSearch,
	deleteSmartList,
	deleteTag,
	deleteWebhookEndpoint,
	forgotPassword,
	getArchival,
	getCollection,
	getConfig,
	getDocumentPlaybackState,
	getEntity,
	getHome,
	getImport,
	getNotifications,
	getNotionSettings,
	getObsidianSettings,
	getOnboarding,
	getPreferences,
	getProfile,
	getSmartList,
	getStatus,
	getTag,
	listChildren,
	listEmailAliases,
	listEmailSenders,
	listEntityDocuments,
	listFeedDeliveries,
	listImports,
	listIntegrations,
	listCollections,
	listNotionExportItems,
	listPersonas,
	listPromptPresets,
	listProviders,
	listRecentHighlights,
	listRecentSearches,
	listSmartLists,
	listSubscriptions,
	listTagHighlights,
	listTags,
	listTokens,
	listWebhookDeliveries,
	listWebhookEndpoints,
	login,
	logout,
	markAllDeliveriesSeen,
	markDeliverySeen,
	mergeTags,
	patchHighlight,
	pinSmartList,
	prepareFeedDelivery,
	previewObsidianExport,
	refresh,
	refreshNotionExportItem,
	register,
	reindexConfig,
	removeEntryFromCollection,
	resetPassword,
	retryMilaDocumentAction,
	resolveDocumentTtsTimestamp,
	resendVerification,
	retrySubscription,
	revokeToken,
	rollbackImport,
	rotateWebhookSecret,
	saveFromDelivery,
	search,
	searchSources,
	setHighlightTags,
	setupObsidianConnection,
	skipOnboarding,
	startDocumentTtsSession,
	streamEvents,
	subscribe,
	suggestions,
	syncIntegration,
	testConfig,
	testWebhookEndpoint,
	unsubscribe,
	unsubscribeEmailSender,
	updateArchival,
	updateCollection,
	updateEmailSender,
	updateNotifications,
	updateNotionExportItems,
	updateNotionSettings,
	updateObsidianSettings,
	updatePreferences,
	updateProfile,
	updatePromptPreset,
	updateSmartList,
	updateSubscription,
	updateTag,
	updateWebhookEndpoint,
	uploadAvatar,
	upsertConfig,
	upsertDocumentPlaybackState,
	upsertNote,
	verifyEmail
} from './generated/sdk.gen';
export type {
	AccentColorDto,
	AliasDestinationDto,
	ApiPermissionDto,
	ApiTokenResponse,
	ArchivalSettingsResponse,
	AuthorizeIntegrationResponse,
	CollectionResponse,
	CreateApiTokenRequest,
	CreateApiTokenResponse,
	CreateWebhookEndpointRequest,
	DefaultViewDto,
	DestinationDto,
	DocumentNoteResponse,
	DocumentReaderAssetResponse,
	DuplicateActionDto,
	DuplicateSensitivityDto,
	EmailAliasResponse,
	EmailSenderResponse,
	EntityDetailResponse,
	EntityDocumentResponse,
	EpubTocResponse,
	FeedDeliveryResponse,
	FeedSourceResponse,
	FeedSubscriptionResponse,
	HighlightResponse,
	HighlightWithNoteResponse,
	HomeItemResponse,
	ImportJobStatusResponse,
	ImportUploadResponse,
	IntegrationConnectionDto,
	IntegrationListResponse,
	ListDensityDto,
	MilaConfigResponse,
	MilaPromptPresetResponse,
	MilaPromptPresetsResponse,
	MilaStatusResponse,
	NotificationsSettingsResponse,
	NotionExportItemDto,
	NotionExportItemsResponse,
	NotionRefreshItemResponse,
	NotionSettingsDto,
	OAuthProviderInfo,
	ObsidianPreviewRequest,
	ObsidianPreviewResponse,
	ObsidianSettingsDto,
	OnboardingStepResponse,
	OpmlImportResponse,
	PreferencesSettingsResponse,
	ProfileResponse,
	ReaderFontFamilyDto,
	ReaderFontSizeDto,
	ReaderLineHeightDto,
	ReaderOpenModeDto,
	RealtimeEventResponse,
	RenderDefaultDto,
	RequiredNullableDuration,
	SearchEmbeddedSenderResponse,
	SidePanelModeDto,
	SidebarModeDto,
	SmartListResponse,
	StepData,
	SyncIntegrationResponse,
	TagResponse,
	TestMilaConfigBodyWritable,
	ThemeDto,
	TriageModeDto,
	UpdateNotionExportItemsRequest,
	UpdateNotionSettingsRequest,
	UpdateObsidianSettingsRequest,
	UpdateWebhookEndpointRequest,
	UpsertMilaConfigBodyWritable,
	WebhookDeliveryResponse,
	WebhookEndpointResponse,
	WebhookEndpointSecretResponse
} from './generated/types.gen';

import * as generated from './generated';
import { documentByLibraryEntry, libraryEntryByDocument } from './document-ids';

export * from './documents';
import type {
	DocumentListEntry,
	DocumentUpdateBody,
	LibraryQueryBody,
	LibraryTriageRequest
} from './compat-types';
import type {
	DocumentReaderResponse,
	LibraryEntryResponse,
	LibraryEntryTagsResponse,
	SaveUrlBody
} from './generated/types.gen';

function rememberEntry(entry: LibraryEntryResponse): void {
	libraryEntryByDocument.set(entry.document_id, entry.library_entry_id);
	documentByLibraryEntry.set(entry.library_entry_id, entry.document_id);
}

function toDocumentListEntry(entry: LibraryEntryResponse): DocumentListEntry {
	rememberEntry(entry);
	const failed = entry.ingest_failure_reason != null;
	return {
		...entry,
		id: entry.document_id,
		item_type: entry.document_type,
		readable_ready: !failed,
		pipeline_status: failed ? 'failed' : undefined,
		pipeline_error: entry.ingest_failure_reason ?? undefined,
		saved: true
	};
}

function toDocumentListEntryPage(page: generated.PaginatedResponseLibraryEntryResponse) {
	return {
		...page,
		data: page.data.map(toDocumentListEntry)
	};
}

function toDocumentListEntryFromReader(reader: DocumentReaderResponse): DocumentListEntry {
	const now = new Date().toISOString();
	return {
		author: reader.author,
		created_at: now,
		document_id: reader.document_id,
		document_type: reader.document_type,
		domain: reader.domain,
		excerpt: reader.excerpt,
		id: reader.document_id,
		is_favorite: false,
		is_shortlisted: false,
		item_type: reader.document_type,
		language: reader.language,
		lead_image_url: reader.lead_image_url,
		thumbnail_url: reader.thumbnail_url,
		chapter_locator: reader.chapter_locator,
		chapter_offset: reader.chapter_offset,
		library_entry_id: reader.library_entry_id ?? null,
		last_read_at: reader.last_read_at,
		max_progress_percent: reader.max_progress_percent,
		object: 'library_entry',
		progress_percent: reader.progress_percent,
		published_at: reader.published_at,
		readable_ready: reader.readable_ready,
		available_assets: reader.available_assets,
		saved: reader.saved,
		saved_at: now,
		source: 'document',
		summary: reader.summary,
		title: reader.title,
		triage_state: 'later',
		updated_at: now,
		url: reader.url,
		word_count: reader.word_count,
		reading_time_minutes: reader.reading_time_minutes
	};
}

async function resolveLibraryEntryId(id: string): Promise<string> {
	if (id.startsWith('lib_')) return id;
	const cached = libraryEntryByDocument.get(id);
	if (cached) return cached;
	const { data } = await generated.getDocumentReader({ path: { document_id: id } });
	const entryId = data?.library_entry_id;
	if (!entryId) throw new Error('Document is not saved in the Library');
	libraryEntryByDocument.set(id, entryId);
	documentByLibraryEntry.set(entryId, id);
	return entryId;
}

export async function queryLibraryEntries(options: { body: LibraryQueryBody }) {
	const { data, error } = await generated.queryLibrary({ body: options.body });
	return { data: data ? toDocumentListEntryPage(data) : undefined, error };
}

export async function listLibraryEntries(options?: {
	query?: { cursor?: string | null; limit?: number | null };
}) {
	const { data } = await generated.listLibrary({ query: options?.query });
	return { data: data ? toDocumentListEntryPage(data) : undefined };
}

export async function listCollectionEntries(options: {
	path: { id: string };
	query?: { cursor?: string | null; limit?: number | null };
}) {
	const { data } = await generated.listCollectionEntries(options);
	return { data: data ? toDocumentListEntryPage(data) : undefined };
}

export async function listTagEntries(options: {
	path: { id: string };
	query?: { cursor?: string | null; limit?: number | null; scope?: string | null };
}) {
	const { data } = await generated.listTagEntries(options);
	return { data: data ? toDocumentListEntryPage(data) : undefined };
}

export type GetDocumentEntryResult = {
	data: DocumentListEntry | undefined;
	/** HTTP status of the underlying fetch; undefined on a network-level error. */
	status?: number;
};

// Concurrent callers (realtime event handlers, reactive panels) routinely request
// the same document at once. Without deduplication a burst of events fans out into
// dozens of identical fetches per document, each costing 1-2 backend requests and
// quickly tripping the server's per-user rate limit. Share one in-flight promise per
// document id; the entry clears once it settles so later state changes refetch.
const inFlightDocumentEntries = new Map<string, Promise<GetDocumentEntryResult>>();

export function getDocumentEntry(options: {
	path: { document_id: string };
	query?: { include?: string | null };
}): Promise<GetDocumentEntryResult> {
	const id = options.path.document_id;
	const existing = inFlightDocumentEntries.get(id);
	if (existing) return existing;
	const pending = fetchDocumentEntry(id).finally(() => {
		inFlightDocumentEntries.delete(id);
	});
	inFlightDocumentEntries.set(id, pending);
	return pending;
}

async function fetchDocumentEntry(id: string): Promise<GetDocumentEntryResult> {
	if (id.startsWith('lib_')) {
		const { data, response } = await generated.getLibraryEntry({
			path: { library_entry_id: id }
		});
		return { data: data ? toDocumentListEntry(data) : undefined, status: response?.status };
	}
	const { data: reader, response } = await generated.getDocumentReader({
		path: { document_id: id }
	});
	if (!reader) return { data: undefined, status: response?.status };
	if (reader.library_entry_id) {
		const { data: entry } = await generated.getLibraryEntry({
			path: { library_entry_id: reader.library_entry_id }
		});
		if (entry) {
			return {
				data: {
					...toDocumentListEntry(entry),
					chapter_locator: reader.chapter_locator,
					chapter_offset: reader.chapter_offset,
					progress_percent: reader.progress_percent,
					max_progress_percent: reader.max_progress_percent,
					last_read_at: reader.last_read_at,
					summary: entry.summary ?? reader.summary
				},
				status: response?.status
			};
		}
		return { data: toDocumentListEntryFromReader(reader), status: response?.status };
	}
	return { data: toDocumentListEntryFromReader(reader), status: response?.status };
}

export async function toggleFavorite(options: { path: { document_id: string } }) {
	const library_entry_id = await resolveLibraryEntryId(options.path.document_id);
	const { data } = await generated.toggleLibraryFavorite({ path: { library_entry_id } });
	return { data: data ? toDocumentListEntry(data) : undefined };
}

export async function triageLibraryEntry(options: {
	path: { document_id: string };
	body: LibraryTriageRequest;
}) {
	const library_entry_id = await resolveLibraryEntryId(options.path.document_id);
	const { data } = await generated.triageEntry({
		path: { library_entry_id },
		body: { triage_state: options.body.state },
		throwOnError: true
	});
	return { data: data ? toDocumentListEntry(data) : undefined };
}

export async function listTrash(options?: {
	query?: { cursor?: string | null; limit?: number | null };
}) {
	const { data } = await generated.listLibraryTrash({ query: options?.query });
	return { data: data ? toDocumentListEntryPage(data) : undefined };
}

export async function restoreLibraryEntry(options: { path: { document_id: string } }) {
	const library_entry_id = await resolveLibraryEntryId(options.path.document_id);
	const { data } = await generated.restoreEntry({ path: { library_entry_id } });
	return { data: data ? toDocumentListEntry(data) : undefined };
}

export async function purgeLibraryEntry(options: { path: { document_id: string } }) {
	const library_entry_id = await resolveLibraryEntryId(options.path.document_id);
	return generated.purgeEntry({ path: { library_entry_id } });
}

export async function deleteLibraryEntry(options: { path: { document_id: string } }) {
	const library_entry_id = await resolveLibraryEntryId(options.path.document_id);
	return generated.deleteLibraryEntry({ path: { library_entry_id }, throwOnError: true });
}

export async function emptyTrash() {
	const { data } = await generated.listLibraryTrash({ query: { limit: 100 } });
	await Promise.allSettled(
		(data?.data ?? []).map((entry) =>
			generated.purgeEntry({ path: { library_entry_id: entry.library_entry_id } })
		)
	);
	return { data: undefined };
}

export async function getLibraryEntryTags(options: { path: { library_entry_id: string } }) {
	const { data } = await generated.getEntryTags({ path: options.path });
	return { data: (data ?? { tags: [] }) as LibraryEntryTagsResponse };
}

export async function replaceLibraryEntryTags(options: {
	path: { library_entry_id: string };
	body: { tags: string[] };
}) {
	const { data } = await generated.setEntryTags({ path: options.path, body: options.body });
	return { data: (data ?? { tags: [] }) as LibraryEntryTagsResponse };
}

export async function updateDocumentEntry(options: {
	path: { document_id: string };
	body: DocumentUpdateBody;
}) {
	return getDocumentEntry({ path: options.path });
}

export async function createDocumentEntry(options: { body: SaveUrlBody & { source?: string } }) {
	const { source, ...body } = options.body;
	void source;
	const { data, error, response } = await generated.saveUrl({
		body,
		throwOnError: false
	});
	return {
		data: data ? toDocumentListEntry(data) : undefined,
		error,
		response: response ?? new Response(null, { status: data ? 200 : 500 })
	};
}
