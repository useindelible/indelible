const catalogs = import.meta.glob('./locales/*.json');

export const SUPPORTED_LOCALES: readonly string[] = Object.keys(catalogs)
	.map((path) => path.slice('./locales/'.length, -'.json'.length))
	.sort();
export const FALLBACK_LOCALE = 'en';

const RTL_LANGUAGES = new Set(['ar', 'he', 'fa', 'ur', 'ps', 'sd', 'ug', 'yi', 'dv']);

export function matchLocale(
	tag: string | null | undefined,
	supported: readonly string[] = SUPPORTED_LOCALES
): string | null {
	if (!tag) return null;

	let locale: Intl.Locale;
	try {
		locale = new Intl.Locale(tag.trim());
	} catch {
		return null;
	}

	const full = locale.toString().toLowerCase();
	const language = locale.language.toLowerCase();
	return (
		supported.find((candidate) => candidate.toLowerCase() === full) ??
		supported.find((candidate) => candidate.toLowerCase() === language) ??
		null
	);
}

export function systemLocale(
	navigatorLanguages: readonly string[] = typeof navigator === 'undefined'
		? []
		: navigator.languages
): string {
	return (
		navigatorLanguages
			.map((tag) => matchLocale(tag))
			.find((match): match is string => match !== null) ?? FALLBACK_LOCALE
	);
}

export function resolveInitialLocale(input: {
	storedLocale?: string | null;
	navigatorLanguages?: readonly string[];
}): string {
	return matchLocale(input.storedLocale) ?? systemLocale(input.navigatorLanguages);
}

export function isRtl(tag: string): boolean {
	try {
		return RTL_LANGUAGES.has(new Intl.Locale(tag).language);
	} catch {
		return false;
	}
}

export function localeDisplayName(tag: string): string {
	try {
		const name = new Intl.DisplayNames([tag], { type: 'language' }).of(tag) ?? tag;
		return name.charAt(0).toLocaleUpperCase(tag) + name.slice(1);
	} catch {
		return tag;
	}
}
