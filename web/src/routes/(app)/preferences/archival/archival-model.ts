import type {
	ArchivalSettingsResponse,
	DuplicateActionDto,
	DuplicateSensitivityDto
} from '$lib/api';
import type { MessageKey } from '$lib/i18n';

export type FormatId = 'readable' | 'monolith' | 'pdf' | 'screenshot' | 'warc';
export type FormatStatus = 'on' | 'off' | 'coming';
export type ArchiveFormatToggleId = 'monolith' | 'pdf' | 'screenshot';
export type DuplicateSensitivity = 1 | 2 | 3;
export type DuplicateAction = 'notify' | 'skip' | 'merge';

export interface ArchiveFormat {
	id: FormatId;
	labelKey: MessageKey;
	descKey: MessageKey;
	size: string;
	alwaysOn?: boolean;
	comingSoon?: boolean;
}

export interface ArchivalSnapshotInput {
	formats: Record<ArchiveFormatToggleId, boolean>;
	dupEnabled: boolean;
	dupSensitivity: DuplicateSensitivity;
	dupAction: DuplicateAction;
	proxyUrl: string;
	proxyAll: boolean;
}

export interface BuildArchivalSettingsInput extends ArchivalSnapshotInput {
	serverData: ArchivalSettingsResponse | null;
}

export const ARCHIVE_FORMATS: ArchiveFormat[] = [
	{
		id: 'readable',
		labelKey: 'archival_format_readable',
		descKey: 'archival_format_readable_description',
		size: '~30 KB',
		alwaysOn: true
	},
	{
		id: 'monolith',
		labelKey: 'archival_format_monolith',
		descKey: 'archival_format_monolith_description',
		size: '~1.2 MB'
	},
	{
		id: 'pdf',
		labelKey: 'archival_format_pdf',
		descKey: 'archival_format_pdf_description',
		size: '~600 KB'
	},
	{
		id: 'screenshot',
		labelKey: 'archival_format_screenshot',
		descKey: 'archival_format_screenshot_description',
		size: '~2.4 MB'
	},
	{
		id: 'warc',
		labelKey: 'archival_format_warc',
		descKey: 'archival_format_warc_description',
		size: '— —',
		comingSoon: true
	}
];

export const DEFAULT_ARCHIVAL_SETTINGS = {
	formats: { monolith: true, pdf: false, screenshot: true } as Record<
		ArchiveFormatToggleId,
		boolean
	>,
	dupEnabled: true,
	dupSensitivity: 2 as DuplicateSensitivity,
	dupAction: 'notify' as DuplicateAction,
	proxyUrl: '',
	proxyAll: false
};

export function sensitivityFromApi(value: DuplicateSensitivityDto): DuplicateSensitivity {
	if (value === 'low') return 1;
	if (value === 'high') return 3;
	return 2;
}

export function sensitivityToApi(value: DuplicateSensitivity): DuplicateSensitivityDto {
	if (value === 1) return 'low';
	if (value === 3) return 'high';
	return 'medium';
}

export function actionFromApi(value: DuplicateActionDto): DuplicateAction {
	if (value === 'skip_silently') return 'skip';
	if (value === 'merge_with_existing') return 'merge';
	return 'notify';
}

export function actionToApi(value: DuplicateAction): DuplicateActionDto {
	if (value === 'skip') return 'skip_silently';
	if (value === 'merge') return 'merge_with_existing';
	return 'notify_me';
}

export function createArchivalSnapshot(input: ArchivalSnapshotInput): string {
	return JSON.stringify({
		formats: input.formats,
		dupEnabled: input.dupEnabled,
		dupSensitivity: input.dupSensitivity,
		dupAction: input.dupAction,
		proxyUrl: input.proxyUrl.trim(),
		proxyAll: input.proxyAll
	});
}

export function getArchiveFormat(id: FormatId): ArchiveFormat | undefined {
	return ARCHIVE_FORMATS.find((format) => format.id === id);
}

export function isArchiveFormatOn(
	id: FormatId,
	formats: Record<ArchiveFormatToggleId, boolean>
): boolean {
	const format = getArchiveFormat(id);
	if (!format) return false;
	if (format.alwaysOn) return true;
	if (format.comingSoon) return false;
	return formats[id as ArchiveFormatToggleId];
}

export function getArchiveFormatStatus(
	id: FormatId,
	formats: Record<ArchiveFormatToggleId, boolean>
): FormatStatus {
	const format = getArchiveFormat(id);
	if (!format) return 'off';
	if (format.alwaysOn) return 'on';
	if (format.comingSoon) return 'coming';
	return formats[id as ArchiveFormatToggleId] ? 'on' : 'off';
}

export function canToggleArchiveFormat(id: FormatId): id is ArchiveFormatToggleId {
	const format = getArchiveFormat(id);
	return !!format && !format.alwaysOn && !format.comingSoon;
}

export function buildArchivalSettingsPayload(
	input: BuildArchivalSettingsInput
): ArchivalSettingsResponse {
	const trimmedProxyUrl = input.proxyUrl.trim();

	return {
		archive_formats: {
			readable_html: true,
			monolith: input.formats.monolith,
			pdf: input.formats.pdf,
			screenshot: input.formats.screenshot,
			warc: input.serverData?.archive_formats.warc ?? false
		},
		duplicate_detection: {
			enabled: input.dupEnabled,
			sensitivity: sensitivityToApi(input.dupSensitivity),
			on_duplicate: actionToApi(input.dupAction)
		},
		processing: input.serverData?.processing ?? {
			browser_timeout_secs: 90,
			max_concurrent_archives: 3,
			ai_auto_processing: false
		},
		proxy: {
			url: trimmedProxyUrl || null,
			all_requests: trimmedProxyUrl.length > 0 && input.proxyAll
		}
	};
}
