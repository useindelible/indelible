import type { MessageKey } from '$lib/i18n';

/** `modal` carries no keymap rows; it exists so an open dialog can own the keyboard. */
export type ShortcutScope = 'global' | 'library' | 'search' | 'reader' | 'modal';

export type ShortcutId =
	| 'add_url'
	| 'add_rss'
	| 'select_next'
	| 'select_prev'
	| 'triage_archive'
	| 'reader_back'
	| 'focus_toggle'
	| 'chapter_prev'
	| 'chapter_next';

export type ShortcutGroup = 'triage' | 'reading' | 'global';

/** `mod` is meta or control. An absent modifier must be absent on the event. */
export type Chord = {
	key: string;
	mod?: boolean;
	alt?: boolean;
	shift?: boolean;
};

export type ShortcutDef = {
	id: ShortcutId;
	scope: ShortcutScope;
	chord: Chord;
	labelKey: MessageKey;
	/**
	 * Rendered key caps, one glyph per entry. Multi-character strings would trip
	 * the raw-copy scan in scripts/i18n-check.mjs, which exempts single characters.
	 */
	caps: string[];
	/** `null` keeps a working binding out of the documented tables. */
	group: ShortcutGroup | null;
};

const GROUP_LABEL_KEYS: Record<ShortcutGroup, MessageKey> = {
	triage: 'prefs_reading_shortcuts_triage',
	reading: 'prefs_reading_shortcuts_reading',
	global: 'prefs_reading_shortcuts_global'
};

export const KEYMAP: readonly ShortcutDef[] = [
	{
		id: 'add_url',
		scope: 'global',
		chord: { key: 'n' },
		labelKey: 'library_save_url',
		caps: ['N'],
		group: 'global'
	},
	{
		id: 'add_url',
		scope: 'global',
		chord: { key: 'n', mod: true },
		labelKey: 'library_save_url',
		caps: ['⌘', 'N'],
		group: null
	},
	{
		id: 'add_rss',
		scope: 'global',
		chord: { key: 'r' },
		labelKey: 'library_add_rss_feed',
		caps: ['R'],
		group: null
	},
	{
		id: 'triage_archive',
		scope: 'library',
		chord: { key: 'a' },
		labelKey: 'prefs_reading_shortcut_archive_selected',
		caps: ['A'],
		group: 'triage'
	},
	{
		id: 'select_next',
		scope: 'library',
		chord: { key: 'j' },
		labelKey: 'reader_next_item',
		caps: ['J'],
		group: 'reading'
	},
	{
		id: 'select_next',
		scope: 'library',
		chord: { key: 'ArrowDown' },
		labelKey: 'reader_next_item',
		caps: ['↓'],
		group: null
	},
	{
		id: 'select_prev',
		scope: 'library',
		chord: { key: 'k' },
		labelKey: 'reader_previous_item',
		caps: ['K'],
		group: 'reading'
	},
	{
		id: 'select_prev',
		scope: 'library',
		chord: { key: 'ArrowUp' },
		labelKey: 'reader_previous_item',
		caps: ['↑'],
		group: null
	},
	{
		id: 'select_next',
		scope: 'search',
		chord: { key: 'j' },
		labelKey: 'reader_next_item',
		caps: ['J'],
		group: null
	},
	{
		id: 'select_next',
		scope: 'search',
		chord: { key: 'ArrowDown' },
		labelKey: 'reader_next_item',
		caps: ['↓'],
		group: null
	},
	{
		id: 'select_prev',
		scope: 'search',
		chord: { key: 'k' },
		labelKey: 'reader_previous_item',
		caps: ['K'],
		group: null
	},
	{
		id: 'select_prev',
		scope: 'search',
		chord: { key: 'ArrowUp' },
		labelKey: 'reader_previous_item',
		caps: ['↑'],
		group: null
	},
	{
		id: 'reader_back',
		scope: 'reader',
		chord: { key: 'Escape' },
		labelKey: 'common_back',
		caps: ['⎋'],
		group: null
	},
	{
		id: 'focus_toggle',
		scope: 'reader',
		chord: { key: 'f' },
		labelKey: 'prefs_reading_shortcut_toggle_focus_mode',
		caps: ['F'],
		group: null
	},
	{
		id: 'chapter_prev',
		scope: 'reader',
		chord: { key: 'ArrowLeft' },
		labelKey: 'prefs_reading_shortcut_previous_chapter',
		caps: ['←'],
		group: null
	},
	{
		id: 'chapter_next',
		scope: 'reader',
		chord: { key: 'ArrowRight' },
		labelKey: 'prefs_reading_shortcut_next_chapter',
		caps: ['→'],
		group: null
	}
];

export function chordId(chord: Chord): string {
	const parts: string[] = [];
	if (chord.mod) parts.push('mod');
	if (chord.alt) parts.push('alt');
	if (chord.shift) parts.push('shift');
	parts.push(chord.key.length === 1 ? chord.key.toLowerCase() : chord.key);
	return parts.join('+');
}

type ModifierState = Pick<KeyboardEvent, 'key' | 'metaKey' | 'ctrlKey' | 'altKey' | 'shiftKey'>;

export function matchChord(event: ModifierState, chord: Chord): boolean {
	if ((event.metaKey || event.ctrlKey) !== Boolean(chord.mod)) return false;
	if (event.altKey !== Boolean(chord.alt)) return false;
	// Shift is only compared when a chord pins it, so `?` still matches on the
	// layouts that produce it with different physical keys.
	if (chord.shift !== undefined && event.shiftKey !== chord.shift) return false;

	if (chord.key.length === 1) return event.key.toLowerCase() === chord.key.toLowerCase();
	return event.key === chord.key;
}

/**
 * Caps for the primary way to reach an action. An id may carry several chords
 * (`n` and `mod+n`); the unmodified one is what the UI advertises.
 */
export function capsFor(id: ShortcutId): string[] {
	const rows = KEYMAP.filter((def) => def.id === id);
	const plain = rows.find((def) => !def.chord.mod && !def.chord.alt);
	return (plain ?? rows[0])?.caps ?? [];
}

export type DocGroup = {
	group: ShortcutGroup;
	groupLabelKey: MessageKey;
	rows: { id: ShortcutId; labelKey: MessageKey; caps: string[] }[];
};

const GROUP_ORDER: ShortcutGroup[] = ['triage', 'reading', 'global'];

/** The single source the preferences table and the help overlay both render. */
export function shortcutDocRows(): DocGroup[] {
	return GROUP_ORDER.map((group) => ({
		group,
		groupLabelKey: GROUP_LABEL_KEYS[group],
		rows: KEYMAP.filter((def) => def.group === group).map((def) => ({
			id: def.id,
			labelKey: def.labelKey,
			caps: def.caps
		}))
	})).filter((docGroup) => docGroup.rows.length > 0);
}
