/** Types shared across more than one component. */

/** A hero signature figure. Numbers count up on entry; strings print as-is. */
export interface Stat {
	value: number | string;
	label: string;
}

/** A primary navigation entry. */
export interface NavLink {
	label: string;
	href: string;
}

/**
 * One block of extracted prose.
 *
 * `emphasis` names a substring to italicise. It is a substring rather than
 * markup for the same reason highlights are: the phrase stays plain text that
 * the page renders into real elements, so nothing here can inject HTML.
 */
export interface ReaderBlock {
	kind: 'p' | 'h2' | 'hr';
	text?: string;
	emphasis?: string;
}

/**
 * Everything the reader screen needs to render one document.
 *
 * The screen takes this whole bundle as a prop, so showing a different
 * document — a 9-minute post or a 336,952-word novel — is a matter of passing
 * different data, never of a second copy of the screen.
 */
export interface ReaderDocument {
	title: string;
	/** Byline as the meta row under the title prints it; may be an address. */
	author: string;
	domain: string;
	published: string;
	/** Reading time as the meta row prints it. */
	length: string;
	/** Which of `ticks` scroll-map marks is the current position. */
	tick: number;
	ticks: number;
	blocks: readonly ReaderBlock[];
	record: {
		/**
		 * The author card, which is not always the meta row's byline: the app
		 * splits an address into a name and a handle there.
		 */
		author: { name: string; handle?: string };
		summary: string;
		fields: readonly {
			label: string;
			value: string;
			progress?: number;
		}[];
	};
}
