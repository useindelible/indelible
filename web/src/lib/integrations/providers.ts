/**
 * Static catalog of M10-wave providers. The backend `listIntegrations` only
 * returns the user's existing connections; rendering disconnected/unavailable
 * cards requires a frontend-owned list of supported providers. Provider-specific
 * tasks (TASK-200..TASK-209) layer on OAuth/PAT/import wiring.
 */

export type IntegrationCapability = 'sync' | 'oauth' | 'pat' | 'importUpload';

export type IntegrationProviderId = 'obsidian' | 'notion' | 'readwise';

export interface IntegrationProvider {
	id: IntegrationProviderId;
	displayName: string;
	description: string;
	/** Longer import-specific description used on the imports landing card (matches prototype copy). */
	importDescription?: string;
	capabilities: IntegrationCapability[];
	importSlug?: string;
	acceptedMimeTypes?: string[];
	acceptedExtensions?: string[];
	maxBytes?: number;
}

// Readwise exports bundle document text and can be large.
// Backend `max_import_upload_bytes` is 200 MB in production (see backend/apps/ind-api/src/config.rs).
const TWO_HUNDRED_MB = 200 * 1024 * 1024;

export const INTEGRATION_PROVIDERS: IntegrationProvider[] = [
	{
		id: 'obsidian',
		displayName: 'Obsidian',
		description: 'Sync highlights and notes with your vault via a scoped access token.',
		capabilities: ['sync', 'pat']
	},
	{
		id: 'notion',
		displayName: 'Notion',
		description: 'Sync highlights and annotations into a Notion database.',
		capabilities: ['sync', 'oauth']
	},
	{
		id: 'readwise',
		displayName: 'Readwise Reader',
		description: 'Import Reader CSV, uploaded-file ZIP archives, and feeds OPML from Readwise.',
		importDescription:
			"Upload any combination of CSV, document ZIP archive, and OPML. We'll match documents across files and route OPML subscriptions to your feeds. Highlights are not migrated.",
		capabilities: ['importUpload'],
		importSlug: 'readwise',
		acceptedMimeTypes: ['text/csv', 'application/zip', 'text/xml', 'application/xml'],
		acceptedExtensions: ['.csv', '.zip', '.opml', '.xml'],
		maxBytes: TWO_HUNDRED_MB
	}
];

export function findProvider(id: string): IntegrationProvider | undefined {
	return INTEGRATION_PROVIDERS.find((p) => p.id === id);
}
