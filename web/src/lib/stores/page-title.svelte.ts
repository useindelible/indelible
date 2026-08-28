import type { MessageKey, Translate } from '$lib/i18n';

export const FALLBACK_TITLE_KEY: MessageKey = 'common_app_page_title';

type Rule = { pattern: RegExp; key: MessageKey };

// Ordered most-specific-first; the first match wins. Every pattern ends on a segment
// boundary so `/feed` cannot claim `/feedback`.
const ROUTE_TITLE_RULES: Rule[] = [
	{ pattern: /^\/reader(?:\/|$)/, key: 'reader_view_reader' },
	{ pattern: /^\/library\/articles(?:\/|$)/, key: 'library_nav_articles' },
	{ pattern: /^\/library\/books(?:\/|$)/, key: 'library_nav_books' },
	{ pattern: /^\/library\/emails(?:\/|$)/, key: 'library_nav_emails' },
	{ pattern: /^\/library\/pdfs(?:\/|$)/, key: 'library_nav_pdfs' },
	{ pattern: /^\/library\/tweets(?:\/|$)/, key: 'library_nav_tweets' },
	{ pattern: /^\/library\/videos(?:\/|$)/, key: 'library_nav_videos' },
	{ pattern: /^\/library(?:\/|$)/, key: 'common_library' },
	{ pattern: /^\/feed(?:\/|$)/, key: 'common_feed' },
	{ pattern: /^\/search(?:\/|$)/, key: 'common_search' },
	{ pattern: /^\/trash(?:\/|$)/, key: 'common_trash' },
	{ pattern: /^\/dashboard(?:\/|$)/, key: 'library_nav_home' },
	{ pattern: /^\/collections(?:\/|$)/, key: 'library_collections' },
	{ pattern: /^\/tags(?:\/|$)/, key: 'common_tags' },
	{ pattern: /^\/entities\/[^/]+(?:\/|$)/, key: 'entity_page_title' },
	{ pattern: /^\/onboarding(?:\/|$)/, key: 'onboarding_page_title' },
	{
		pattern: /^\/preferences\/integrations\/notion(?:\/|$)/,
		key: 'settings_notion_page_title'
	},
	{
		pattern: /^\/preferences\/integrations\/obsidian(?:\/|$)/,
		key: 'settings_obsidian_page_title'
	},
	{ pattern: /^\/preferences\/integrations(?:\/|$)/, key: 'settings_integrations' },
	{ pattern: /^\/preferences\/account(?:\/|$)/, key: 'settings_account' },
	{
		pattern: /^\/preferences\/reading-appearance(?:\/|$)/,
		key: 'settings_reading_appearance'
	},
	{ pattern: /^\/preferences\/ai(?:\/|$)/, key: 'settings_ai' },
	{ pattern: /^\/preferences\/archival(?:\/|$)/, key: 'settings_archival' },
	{ pattern: /^\/preferences\/developer(?:\/|$)/, key: 'settings_developer' },
	{ pattern: /^\/preferences\/email(?:\/|$)/, key: 'settings_email' },
	{
		pattern: /^\/preferences\/feed-management(?:\/|$)/,
		key: 'settings_feed_management'
	},
	{ pattern: /^\/preferences\/add-to-feed(?:\/|$)/, key: 'settings_add_to_feed' },
	{
		pattern: /^\/preferences\/add-to-library(?:\/|$)/,
		key: 'settings_add_to_library'
	},
	{ pattern: /^\/preferences\/import-export(?:\/|$)/, key: 'settings_import_export' },
	{ pattern: /^\/preferences(?:\/|$)/, key: 'settings_preferences' },
	{ pattern: /^\/login(?:\/|$)/, key: 'auth_sign_in_title' },
	{ pattern: /^\/register(?:\/|$)/, key: 'auth_create_account_title' },
	{ pattern: /^\/forgot-password(?:\/|$)/, key: 'auth_reset_password_title' },
	{ pattern: /^\/reset-password(?:\/|$)/, key: 'auth_set_new_password_title' },
	{ pattern: /^\/verify-email(?:\/|$)/, key: 'auth_verify_email_title' },
	{ pattern: /^\/auth\/callback(?:\/|$)/, key: 'auth_callback_page_title' },
	{ pattern: /^\/extension\/auth(?:\/|$)/, key: 'extension_auth_page_title' }
];

export function routeTitleKey(pathname: string): MessageKey {
	return ROUTE_TITLE_RULES.find((rule) => rule.pattern.test(pathname))?.key ?? FALLBACK_TITLE_KEY;
}

export function resolveTitle(input: {
	pathname: string;
	errorStatus: number | null;
	override: string | null;
	translate: Translate;
}): string {
	const { pathname, errorStatus, override, translate } = input;

	if (errorStatus !== null) {
		return translate(errorStatus === 404 ? 'error_page_not_found_title' : 'error_generic_title');
	}

	const name = override?.replace(/\s+/g, ' ').trim();
	if (name) return name;

	return translate(routeTitleKey(pathname));
}

let override = $state<(() => string | null) | null>(null);

/** Lets a route name the document after its own data. Clears itself on unmount. */
export function setDocumentTitle(getter: () => string | null): void {
	$effect(() => {
		override = getter;
		// Identity check: a late unmount must not clear a newer provider.
		return () => {
			if (override === getter) override = null;
		};
	});
}

export function readTitleOverride(): string | null {
	return override?.() ?? null;
}
