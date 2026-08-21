/**
 * The themes the site ships.
 *
 * Adding one is three steps: add a block to styles/themes.css, add an entry
 * here, and nothing else. The toggle, the pre-paint script and the docs
 * hand-off all read this list.
 */

export interface ThemeDef {
	/** Matches the [data-theme='…'] selector in styles/themes.css. */
	id: string;
	label: string;
	/**
	 * Which of Starlight's two themes the docs should use when this one is
	 * active. Starlight only understands 'light' and 'dark', so a third
	 * palette has to declare which side of that line it sits on.
	 */
	base: 'light' | 'dark';
}

export const THEMES: readonly ThemeDef[] = [
	{ id: 'dark', label: 'Dark', base: 'dark' },
	{ id: 'light', label: 'Light', base: 'light' },
];

export const DEFAULT_THEME = 'dark';
