import type { MessageKey } from '$lib/i18n';

export type IntegrationCapability = 'sync' | 'oauth' | 'pat' | 'importUpload';

export type IntegrationProviderId = 'obsidian' | 'notion' | 'readwise';

export interface IntegrationProvider {
	id: IntegrationProviderId;
	displayName: string;
	descriptionKey: MessageKey;
	/** Longer import-specific description used on the imports landing card (matches prototype copy). */
	importDescriptionKey?: MessageKey;
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
		descriptionKey: 'integrations_provider_obsidian_description',
		capabilities: ['sync', 'pat']
	},
	{
		id: 'notion',
		displayName: 'Notion',
		descriptionKey: 'integrations_provider_notion_description',
		capabilities: ['sync', 'oauth']
	},
	{
		id: 'readwise',
		displayName: 'Readwise Reader',
		descriptionKey: 'integrations_provider_readwise_description',
		importDescriptionKey: 'integrations_provider_readwise_import_description',
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
