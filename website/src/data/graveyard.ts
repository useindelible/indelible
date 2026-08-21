/**
 * Read-it-later services that took their users' libraries with them.
 *
 * Every date is verified against public reporting. These are load-bearing
 * claims on a page that accuses other companies of losing people's data —
 * do not soften, round, or estimate them.
 */

export interface GraveEntry {
	name: string;
	dates: string;
	fate: string;
	/** The one that is still running: rendered as the answer, not a headstone. */
	live?: boolean;
}

export const GRAVEYARD: readonly GraveEntry[] = [
	{
		name: 'Pocket',
		dates: '2007 – 2025',
		fate: 'Mozilla shut it down on 8 July 2025. Every saved library was permanently deleted on 8 October 2025.',
	},
	{
		name: 'Omnivore',
		dates: '2022 – 2024',
		fate: 'Acquihired by ElevenLabs on 1 November 2024. Dead fourteen days later. It was open source, and that did not save anyone, because the data lived on their servers.',
	},
	{
		name: 'Indelible',
		dates: 'Your hardware',
		fate: 'One compose file, on a machine you own, under AGPL-3.0. There is no company in the position to take it away from you.',
		live: true,
	},
];

export const GRAVEYARD_NOTE =
	'Pocket and Omnivore dates verified against public reporting, August 2026.';
