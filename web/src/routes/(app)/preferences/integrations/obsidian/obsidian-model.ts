import type {
	IntegrationConnectionDto,
	ObsidianPreviewResponse,
	ObsidianSettingsDto,
	UpdateObsidianSettingsRequest
} from '$lib/api';
import type { ConnectionState } from '$lib/integrations/status';
import type { MessageKey, Translate } from '$lib/i18n';
import { relativeTime } from '$lib/utils/relative-time';

export type PreviewView = 'note' | 'full';
export type ObsidianHeroState = 'connected' | 'syncing' | 'error' | 'disconnected';

export function serializeForCompare(settings: ObsidianSettingsDto) {
	return {
		group_files_in_category_folders: settings.group_files_in_category_folders,
		export_all_reader_documents: settings.export_all_reader_documents,
		sync_notifications: settings.sync_notifications,
		properties_template: settings.properties_template ?? '',
		page_title_template: settings.page_title_template,
		metadata_template: settings.metadata_template,
		highlight_header_template: settings.highlight_header_template,
		highlight_template: settings.highlight_template,
		file_name_template: settings.file_name_template ?? '',
		category_folder_templates: { ...settings.category_folder_templates },
		sync_notification_template: settings.sync_notification_template
	};
}

export function snapshotObsidianSettings(settings: ObsidianSettingsDto): ObsidianSettingsDto {
	return {
		...settings,
		category_folder_templates: { ...settings.category_folder_templates }
	};
}

export function buildObsidianSaveBody(
	settings: ObsidianSettingsDto
): UpdateObsidianSettingsRequest {
	return {
		group_files_in_category_folders: settings.group_files_in_category_folders,
		export_all_reader_documents: settings.export_all_reader_documents,
		sync_notifications: settings.sync_notifications,
		properties_template: blankToNull(settings.properties_template),
		page_title_template: settings.page_title_template,
		metadata_template: settings.metadata_template,
		highlight_header_template: settings.highlight_header_template,
		highlight_template: settings.highlight_template,
		file_name_template: blankToNull(settings.file_name_template),
		category_folder_templates: settings.category_folder_templates,
		sync_notification_template: settings.sync_notification_template
	};
}

export function blankToNull(value: string | null | undefined): string | null {
	const trimmed = value?.trim();
	return trimmed ? (value ?? null) : null;
}

export function obsidianHeroState(
	connection: IntegrationConnectionDto | undefined,
	connectionState: ConnectionState
): ObsidianHeroState {
	if (!connection) return 'disconnected';
	switch (connectionState) {
		case 'syncing':
			return 'syncing';
		case 'failed':
			return 'error';
		case 'disconnected':
		case 'unavailable':
			return 'disconnected';
		default:
			return 'connected';
	}
}

export function obsidianHeroStatusLabel(state: ObsidianHeroState): MessageKey {
	switch (state) {
		case 'syncing':
			return 'integrations_obsidian_status_syncing';
		case 'error':
			return 'integrations_obsidian_last_sync_failed';
		case 'disconnected':
			return 'integrations_obsidian_no_plugin_connected';
		default:
			return 'integrations_hub_status_connected';
	}
}

export function formatObsidianLastSync(
	connection: IntegrationConnectionDto | undefined,
	translate: Translate
): string {
	return relativeTime(connection?.last_sync_at) ?? translate('integrations_obsidian_never');
}

export function previewFilePath(
	preview: ObsidianPreviewResponse | null,
	previewView: PreviewView
): string {
	if (!preview) return '';
	return previewView === 'full' && preview.full_document_text_path
		? preview.full_document_text_path
		: preview.file_path;
}

export function previewBody(
	preview: ObsidianPreviewResponse | null,
	previewView: PreviewView
): string {
	if (!preview) return '';
	if (previewView === 'full') return preview.full_document_text ?? '';
	return preview.full_content;
}

export function previewMissingSummary(
	preview: ObsidianPreviewResponse | null,
	previewView: PreviewView
): boolean {
	if (!preview || previewView !== 'note') return false;
	return !preview.full_content.includes('Summary:');
}
