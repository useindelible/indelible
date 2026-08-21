/**
 * Browser-toolbar icons.
 *
 * A third family: 16 viewport at stroke 1.4. lib/icons.ts is the web sidebar
 * (24 / 1.6) and lib/phone-icons.ts is mobile (24 / 2). A browser toolbar sits
 * at a different optical size from either, so it needs its own weight.
 */

export const BROWSER_ICONS = {
	"back": "<path d=\"M13.2 8H3.2\"/><path d=\"M7 3.6L2.6 8 7 12.4\"/>",
	"chev-d": "<path d=\"M4 6l4 4 4-4\"/>",
	"chev-r": "<path d=\"M6 4l4 4-4 4\"/>",
	"chev-u": "<path d=\"M4 10l4-4 4 4\"/>",
	"chevs": "<path d=\"M3 4.4L6.2 8 3 11.6\"/><path d=\"M8.4 4.4L11.6 8 8.4 11.6\"/>",
	"close": "<path d=\"M4.2 4.2l7.6 7.6M11.8 4.2l-7.6 7.6\"/>",
	"folder": "<path d=\"M2.4 4.4a1 1 0 0 1 1-1h2.3l1.2 1.4h5.7a1 1 0 0 1 1 1v6a1 1 0 0 1-1 1H3.4a1 1 0 0 1-1-1z\"/>",
	"fwd": "<path d=\"M2.8 8h10\"/><path d=\"M9 3.6L13.4 8 9 12.4\"/>",
	"house": "<path d=\"M2.6 7.3L8 2.9l5.4 4.4\"/><path d=\"M4.1 6.5v6a1 1 0 0 0 1 1h5.8a1 1 0 0 0 1-1v-6\"/>",
	"inbox": "<path d=\"M2.5 9.5h3l1.5 2.5h2l1.5-2.5h3v3a1 1 0 01-1 1H3.5a1 1 0 01-1-1v-3z\"/><path d=\"M5 9.5V7a1 1 0 011-1h4a1 1 0 011 1v2.5\"/>",
	"kebab": "<circle cx=\"8\" cy=\"3.4\" r=\"1.2\"/><circle cx=\"8\" cy=\"8\" r=\"1.2\"/><circle cx=\"8\" cy=\"12.6\" r=\"1.2\"/>",
	"li": "<path d=\"M2.6 5.9h2.3v7.5H2.6z\" fill=\"currentColor\" stroke=\"none\"/><circle cx=\"3.75\" cy=\"3.4\" r=\"1.35\" fill=\"currentColor\" stroke=\"none\"/><path d=\"M6.6 13.4V5.9h2.2v1.03a2.7 2.7 0 012.3-1.2c1.75 0 2.7 1.13 2.7 3.2v4.47h-2.3V9.3c0-1.03-.42-1.66-1.28-1.66-.9 0-1.32.63-1.32 1.66v4.1z\" fill=\"currentColor\" stroke=\"none\"/>",
	"mag": "<circle cx=\"7.1\" cy=\"7.1\" r=\"4.3\"/><path d=\"M10.3 10.3L13.9 13.9\"/>",
	"noimg": "<rect x=\"2.4\" y=\"3.6\" width=\"11.2\" height=\"8.8\" rx=\"1.4\"/><path d=\"M2.9 12L6.4 8.4l2.3 2.4L10.6 9.2l2.6 2.8\"/><path d=\"M2.6 2.4l10.8 11.2\"/>",
	"note": "<path d=\"M2.2 5a2 2 0 0 1 2-2h5a2 2 0 0 1 2 2v2.5a2 2 0 0 1-2 2H5.4L2.8 12v-2.5h-.6z\"/><path d=\"M13 10v3.6M11.2 11.8h3.6\"/>",
	"puzzle": "<path d=\"M6.2 2.6h1.2a1.3 1.3 0 1 1 1.2 1.9v1.2h2.9v1.2a1.3 1.3 0 1 0 0 2.4v2.9H8.6v-1.2a1.3 1.3 0 1 0-2.4 0v1.2H3.3V9.3h1.2a1.3 1.3 0 1 0 0-2.4H3.3V4.5h2.9z\"/>",
	"reload": "<path d=\"M13.3 8a5.3 5.3 0 1 1-1.75-3.93\"/><path d=\"M13.5 2.4v3.1h-3.1\"/>",
	"shield": "<path d=\"M8 2.2l4.6 1.7v3.4c0 2.7-1.9 4.8-4.6 5.7-2.7-.9-4.6-3-4.6-5.7V3.9z\"/>",
	"star": "<path d=\"M8 2l1.85 3.95 4.35.46-3.25 2.97.93 4.27L8 11.55 4.12 13.65l.93-4.27L1.8 6.41l4.35-.46z\"/>",
	"tag": "<path d=\"M2.4 2.4h4.6a1 1 0 0 1 .7.3l4.3 4.3a1 1 0 0 1 0 1.4l-3.4 3.4a1 1 0 0 1-1.4 0L3 8.1a1 1 0 0 1-.6-.7V2.4z\"/><circle cx=\"4.6\" cy=\"4.6\" r=\"0.9\" fill=\"currentColor\" stroke=\"none\"/><path d=\"M12.7 10.5v3.6M10.9 12.3h3.6\"/>",
	"tick": "<path d=\"M3 8.2l3.4 3.4L13 4.6\"/>",
	"tune": "<path d=\"M2.4 5.2h5.1M10.9 5.2h2.7M2.4 10.8h2.7M8.1 10.8h5.5\"/><circle cx=\"9.2\" cy=\"5.2\" r=\"1.5\"/><circle cx=\"6.4\" cy=\"10.8\" r=\"1.5\"/>",
	"x": "<path d=\"M3.1 3.1l9.8 9.8M12.9 3.1L3.1 12.9\" stroke-width=\"1.7\"/>",
} as const;

export type BrowserIconName = keyof typeof BROWSER_ICONS;
