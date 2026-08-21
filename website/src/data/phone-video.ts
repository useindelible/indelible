/**
 * The saved video on a phone: the same YouTube item as the desktop screen,
 * with its transcript below the poster.
 */

export const PHONE_VIDEO = {
	title: 'The Beatles - Don’t Let Me Down (Live Performance)',
	channel: 'TheBeatlesVEVO',
	views: '575.8M views',
	duration: '3:31',
	domain: 'www.youtube.com',
} as const;

export interface TranscriptLine {
	at: string;
	text: string;
	current?: boolean;
}

export const PHONE_VIDEO_TRANSCRIPT: readonly TranscriptLine[] = [
	{ at: '0:00', text: 'The Beatles performing “Don’t Let Me Down.”', current: true },
	{ at: '0:22', text: 'Written by John as an expression of his love for Yoko Ono.' },
	{ at: '1:04', text: 'Recorded on the Apple rooftop in Savile Row, January 1969.' },
	{ at: '1:38', text: 'A composite of two performances used for Let It Be Naked.' },
	{ at: '2:15', text: 'The band’s last performance before an audience.' },
];
