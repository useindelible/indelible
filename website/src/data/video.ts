/**
 * The saved YouTube item.
 *
 * The design source loads the real ytimg.com thumbnail here — the one raster
 * on the page. This site codes it instead: hotlinking a third-party image
 * from a marketing page sends every visitor's browser to Google, and the
 * page's whole argument is that everything on it is drawn, not screenshotted.
 * The poster below is the rooftop performance as flat shapes.
 */
import type { RecordField } from '../components/screens/app/RecordPanel.astro';

export const VIDEO = {
	title:
		'The Beatles - The Beatles - Don’t Let Me Down (Live Performance) [Mono / 2009 Remaster]',
	/** Truncated the way the overlay bar truncates it. */
	overlayTitle:
		'The Beatles - The Beatles - Don’t Let Me Down (Live Performance) [Mono',
	channel: 'TheBeatlesVEVO',
	domain: 'www.youtube.com',
	stats: '575.8M views • 3:31',
	posterAlt: 'The Beatles performing Don’t Let Me Down on the Apple rooftop',
} as const;

export const VIDEO_PARAGRAPHS: readonly string[] = [
	'The Beatles performing “Don’t Let Me Down.”',
	'Written by John as an expression of his love for Yoko Ono, the song is heartfelt and passionate. As John told Rolling Stone magazine in 1970, “When it gets down to it, when you’re drowning, you don’t say, ‘I would be incredibly pleased if someone would have the foresight to notice me drowning and come and help me,’ you just scream.”',
];

/** The rooftop: four figures on the ledge under a flat London sky. */
export const VIDEO_POSTER =
	'<span class="cvr">' +
	// sky, then the far rooftops, then the Apple building's own roof
	'<i style="inset:0;background:linear-gradient(180deg,#9AA6B0,#C3C8CB 58%,#AEB2B3)"></i>' +
	'<i style="left:0;top:52%;width:26%;height:12%;background:#7C8288"></i>' +
	'<i style="left:30%;top:49%;width:18%;height:15%;background:#888E93"></i>' +
	'<i style="left:70%;top:47%;width:30%;height:17%;background:#7C8288"></i>' +
	'<i style="left:0;top:64%;right:0;bottom:0;background:#4A4E51"></i>' +
	'<i style="left:0;top:62%;right:0;height:4%;background:#6A6E71"></i>' +
	// the band, small against it
	'<i style="left:27%;top:44%;width:5%;height:20%;background:#25282B;border-radius:44% 44% 0 0"></i>' +
	'<i style="left:39%;top:42%;width:5%;height:22%;background:#1B1E21;border-radius:44% 44% 0 0"></i>' +
	'<i style="left:51%;top:43%;width:5%;height:21%;background:#25282B;border-radius:44% 44% 0 0"></i>' +
	'<i style="left:62%;top:45%;width:5%;height:19%;background:#1B1E21;border-radius:44% 44% 0 0"></i>' +
	'</span>';

export const VIDEO_RECORD = {
	title: VIDEO.title,
	domain: VIDEO.domain,
	author: 'TheBeatl…',
	summary:
		'The Beatles perform Don’t Let Me Down in a passionate live rooftop rendition recorded at Apple in Savile Row. John Lennon writes the song as an expression of his love for Yoko Ono, and its urgent plea reflects the vulnerability of needing emotional support. The video presents a composite of two rooftop performances used for the Let It Be Naked album, during the band’s last performance before an audience.',
	fields: [
		{ label: 'Type', value: 'Video' },
		{ label: 'Domain', value: 'www.youtube.com' },
		{ label: 'Published', value: '—' },
		{ label: 'Length', value: '—' },
		{ label: 'Words', value: '—' },
		{ label: 'Saved', value: '7 hours ago' },
		{ label: 'Progress', value: '', progress: 37 },
		{ label: 'Last read', value: 'just now' },
		{ label: 'Language', value: 'English' },
	] satisfies RecordField[],
};
