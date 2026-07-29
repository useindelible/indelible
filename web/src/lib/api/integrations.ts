import {
	authorizeIntegration,
	deleteIntegration,
	getObsidianSettings,
	getNotionSettings,
	listIntegrations,
	listNotionExportItems,
	previewObsidianExport,
	refreshNotionExportItem,
	setupObsidianConnection,
	syncIntegration,
	updateObsidianSettings,
	updateNotionExportItems,
	updateNotionSettings,
	type AuthorizeIntegrationResponse,
	type IntegrationConnectionDto,
	type IntegrationListResponse,
	type NotionExportItemsResponse,
	type NotionRefreshItemResponse,
	type NotionSettingsDto,
	type ObsidianPreviewRequest,
	type ObsidianPreviewResponse,
	type ObsidianSettingsDto,
	type UpdateNotionExportItemsRequest,
	type UpdateNotionSettingsRequest,
	type UpdateObsidianSettingsRequest,
	type SyncIntegrationResponse
} from '$lib/api';

type ApiProblem = {
	detail?: string;
	error?: string;
	message?: string;
};

export type ApiResult<T> = { success: true; data: T } | { success: false; error: string };

function extractMessage(problem: unknown, fallback: string): string {
	if (!problem || typeof problem !== 'object') {
		return fallback;
	}
	const candidate = problem as ApiProblem;
	return candidate.detail ?? candidate.message ?? candidate.error ?? fallback;
}

function failure<T>(err: unknown, action: string): ApiResult<T> {
	console.error(`[integrations api] ${action}`, err);
	if (err && typeof err === 'object' && 'message' in err && typeof err.message === 'string') {
		return { success: false, error: err.message };
	}
	return { success: false, error: `An unexpected error occurred while ${action}.` };
}

export async function loadIntegrationConnections(): Promise<ApiResult<IntegrationListResponse>> {
	try {
		const { data, error } = await listIntegrations();
		if (data) {
			return { success: true, data };
		}
		return { success: false, error: extractMessage(error, 'Failed to load integrations') };
	} catch (err) {
		return failure(err, 'loading integrations');
	}
}

export async function startIntegrationAuthorization(
	provider: string,
	redirectAfter?: string
): Promise<ApiResult<AuthorizeIntegrationResponse>> {
	try {
		const { data, error } = await authorizeIntegration({
			path: { provider },
			body: { redirect_after: redirectAfter ?? null }
		});
		if (data) {
			return { success: true, data };
		}
		return {
			success: false,
			error: extractMessage(error, `Failed to start ${provider} authorization`)
		};
	} catch (err) {
		return failure(err, `starting ${provider} authorization`);
	}
}

export async function dispatchIntegrationSync(
	connectionId: string
): Promise<ApiResult<SyncIntegrationResponse>> {
	try {
		const { data, error } = await syncIntegration({ path: { id: connectionId } });
		if (data) {
			return { success: true, data };
		}
		return { success: false, error: extractMessage(error, 'Failed to start sync') };
	} catch (err) {
		return failure(err, 'starting integration sync');
	}
}

export async function disconnectIntegration(connectionId: string): Promise<ApiResult<void>> {
	try {
		const { error, response } = await deleteIntegration({ path: { id: connectionId } });
		if (response?.ok) {
			return { success: true, data: undefined };
		}
		return { success: false, error: extractMessage(error, 'Failed to disconnect integration') };
	} catch (err) {
		return failure(err, 'disconnecting integration');
	}
}

export async function loadNotionSettings(
	connectionId: string
): Promise<ApiResult<NotionSettingsDto>> {
	try {
		const { data, error } = await getNotionSettings({ path: { id: connectionId } });
		if (data) {
			return { success: true, data };
		}
		return { success: false, error: extractMessage(error, 'Failed to load Notion settings') };
	} catch (err) {
		return failure(err, 'loading Notion settings');
	}
}

export async function saveNotionSettings(
	connectionId: string,
	body: UpdateNotionSettingsRequest
): Promise<ApiResult<NotionSettingsDto>> {
	try {
		const { data, error } = await updateNotionSettings({ path: { id: connectionId }, body });
		if (data) {
			return { success: true, data };
		}
		return { success: false, error: extractMessage(error, 'Failed to update Notion settings') };
	} catch (err) {
		return failure(err, 'updating Notion settings');
	}
}

export async function loadNotionExportItems(
	connectionId: string,
	query: { q?: string | null; limit?: number; offset?: number } = {}
): Promise<ApiResult<NotionExportItemsResponse>> {
	try {
		const { data, error } = await listNotionExportItems({
			path: { id: connectionId },
			query
		});
		if (data) {
			return { success: true, data };
		}
		return { success: false, error: extractMessage(error, 'Failed to load Notion export items') };
	} catch (err) {
		return failure(err, 'loading Notion export items');
	}
}

export async function saveNotionExportItems(
	connectionId: string,
	body: UpdateNotionExportItemsRequest
): Promise<ApiResult<void>> {
	try {
		const { error, response } = await updateNotionExportItems({ path: { id: connectionId }, body });
		if (response?.ok) {
			return { success: true, data: undefined };
		}
		return { success: false, error: extractMessage(error, 'Failed to update export selection') };
	} catch (err) {
		return failure(err, 'updating Notion export selection');
	}
}

export async function refreshNotionDocumentExport(
	connectionId: string,
	libraryEntryId: string
): Promise<ApiResult<NotionRefreshItemResponse>> {
	try {
		const { data, error } = await refreshNotionExportItem({
			path: { id: connectionId, library_entry_id: libraryEntryId }
		});
		if (data) {
			return { success: true, data };
		}
		return { success: false, error: extractMessage(error, 'Failed to refresh Notion document') };
	} catch (err) {
		return failure(err, 'refreshing Notion document');
	}
}

export async function loadObsidianSettings(
	connectionId: string
): Promise<ApiResult<ObsidianSettingsDto>> {
	try {
		const { data, error } = await getObsidianSettings({ path: { id: connectionId } });
		if (data) {
			return { success: true, data };
		}
		return { success: false, error: extractMessage(error, 'Failed to load Obsidian settings') };
	} catch (err) {
		return failure(err, 'loading Obsidian settings');
	}
}

export async function saveObsidianSettings(
	connectionId: string,
	body: UpdateObsidianSettingsRequest
): Promise<ApiResult<ObsidianSettingsDto>> {
	try {
		const { data, error } = await updateObsidianSettings({ path: { id: connectionId }, body });
		if (data) {
			return { success: true, data };
		}
		return { success: false, error: extractMessage(error, 'Failed to update Obsidian settings') };
	} catch (err) {
		return failure(err, 'updating Obsidian settings');
	}
}

export async function previewObsidianSettings(
	connectionId: string,
	body: ObsidianPreviewRequest
): Promise<ApiResult<ObsidianPreviewResponse>> {
	try {
		const { data, error } = await previewObsidianExport({ path: { id: connectionId }, body });
		if (data) {
			return { success: true, data };
		}
		return { success: false, error: extractMessage(error, 'Failed to render Obsidian preview') };
	} catch (err) {
		return failure(err, 'rendering Obsidian preview');
	}
}

export async function setupObsidianExportConnection(): Promise<
	ApiResult<IntegrationConnectionDto>
> {
	try {
		const { data, error } = await setupObsidianConnection();
		if (data) {
			return { success: true, data };
		}
		return {
			success: false,
			error: extractMessage(error, 'Failed to set up Obsidian export')
		};
	} catch (err) {
		return failure(err, 'setting up Obsidian export');
	}
}
