/**
 * Application icons.
 *
 * Captured verbatim from the live app's DOM, not transcribed from component
 * source: several icons are passed as props or live in components that are
 * easy to mis-map, and reading the source produced the wrong glyph repeatedly.
 *
 * Geometry is a 24 viewport at stroke 1.6 — the web sidebar family. Phone and
 * browser-toolbar surfaces have their own sets at their own optical sizes.
 */

export const APP_ICONS = {
	'add': '<line x1="12" y1="5" x2="12" y2="19"/><line x1="5" y1="12" x2="19" y2="12"/>',
	'add-bookmark': '<path d="M19 21l-7-5-7 5V5a2 2 0 012-2h10a2 2 0 012 2z" />',
	'archive': '<polyline points="21 8 21 21 3 21 3 8"/><rect x="1" y="3" width="22" height="5"/><line x1="10" y1="12" x2="14" y2="12"/>',
	'articles': '<rect x="3" y="3" width="18" height="18" rx="2"/><line x1="7" y1="8" x2="17" y2="8"/><line x1="7" y1="12" x2="17" y2="12"/><line x1="7" y1="16" x2="13" y2="16"/>',
	'back-to-library': '<polyline points="15 18 9 12 15 6" />',
	'books': '<path d="M2 4h6a4 4 0 0 1 4 4v13a3 3 0 0 0-3-3H2V4z"/><path d="M22 4h-6a4 4 0 0 0-4 4v13a3 3 0 0 1 3-3h7V4z"/>',
	'chat': '<path d="M21 11.5a8.38 8.38 0 0 1-.9 3.8 8.5 8.5 0 0 1-7.6 4.7 8.38 8.38 0 0 1-3.8-.9L3 21l1.9-5.7a8.38 8.38 0 0 1-.9-3.8 8.5 8.5 0 0 1 4.7-7.6 8.38 8.38 0 0 1 3.8-.9h.5a8.48 8.48 0 0 1 8 8v.5z"/>',
	'chevron-right': '<polyline points="9 6 15 12 9 18" />',
	'chevron-down': '<polyline points="6 9 12 15 18 9" />',
	'collections': '<path d="M3 7v10a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2V9a2 2 0 0 0-2-2h-6l-2-2H5a2 2 0 0 0-2 2z"/>',
	'emails': '<path d="M4 4h16a2 2 0 0 1 2 2v12a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V6a2 2 0 0 1 2-2z"/><path d="M22 6L12 13 2 6"/>',
	'feed': '<circle cx="4" cy="20" r="1.5" fill="currentColor" stroke="none"/><path d="M4 13a7 7 0 0 1 7 7"/><path d="M4 6a14 14 0 0 1 14 14"/>',
	'filter': '<polygon points="22 3 2 3 10 12.46 10 19 14 21 14 12.46 22 3" />',
	'globe': '<circle cx="12" cy="12" r="10"/><path d="M2 12h20"/><path d="M12 2a15.3 15.3 0 0 1 4 10 15.3 15.3 0 0 1-4 10 15.3 15.3 0 0 1-4-10 15.3 15.3 0 0 1 4-10z"/>',
	'hide-detail-panel': '<rect x="3" y="4" width="14" height="12" rx="1.5"/><line x1="13" y1="4" x2="13" y2="16"/>',
	'hide-sidebar': '<rect x="3" y="4" width="14" height="12" rx="1.5"/><line x1="8" y1="4" x2="8" y2="16"/>',
	'home': '<path d="M3 10L12 3l9 7V22H3V10z"/><path d="M9 22V13h6v9"/>',
	'inbox': '<polyline points="22 12 16 12 14 15 10 15 8 12 2 12"/><path d="M5.45 5.11 2 12v6a2 2 0 0 0 2 2h16a2 2 0 0 0 2-2v-6l-3.45-6.89A2 2 0 0 0 16.76 4H7.24a2 2 0 0 0-1.79 1.11z"/>',
	'info': '<circle cx="12" cy="12" r="9"/><line x1="12" y1="11" x2="12" y2="16"/><circle cx="12" cy="8" r="0.5" fill="currentColor"/>',
	'later': '<circle cx="12" cy="12" r="9"/><polyline points="12 7 12 12 15.5 13.5"/>',
	'next-item': '<polyline points="6 9 12 15 18 9" />',
	'notebook': '<path d="M4 19.5A2.5 2.5 0 0 1 6.5 17H20"/><path d="M6.5 2H20v20H6.5A2.5 2.5 0 0 1 4 19.5v-15A2.5 2.5 0 0 1 6.5 2z"/>',
	'pause': '<rect x="6" y="5" width="4" height="14" rx="1"/><rect x="14" y="5" width="4" height="14" rx="1"/>',
	'pdfs': '<path d="M5 3h14a1 1 0 0 1 1 1v16a1 1 0 0 1-1 1H5a1 1 0 0 1-1-1V4a1 1 0 0 1 1-1z"/><line x1="4" y1="9" x2="20" y2="9"/><line x1="7" y1="13" x2="17" y2="13"/><line x1="7" y1="17" x2="13" y2="17"/>',
	'person': '<path d="M20 21v-2a4 4 0 0 0-4-4H8a4 4 0 0 0-4 4v2"/><circle cx="12" cy="7" r="4"/>',
	'pinned': '<path d="M19 21l-7-5-7 5V5a2 2 0 0 1 2-2h10a2 2 0 0 1 2 2z" />',
	'playback-speed-speedlabel': '<polyline points="6 9 12 15 18 9" />',
	'podcasts': '<path d="M3 18v-6a9 9 0 0 1 18 0v6" /> <path d="M21 19a2 2 0 0 1-2 2h-1a2 2 0 0 1-2-2v-3a2 2 0 0 1 2-2h3v5z" /> <path d="M3 19a2 2 0 0 0 2 2h1a2 2 0 0 0 2-2v-3a2 2 0 0 0-2-2H3v5z" />',
	'preferences': '<circle cx="12" cy="12" r="3"/><path d="M12 2v3M12 19v3M4.22 4.22l2.12 2.12M17.66 17.66l2.12 2.12M2 12h3M19 12h3M4.22 19.78l2.12-2.12M17.66 6.34l2.12-2.12"/>',
	'previous-item': '<polyline points="18 15 12 9 6 15" />',
	'remove': '<line x1="18" y1="6" x2="6" y2="18" /> <line x1="6" y1="6" x2="18" y2="18" />',
	'search': '<circle cx="11" cy="11" r="8"/><line x1="21" y1="21" x2="16.65" y2="16.65"/>',
	'skip-back-15-seconds': '<path d="M12 5V1L7 6l5 5V7c3.31 0 6 2.69 6 6s-2.69 6-6 6-6-2.69-6-6H4c0 4.42 3.58 8 8 8s8-3.58 8-8-3.58-8-8-8z" /> <text x="12" y="16.5" text-anchor="middle" font-size="7.5" font-weight="700" letter-spacing="-0.3" font-family="-apple-system, BlinkMacSystemFont, system-ui, sans-serif">15</text >',
	'skip-forward-15-seconds': '<path d="M12 5V1l5 5-5 5V7c-3.31 0-6 2.69-6 6s2.69 6 6 6 6-2.69 6-6h2c0 4.42-3.58 8-8 8s-8-3.58-8-8 3.58-8 8-8z" /> <text x="12" y="16.5" text-anchor="middle" font-size="7.5" font-weight="700" letter-spacing="-0.3" font-family="-apple-system, BlinkMacSystemFont, system-ui, sans-serif">15</text >',
	'stop-listening': '<line x1="18" y1="6" x2="6" y2="18" /> <line x1="6" y1="6" x2="18" y2="18" />',
	'switch-view': '<rect x="3" y="4" width="14" height="12" rx="1.5"/><line x1="13" y1="4" x2="13" y2="16"/>',
	'tags': '<path d="M20.59 13.41l-7.17 7.17a2 2 0 0 1-2.83 0L2 12V2h10l8.59 8.59a2 2 0 0 1 0 2.82z"/><circle cx="7" cy="7" r="1.5" fill="currentColor" stroke="none"/>',
	'trash': '<path d="M3 6h18"/><path d="M16 6V4a2 2 0 0 0-2-2h-4a2 2 0 0 0-2 2v2"/><path d="M19 6l-1 14a2 2 0 0 1-2 2H8a2 2 0 0 1-2-2L5 6"/><line x1="10" y1="11" x2="10" y2="17"/><line x1="14" y1="11" x2="14" y2="17"/>',
	'tweets': '<path d="M18.244 2.25h3.308l-7.227 8.26 8.502 11.24H16.17l-5.214-6.817L4.99 21.75H1.68l7.73-8.835L1.254 2.25H8.08l4.713 6.231zm-1.161 17.52h1.833L7.084 4.126H5.117z" fill="currentColor" stroke="none"/>',
	'videos': '<rect x="2" y="5" width="20" height="13" rx="2"/><path d="M10 8.5l5 3.5-5 3.5z" fill="currentColor" stroke="none"/><line x1="8" y1="21" x2="16" y2="21"/><line x1="12" y1="18" x2="12" y2="21"/>',
	'voice-selectedpersona-display-name-default': '<polyline points="6 9 12 15 18 9" />',
} as const;

export type AppIconName = keyof typeof APP_ICONS;
