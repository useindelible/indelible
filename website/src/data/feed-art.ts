/**
 * Feed entry covers.
 *
 * Most entries ship no image, so the app draws a tinted placeholder. The two
 * that do have one (the Cooper portrait, the false-colour satellite frame) are
 * coded like every other cover on the page.
 */

export const FEED_ART = {
	"ph-blue": "<span class=\"cvr\"><i style=\"inset:0;background:#294169\"></i><i style=\"left:26%;top:30%;width:48%;height:40%;border:1.5px solid rgba(255,255,255,.45);border-radius:3px\"></i></span>",
	"ph-green": "<span class=\"cvr\"><i style=\"inset:0;background:#2C4E37\"></i><i style=\"left:26%;top:30%;width:48%;height:40%;border:1.5px solid rgba(255,255,255,.45);border-radius:3px\"></i></span>",
	"photo": "<span class=\"cvr\"><i style=\"inset:0;background:linear-gradient(165deg,#6E7885,#232A33)\"></i><i style=\"left:12%;bottom:-12%;width:76%;height:52%;background:#C9CDD3;border-radius:44% 44% 0 0\"></i><i style=\"left:22%;top:14%;width:56%;height:62%;background:#E4E7EB;border-radius:46%\"></i><i style=\"left:29%;top:26%;width:42%;height:38%;background:#1B222B;border-radius:40%\"></i><i style=\"left:33%;top:30%;width:16%;height:12%;background:rgba(255,255,255,.35);border-radius:50%\"></i></span>",
	"sat": "<span class=\"cvr\"><i style=\"inset:0;background:linear-gradient(140deg,#12324F,#1E7F8C 45%,#7FE3C8)\"></i><i style=\"left:18%;top:22%;width:64%;height:52%;background:linear-gradient(120deg,#B26FD1,#3F7FD6);border-radius:52% 40% 58% 44%;opacity:.85\"></i></span>",
} as const;

export type FeedArtName = keyof typeof FEED_ART;
