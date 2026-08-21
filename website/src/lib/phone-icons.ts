/**
 * Phone icons.
 *
 * The mobile set: 24 viewport at stroke 2, matching IndelibleIcons.kt. These
 * are deliberately NOT the web sidebar icons in lib/icons.ts, which are a
 * denser family at stroke 1.6. Both are correct for their own surface — a
 * 22px phone tab bar and a 15px desktop sidebar sit at different optical
 * sizes and need different weights.
 */

export const PHONE_ICONS = {
	'article': '<rect x="3" y="3" width="18" height="18" rx="2"/><line x1="7" y1="8" x2="17" y2="8"/><line x1="7" y1="12" x2="17" y2="12"/><line x1="7" y1="16" x2="13" y2="16"/>',
	'book': '<path d="M2 4h6a4 4 0 014 4v13a3 3 0 00-3-3H2V4z"/><path d="M22 4h-6a4 4 0 00-4 4v13a3 3 0 013-3h7V4z"/>',
	'bookmark': '<path d="M6 4.5h12a1 1 0 011 1V20l-7-4-7 4V5.5a1 1 0 011-1z"/>',
	'check': '<circle cx="12" cy="12" r="8.5"/><path d="M8.4 12l2.5 2.5L15.6 9"/>',
	'chev-d': '<path d="M6 9.5l6 6 6-6"/>',
	'chev-l': '<path d="M15 5l-7 7 7 7"/>',
	'chev-r': '<path d="M9 6l6 6-6 6"/>',
	'clock': '<circle cx="12" cy="12" r="8.5"/><path d="M12 7.4V12l3 2"/>',
	'close': '<path d="M6 6l12 12M18 6L6 18"/>',
	'cog': '<circle cx="12" cy="12" r="3"/><path d="M12 2.5v2.8M12 18.7v2.8M4.5 4.5l2 2M17.5 17.5l2 2M2.5 12h2.8M18.7 12h2.8M4.5 19.5l2-2M17.5 6.5l2-2"/>',
	'email': '<path d="M4 4h16a2 2 0 012 2v12a2 2 0 01-2 2H4a2 2 0 01-2-2V6a2 2 0 012-2z"/><path d="M22 6L12 13 2 6"/>',
	'feed': '<circle cx="5.5" cy="18.5" r="1.6" fill="currentColor" stroke="none"/><path d="M4 11a9 9 0 019 9"/><path d="M4 4.5a16 16 0 0115.5 15.5"/>',
	'grid': '<rect x="3" y="3" width="7.5" height="7.5" rx="2"/><rect x="13.5" y="3" width="7.5" height="7.5" rx="2"/><rect x="3" y="13.5" width="7.5" height="7.5" rx="2"/><rect x="13.5" y="13.5" width="7.5" height="7.5" rx="2"/>',
	'highlight': '<path d="M4 19.5l1-3.5L15 6a2 2 0 012.8 0l.2.2a2 2 0 010 2.8L8 19l-3.5 1z"/><line x1="13" y1="8" x2="16" y2="11"/>',
	'home': '<path d="M4 11l8-7 8 7"/><path d="M6 9.5V20a1 1 0 001 1h10a1 1 0 001-1V9.5"/>',
	'inbox': '<path d="M21.5 12.5H16l-1.7 2.7H9.7L8 12.5H2.5"/><path d="M5.6 4.7L2.5 12.5V19a1.5 1.5 0 001.5 1.5h16a1.5 1.5 0 001.5-1.5v-6.5l-3.1-7.8A1.5 1.5 0 0016.9 4H7.1a1.5 1.5 0 00-1.5.7z"/>',
	'library': '<path d="M5 4.5A1.5 1.5 0 016.5 3H19a1 1 0 011 1v15.5a.5.5 0 01-.7.46L12 17l-7.3 2.96A.5.5 0 014 19.5V5.5z"/>',
	'menu': '<line x1="4" y1="7" x2="20" y2="7"/><line x1="4" y1="12" x2="14" y2="12"/><line x1="4" y1="17" x2="20" y2="17"/>',
	'more': '<circle cx="12" cy="5.5" r="1.4" fill="currentColor" stroke="none"/><circle cx="12" cy="12" r="1.4" fill="currentColor" stroke="none"/><circle cx="12" cy="18.5" r="1.4" fill="currentColor" stroke="none"/>',
	'move': '<path d="M4 9h13"/><path d="M13.5 5.5L17 9l-3.5 3.5"/><path d="M20 15H7"/><path d="M10.5 11.5L7 15l3.5 3.5"/>',
	'note': '<path d="M5 5h14a1 1 0 011 1v9a1 1 0 01-1 1H10l-4 3.5V16H5a1 1 0 01-1-1V6a1 1 0 011-1z"/>',
	'pdf': '<path d="M5 3h14a1 1 0 011 1v16a1 1 0 01-1 1H5a1 1 0 01-1-1V4a1 1 0 011-1z"/><line x1="4" y1="9" x2="20" y2="9"/><line x1="7" y1="13" x2="17" y2="13"/><line x1="7" y1="17" x2="13" y2="17"/>',
	/* Solid: pass `solid` to PhoneIcon so it fills instead of strokes. */
	'play': '<polygon points="6 4 20 12 6 20 6 4"/>',
	'plus': '<line x1="12" y1="5" x2="12" y2="19"/><line x1="5" y1="12" x2="19" y2="12"/>',
	'search': '<circle cx="11" cy="11" r="7"/><line x1="16.4" y1="16.4" x2="21" y2="21"/>',
	'spark': '<path d="M12 3l1.9 5.1L19 10l-5.1 1.9L12 17l-1.9-5.1L5 10l5.1-1.9z"/>',
	'tag': '<path d="M20.6 13.4l-7.2 7.2a2 2 0 01-2.8 0L3 13V3h10l7.6 7.6a2 2 0 010 2.8z"/><circle cx="7.5" cy="7.5" r="1.3" fill="currentColor" stroke="none"/>',
	'toc': '<line x1="8" y1="7" x2="20" y2="7"/><line x1="8" y1="12" x2="20" y2="12"/><line x1="8" y1="17" x2="16" y2="17"/><circle cx="4" cy="7" r="1" fill="currentColor" stroke="none"/><circle cx="4" cy="12" r="1" fill="currentColor" stroke="none"/><circle cx="4" cy="17" r="1" fill="currentColor" stroke="none"/>',
	'trash': '<path d="M3.5 6.5h17"/><path d="M16 6.5V5a2 2 0 00-2-2h-4a2 2 0 00-2 2v1.5"/><path d="M18.5 6.5L17.6 20a2 2 0 01-2 1.9H8.4a2 2 0 01-2-1.9L5.5 6.5"/>',
	'video': '<rect x="2" y="5" width="20" height="13" rx="2"/><path d="M10 8.5l5 3.5-5 3.5z" fill="currentColor" stroke="none"/><line x1="8" y1="21" x2="16" y2="21"/><line x1="12" y1="18" x2="12" y2="21"/>',
	'you': '<circle cx="12" cy="8" r="4"/><path d="M5.5 21a6.5 6.5 0 0113 0"/>',
} as const;

export type PhoneIconName = keyof typeof PHONE_ICONS;
