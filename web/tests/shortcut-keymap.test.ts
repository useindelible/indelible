import { readdirSync, readFileSync, statSync } from 'node:fs';
import { join, relative, sep } from 'node:path';
import { describe, expect, it } from 'vitest';

import en from '$lib/i18n/locales/en.json';
import fr from '$lib/i18n/locales/fr.json';
import { KEYMAP, chordId, shortcutDocRows } from '$lib/shortcuts/keymap';

const SOURCE_DIR = 'src';

/** Chords that two scopes may legitimately claim. Adding one is a deliberate act. */
const INTENTIONAL_OVERLAPS: string[] = [];

/** The one sanctioned global listener; everything else must go through the keymap. */
const DISPATCHER = 'src/lib/components/shortcuts/ShortcutHost.svelte';

/**
 * Listeners that stay hand-written: dismissal-only dropdowns, dialogs that own the
 * keyboard while focused, and reader surfaces that handle their own keys. A new
 * entry here is a new way for two handlers to claim one key, so it must be argued for.
 */
const BESPOKE_KEYDOWN_ALLOWLIST = [
	'src/lib/components/library/ContentTypeDropdown.svelte',
	'src/lib/components/library/ContextMenu.svelte',
	'src/lib/components/library/FilterBar.svelte',
	'src/lib/components/library/LibraryShell.svelte',
	'src/lib/components/library/LibrarySidebar.svelte',
	'src/lib/components/library/SaveViewModal.svelte',
	'src/lib/components/library/SortDropdown.svelte',
	'src/lib/components/library/ViewConfigDropdown.svelte',
	'src/lib/components/reader/FocusMode.svelte',
	'src/lib/components/reader/TypographyPopover.svelte',
	'src/routes/(app)/preferences/feed-management/+page.svelte'
];

const GLOBAL_LISTENER =
	/svelte:window[^>]*onkeydown|(?:document|window)\.addEventListener\('keydown'/;

function walk(dir: string): string[] {
	return readdirSync(dir).flatMap((entry) => {
		const full = join(dir, entry);
		return statSync(full).isDirectory() ? walk(full) : [full];
	});
}

function sourceFiles(): string[] {
	return walk(SOURCE_DIR)
		.filter((file) => /\.(svelte|ts)$/.test(file))
		.map((file) => relative('.', file).split(sep).join('/'))
		.sort();
}

describe('keymap', () => {
	it('lets exactly one binding claim a chord within a scope', () => {
		const seen = new Map<string, number>();
		for (const def of KEYMAP) {
			const tuple = `${def.scope}::${chordId(def.chord)}`;
			seen.set(tuple, (seen.get(tuple) ?? 0) + 1);
		}

		const duplicates = [...seen.entries()]
			.filter(([tuple, count]) => count > 1 && !INTENTIONAL_OVERLAPS.includes(tuple))
			.map(([tuple]) => tuple);

		expect(duplicates).toEqual([]);
	});

	it('never lets a global binding shadow a scoped one', () => {
		const globalChords = new Set(
			KEYMAP.filter((def) => def.scope === 'global').map((def) => chordId(def.chord))
		);

		const shadowed = KEYMAP.filter(
			(def) =>
				def.scope !== 'global' &&
				globalChords.has(chordId(def.chord)) &&
				!INTENTIONAL_OVERLAPS.includes(`${def.scope}::${chordId(def.chord)}`)
		).map((def) => `${def.scope}::${chordId(def.chord)}`);

		expect(shadowed).toEqual([]);
	});

	it('renders every cap as a single glyph so the copy scan stays quiet', () => {
		const multiCharacter = KEYMAP.flatMap((def) => def.caps).filter((cap) => [...cap].length !== 1);
		expect(multiCharacter).toEqual([]);
	});

	it('labels every binding with a key both catalogs carry', () => {
		const missing = KEYMAP.map((def) => def.labelKey).filter((key) => !(key in en) || !(key in fr));
		expect(missing).toEqual([]);
	});

	it('documents exactly the bindings users are told about', () => {
		expect(shortcutDocRows()).toEqual([
			{
				group: 'triage',
				groupLabelKey: 'prefs_reading_shortcuts_triage',
				rows: [
					{
						id: 'triage_archive',
						labelKey: 'prefs_reading_shortcut_archive_selected',
						caps: ['A']
					}
				]
			},
			{
				group: 'reading',
				groupLabelKey: 'prefs_reading_shortcuts_reading',
				rows: [
					{ id: 'select_next', labelKey: 'reader_next_item', caps: ['J'] },
					{ id: 'select_prev', labelKey: 'reader_previous_item', caps: ['K'] }
				]
			},
			{
				group: 'global',
				groupLabelKey: 'prefs_reading_shortcuts_global',
				rows: [{ id: 'add_url', labelKey: 'library_save_url', caps: ['N'] }]
			}
		]);
	});

	it('routes every global keydown listener through the dispatcher', () => {
		const offenders = sourceFiles().filter(
			(file) =>
				file !== DISPATCHER &&
				!BESPOKE_KEYDOWN_ALLOWLIST.includes(file) &&
				GLOBAL_LISTENER.test(readFileSync(file, 'utf8'))
		);

		expect(offenders).toEqual([]);
	});

	it('makes every transient menu or listbox declare shortcut suppression', () => {
		const declaredElsewhere = [
			// The library list is permanent, and its rows are focusable. Suppressing on
			// it would silence every shortcut the moment a row took focus.
			'src/lib/components/library/ItemList.svelte',
			// Covered by the add-item store's overlayOpen, declared in (app)/+layout.
			'src/lib/components/library/AddPopover.svelte',
			'src/lib/components/library/XPostModal.svelte',
			'src/lib/components/library/YouTubeModal.svelte',
			// Rendered under the sidebar's popupOpen, which declares suppression.
			'src/lib/components/library/SidebarUserMenu.svelte',
			// Typeahead lists owned by a focused text input, so the typing guard applies.
			'src/lib/components/library/TagInput.svelte',
			'src/lib/components/library/SaveUrlCollectionPicker.svelte',
			'src/lib/components/search/SearchAutocomplete.svelte'
		];

		const undeclared = sourceFiles().filter((file) => {
			if (!file.endsWith('.svelte') || declaredElsewhere.includes(file)) return false;
			const source = readFileSync(file, 'utf8');
			// Match the call, not the identifier: the import alone must not satisfy this.
			return (
				/role="(menu|listbox)"/.test(source) && !/suppressShortcutsWhileOpen\s*\(/.test(source)
			);
		});

		expect(undeclared).toEqual([]);
	});

	it('keeps the bespoke allowlist free of files that no longer listen', () => {
		const stale = BESPOKE_KEYDOWN_ALLOWLIST.filter(
			(file) => !GLOBAL_LISTENER.test(readFileSync(file, 'utf8'))
		);
		expect(stale).toEqual([]);
	});
});
