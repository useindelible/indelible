/**
 * Theme selection.
 *
 * Two keys are in play. `indelible-theme` is ours and holds any theme id from
 * data/themes.ts. `starlight-theme` is Starlight's and only understands
 * 'light' or 'dark', so we mirror our choice onto it using the theme's
 * declared `base`. Writing only Starlight's key would make a third palette
 * impossible; writing only ours would let the docs disagree with the landing
 * page.
 */

import { THEMES, DEFAULT_THEME } from '../data/themes';

export const THEME_KEY = 'indelible-theme';
export const STARLIGHT_KEY = 'starlight-theme';

const IDS = THEMES.map((t) => t.id);

export function isThemeId(value: unknown): value is string {
	return typeof value === 'string' && IDS.includes(value);
}

export function themeDef(id: string) {
	return THEMES.find((t) => t.id === id) ?? THEMES[0];
}

export function systemTheme(): string {
	const prefersLight =
		typeof matchMedia === 'function' &&
		matchMedia('(prefers-color-scheme: light)').matches;
	const wanted = prefersLight ? 'light' : 'dark';
	return isThemeId(wanted) ? wanted : DEFAULT_THEME;
}

export function storedTheme(): string | null {
	try {
		const own = localStorage.getItem(THEME_KEY);
		if (isThemeId(own)) return own;

		// Fall back to Starlight's key so arriving from the docs keeps the look.
		const sl = localStorage.getItem(STARLIGHT_KEY);
		return isThemeId(sl) ? sl : null;
	} catch {
		// Private browsing and blocked storage both throw; prefer the system.
		return null;
	}
}

export function currentTheme(): string {
	const attr = document.documentElement.dataset.theme;
	if (isThemeId(attr)) return attr;
	return storedTheme() ?? systemTheme();
}

export function applyTheme(id: string): void {
	const def = themeDef(id);
	document.documentElement.dataset.theme = def.id;
	try {
		localStorage.setItem(THEME_KEY, def.id);
		localStorage.setItem(STARLIGHT_KEY, def.base);
	} catch {
		// Not persisting is survivable; the page still renders in `id`.
	}
}

/** Advance to the next theme in the list, wrapping. */
export function cycleTheme(): string {
	const i = IDS.indexOf(currentTheme());
	const next = IDS[(i + 1) % IDS.length];
	applyTheme(next);
	return next;
}

/**
 * Runs in <head> before first paint, inlined so there is no round trip and
 * therefore no flash of the wrong palette. Dependency-free and ES5-safe.
 */
export const PRE_PAINT_SNIPPET = `(function(){try{
  var ids=${JSON.stringify(IDS)},d=${JSON.stringify(DEFAULT_THEME)};
  var v=localStorage.getItem(${JSON.stringify(THEME_KEY)})
     || localStorage.getItem(${JSON.stringify(STARLIGHT_KEY)});
  if(ids.indexOf(v)<0){
    v = matchMedia('(prefers-color-scheme: light)').matches ? 'light' : 'dark';
    if(ids.indexOf(v)<0) v = d;
  }
  document.documentElement.dataset.theme = v;
}catch(e){document.documentElement.dataset.theme=${JSON.stringify(DEFAULT_THEME)};}})();`;
