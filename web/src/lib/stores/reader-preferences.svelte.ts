import { browser } from '$app/environment';

export type ReaderTheme = 'light' | 'dark' | 'sepia' | 'auto';
export type ReaderTypeface = 'serif' | 'sans' | 'mono';
export type ReaderTextAlign = 'left' | 'justify';

export interface ReaderPreferences {
	theme: ReaderTheme;
	typeface: ReaderTypeface;
	fontSize: number;
	lineHeight: number;
	contentWidth: number;
	paragraphSpacing: number;
	textAlign: ReaderTextAlign;
}

const STORAGE_KEY = 'indelible:reader-preferences';

const DEFAULTS: ReaderPreferences = {
	theme: 'auto',
	typeface: 'sans',
	fontSize: 18,
	lineHeight: 1.75,
	contentWidth: 760,
	paragraphSpacing: 1.2,
	textAlign: 'left'
};

const FONT_STACKS: Record<ReaderTypeface, string> = {
	serif: "'Lora', Georgia, 'Times New Roman', serif",
	sans: "'Geist', -apple-system, BlinkMacSystemFont, 'Helvetica Neue', sans-serif",
	mono: "'Geist Mono', 'SF Mono', 'Fira Code', monospace"
};

function loadFromStorage(): ReaderPreferences {
	if (!browser) return { ...DEFAULTS };
	try {
		const raw = localStorage.getItem(STORAGE_KEY);
		if (!raw) return { ...DEFAULTS };
		const parsed = JSON.parse(raw) as Partial<ReaderPreferences>;
		return { ...DEFAULTS, ...parsed };
	} catch {
		return { ...DEFAULTS };
	}
}

function saveToStorage(prefs: ReaderPreferences): void {
	if (!browser) return;
	try {
		localStorage.setItem(STORAGE_KEY, JSON.stringify(prefs));
	} catch {
		// Storage full or unavailable
	}
}

function createReaderPreferences() {
	const initial = loadFromStorage();

	let theme = $state<ReaderTheme>(initial.theme);
	let typeface = $state<ReaderTypeface>(initial.typeface);
	let fontSize = $state(initial.fontSize);
	let lineHeight = $state(initial.lineHeight);
	let contentWidth = $state(initial.contentWidth);
	let paragraphSpacing = $state(initial.paragraphSpacing);
	let textAlign = $state<ReaderTextAlign>(initial.textAlign);

	const fontFamily = $derived(FONT_STACKS[typeface]);

	function persist() {
		saveToStorage({
			theme,
			typeface,
			fontSize,
			lineHeight,
			contentWidth,
			paragraphSpacing,
			textAlign
		});
	}

	return {
		get theme() {
			return theme;
		},
		set theme(v: ReaderTheme) {
			theme = v;
			persist();
		},

		get typeface() {
			return typeface;
		},
		set typeface(v: ReaderTypeface) {
			typeface = v;
			persist();
		},

		get fontSize() {
			return fontSize;
		},
		set fontSize(v: number) {
			fontSize = v;
			persist();
		},

		get lineHeight() {
			return lineHeight;
		},
		set lineHeight(v: number) {
			lineHeight = v;
			persist();
		},

		get contentWidth() {
			return contentWidth;
		},
		set contentWidth(v: number) {
			contentWidth = v;
			persist();
		},

		get paragraphSpacing() {
			return paragraphSpacing;
		},
		set paragraphSpacing(v: number) {
			paragraphSpacing = v;
			persist();
		},

		get textAlign() {
			return textAlign;
		},
		set textAlign(v: ReaderTextAlign) {
			textAlign = v;
			persist();
		},

		get fontFamily() {
			return fontFamily;
		},

		get defaults() {
			return DEFAULTS;
		},

		reset() {
			theme = DEFAULTS.theme;
			typeface = DEFAULTS.typeface;
			fontSize = DEFAULTS.fontSize;
			lineHeight = DEFAULTS.lineHeight;
			contentWidth = DEFAULTS.contentWidth;
			paragraphSpacing = DEFAULTS.paragraphSpacing;
			textAlign = DEFAULTS.textAlign;
			persist();
		}
	};
}

let instance: ReturnType<typeof createReaderPreferences> | null = null;

export function getReaderPreferences() {
	if (!instance) {
		instance = createReaderPreferences();
	}
	return instance;
}
