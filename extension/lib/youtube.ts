import { escapeHtml } from './html'

export interface TranscriptSegment {
  startMs: number
  endMs?: number // populated from JSON3 dDurationMs / XML dur; enables true silence gap
  text: string
  newSpeaker?: boolean // true when a speaker-change marker (>>) was stripped from the front
}

export interface YouTubePageData {
  videoId: string
  title: string
  channelName: string
  description: string
  viewCount: string | undefined
  durationSeconds: number | undefined
  thumbnailUrl: string | undefined
  captionTrackUrl: string | undefined
  readerHtml: string
}

export function isYouTubePage(): boolean {
  const hostname = window.location.hostname
  if (hostname !== 'youtube.com' && hostname !== 'www.youtube.com') return false
  return new URLSearchParams(window.location.search).has('v')
}

interface VideoDetails {
  videoId?: string
  title?: string
  author?: string
  shortDescription?: string
  lengthSeconds?: string
  viewCount?: string
  thumbnail?: {
    thumbnails?: Array<{ url?: string; width?: number }>
  }
}

interface CaptionTrack {
  baseUrl?: string
  vssId?: string
  languageCode?: string
}

interface PlayerResponse {
  videoDetails?: VideoDetails
  captions?: {
    playerCaptionsTracklistRenderer?: {
      captionTracks?: CaptionTrack[]
    }
  }
}

export function extractYtPlayerResponse(): PlayerResponse | null {
  const scripts = document.querySelectorAll('script')
  const marker = 'var ytInitialPlayerResponse = '

  for (const script of scripts) {
    const text = script.textContent
    if (!text) continue

    const startIndex = text.indexOf(marker)
    if (startIndex === -1) continue

    const jsonStart = startIndex + marker.length
    let depth = 0
    let jsonEnd = -1

    let inString = false
    let escaped = false
    for (let i = jsonStart; i < text.length; i++) {
      const ch = text[i]
      if (escaped) {
        escaped = false
        continue
      }
      if (ch === '\\' && inString) {
        escaped = true
        continue
      }
      if (ch === '"') {
        inString = !inString
        continue
      }
      if (inString) continue
      if (ch === '{') depth++
      else if (ch === '}') {
        depth--
        if (depth === 0) {
          jsonEnd = i + 1
          break
        }
      }
    }

    if (jsonEnd === -1) continue

    try {
      return JSON.parse(text.slice(jsonStart, jsonEnd)) as PlayerResponse
    } catch {
      return null
    }
  }

  return null
}

function directTimedtextUrl(videoId: string): string {
  return `https://www.youtube.com/api/timedtext?v=${encodeURIComponent(videoId)}&lang=en&fmt=xml`
}

function pickCaptionTrackUrl(tracks: CaptionTrack[]): string | undefined {
  const english = tracks.find((t) => t.vssId?.includes('.en') || t.vssId?.includes('a.en'))
  const selected = english ?? tracks[0]
  return selected?.baseUrl ?? undefined
}

function pickLargestThumbnail(
  thumbnails: Array<{ url?: string; width?: number }>,
): string | undefined {
  if (thumbnails.length === 0) return undefined
  const sorted = [...thumbnails].sort((a, b) => (b.width ?? 0) - (a.width ?? 0))
  return sorted[0]?.url ?? undefined
}

function parseIsoDuration(iso: string): number | undefined {
  const match = iso.match(/^PT(?:(\d+)H)?(?:(\d+)M)?(?:(\d+)S)?$/)
  if (!match) return undefined
  const hours = parseInt(match[1] ?? '0', 10)
  const minutes = parseInt(match[2] ?? '0', 10)
  const seconds = parseInt(match[3] ?? '0', 10)
  return hours * 3600 + minutes * 60 + seconds
}

function extractFromMetaTags(): Partial<YouTubePageData> {
  const result: Partial<YouTubePageData> = {}

  const ogDesc = document.querySelector('meta[property="og:description"]')?.getAttribute('content')
  if (ogDesc) result.description = ogDesc

  const channelLink = document.querySelector<HTMLElement>('link[itemprop="name"]')
  const channelName = channelLink?.getAttribute('content')
  if (channelName) result.channelName = channelName

  const durationMeta = document.querySelector('meta[itemprop="duration"]')?.getAttribute('content')
  if (durationMeta) result.durationSeconds = parseIsoDuration(durationMeta)

  return result
}

// Extracts ytcfg values embedded in YouTube page scripts.
function extractYtcfgValue(key: string): string | undefined {
  const scripts = document.querySelectorAll('script')
  const pattern = `"${key}"`
  for (const script of scripts) {
    const text = script.textContent
    if (!text) continue
    const idx = text.indexOf(pattern)
    if (idx === -1) continue
    // Match "KEY":"value" or "KEY":number
    const after = text.slice(idx + pattern.length)
    const strMatch = after.match(/^:\s*"([^"]+)"/)
    if (strMatch) return strMatch[1]
    const numMatch = after.match(/^:\s*(\d+)/)
    if (numMatch) return numMatch[1]
  }
  return undefined
}

// IOS client returns caption URLs without PO Token gating (no exp=xpe).
// WEB client URLs require a Proof-of-Origin token and return empty without it.
// Request shape matches yt-dlp's IOS player request for maximum compatibility.
async function fetchPlayerResponse(videoId: string): Promise<PlayerResponse | null> {
  try {
    const visitorData = extractYtcfgValue('VISITOR_DATA')
    const sts = extractYtcfgValue('STS')

    const headers: Record<string, string> = {
      'Content-Type': 'application/json',
      'X-Youtube-Client-Name': '5',
      'X-Youtube-Client-Version': '20.10.4',
      Origin: 'https://www.youtube.com',
      'User-Agent': 'com.google.ios.youtube/20.10.4 (iPhone16,2; U; CPU iOS 18_3_2 like Mac OS X;)',
    }
    if (visitorData) headers['X-Goog-Visitor-Id'] = visitorData

    const body: Record<string, unknown> = {
      context: {
        client: {
          clientName: 'IOS',
          clientVersion: '20.10.4',
          deviceMake: 'Apple',
          deviceModel: 'iPhone16,2',
          userAgent:
            'com.google.ios.youtube/20.10.4 (iPhone16,2; U; CPU iOS 18_3_2 like Mac OS X;)',
          osName: 'iPhone',
          osVersion: '18.3.2.22D82',
          hl: 'en',
          ...(visitorData ? { visitorData } : {}),
        },
      },
      videoId,
      contentCheckOk: true,
      racyCheckOk: true,
    }

    if (sts) {
      body.playbackContext = {
        contentPlaybackContext: {
          html5Preference: 'HTML5_PREF_WANTS',
          signatureTimestamp: parseInt(sts, 10),
        },
      }
    }

    const response = await fetch('https://www.youtube.com/youtubei/v1/player?prettyPrint=false', {
      method: 'POST',
      headers,
      body: JSON.stringify(body),
    })
    if (!response.ok) return null
    return (await response.json()) as PlayerResponse
  } catch {
    return null
  }
}

function buildFromPlayer(videoId: string, player: PlayerResponse): YouTubePageData | null {
  const vd = player.videoDetails
  if (!vd || vd.videoId !== videoId) return null

  const title = vd.title ?? document.title
  const channelName = vd.author ?? ''
  const description = vd.shortDescription ?? ''
  const viewCount = vd.viewCount ?? undefined
  const durationSeconds = vd.lengthSeconds ? parseInt(vd.lengthSeconds, 10) : undefined
  const thumbnailUrl = vd.thumbnail?.thumbnails
    ? pickLargestThumbnail(vd.thumbnail.thumbnails)
    : undefined

  const captionTracks = player.captions?.playerCaptionsTracklistRenderer?.captionTracks
  const captionTrackUrl = captionTracks?.length
    ? pickCaptionTrackUrl(captionTracks)
    : directTimedtextUrl(videoId)

  return {
    videoId,
    title,
    channelName,
    description,
    viewCount,
    durationSeconds: Number.isFinite(durationSeconds) ? durationSeconds : undefined,
    thumbnailUrl,
    captionTrackUrl,
    readerHtml: '',
  }
}

export async function extractYouTubeData(): Promise<YouTubePageData | null> {
  const videoId = new URLSearchParams(window.location.search).get('v')
  if (!videoId) return null

  // Use embedded player response for metadata (instant, no network).
  const embedded = extractYtPlayerResponse()
  let result = embedded ? buildFromPlayer(videoId, embedded) : null

  // Fetch via IOS client to get caption URLs that aren't PO-Token-gated.
  // The WEB client's timedtext URLs (from the embedded response) return
  // empty without a Proof-of-Origin token.
  const fetched = await fetchPlayerResponse(videoId)
  if (fetched) {
    const fetchedResult = buildFromPlayer(videoId, fetched)
    if (fetchedResult) {
      if (result) {
        // Keep metadata from embedded (more complete), but use caption URL
        // from the non-WEB client (not PO-Token-gated).
        result.captionTrackUrl = fetchedResult.captionTrackUrl
      } else {
        result = fetchedResult
      }
    }
  }

  if (result) return result

  // Both paths failed. Fall back to meta tags (limited data).
  const meta = extractFromMetaTags()
  return {
    videoId,
    title: document.title,
    channelName: meta.channelName ?? '',
    description: meta.description ?? '',
    viewCount: undefined,
    durationSeconds: meta.durationSeconds,
    thumbnailUrl: undefined,
    captionTrackUrl: directTimedtextUrl(videoId),
    readerHtml: '',
  }
}

function decodeHtmlEntities(text: string): string {
  const textarea = document.createElement('textarea')
  textarea.innerHTML = text
  return textarea.value
}

// ---- Caption normalisation helpers ----

// Strips leading speaker-change markers (>>) and marks the segment.
// Optional fields are only set when they have meaningful values so that
// toEqual checks in tests are not broken by extra undefined properties.
function normalizeSegment(raw: string, startMs: number, endMs?: number): TranscriptSegment {
  const text = raw.trim()
  // Require at least one non-whitespace char after >> to avoid keeping bare marker segments
  const speakerMatch = text.match(/^>>\s*(.+)/)
  const speakerText = speakerMatch?.[1]?.trim()
  const seg: TranscriptSegment = {
    startMs,
    text: speakerText || text,
  }
  if (endMs !== undefined) seg.endMs = endMs
  if (speakerMatch) seg.newSpeaker = true
  return seg
}

// Splits segments where a speaker marker appears mid-text
// (e.g. "...Epstein. >> It was..." → two segments).
// The second and later pieces inherit the original segment's timing
// (best we can do without sub-segment data) and get newSpeaker: true.
function splitOnSpeakerMarkers(segments: TranscriptSegment[]): TranscriptSegment[] {
  const result: TranscriptSegment[] = []
  for (const seg of segments) {
    const parts = seg.text.split(/\s*>>\s*/)
    if (parts.length <= 1) {
      result.push(seg)
      continue
    }
    for (let i = 0; i < parts.length; i++) {
      const text = parts[i]?.trim() ?? ''
      if (!text) continue
      const piece: TranscriptSegment = { startMs: seg.startMs, text }
      if (seg.endMs !== undefined) piece.endMs = seg.endMs
      if (i > 0) piece.newSpeaker = true
      result.push(piece)
    }
  }
  return result
}

interface Json3Event {
  tStartMs?: number
  dDurationMs?: number
  aAppendMs?: number // present on rolling-append events — skip these
  segs?: Array<{ utf8?: string }>
}

interface Json3Response {
  wireMagic?: string
  events?: Json3Event[]
}

function parseJson3Segments(data: Json3Response): TranscriptSegment[] {
  const segments: TranscriptSegment[] = []
  if (!data.events) return segments
  for (const event of data.events) {
    // Rolling-append events duplicate the previous cue for visual continuity; skip them.
    if (event.aAppendMs !== undefined) continue
    if (!event.segs) continue
    const raw = event.segs
      .map((s) => s.utf8 ?? '')
      .join('')
      .replace(/\s+/g, ' ')
      .trim()
    if (!raw) continue
    const startMs = event.tStartMs ?? 0
    const endMs = event.dDurationMs != null ? startMs + event.dDurationMs : undefined
    segments.push(normalizeSegment(raw, startMs, endMs))
  }
  return segments
}

function parseXmlSegments(xml: string): TranscriptSegment[] {
  const doc = new DOMParser().parseFromString(xml, 'text/xml')
  const segments: TranscriptSegment[] = []
  for (const el of doc.querySelectorAll('text')) {
    const raw = el.textContent?.trim()
    if (!raw) continue
    const startAttr = el.getAttribute('start')
    const durAttr = el.getAttribute('dur')
    const startMs = startAttr ? Math.round(parseFloat(startAttr) * 1000) : 0
    const endMs = durAttr ? startMs + Math.round(parseFloat(durAttr) * 1000) : undefined
    segments.push(normalizeSegment(decodeHtmlEntities(raw), startMs, endMs))
  }
  return segments
}

async function fetchTimedtextSegments(url: URL): Promise<TranscriptSegment[]> {
  const response = await fetch(url.toString())
  if (!response.ok) return []
  const body = await response.text()
  if (!body) return []

  // Try JSON3 (srv3) format first — YouTube's default and most widely available.
  try {
    const json = JSON.parse(body) as Json3Response
    if (json.events) return parseJson3Segments(json)
  } catch {
    // Not JSON — fall through to XML parsing
  }

  return parseXmlSegments(body)
}

// Applies yt-dlp's URL modifications: set fmt, remove xosf (causes bad
// positioning data — see yt-dlp issue #13654).
function prepareTimedtextUrl(raw: string, fmt: string = 'json3'): URL {
  const url = new URL(raw)
  url.searchParams.set('fmt', fmt)
  url.searchParams.delete('xosf')
  return url
}

export async function fetchTranscript(captionTrackUrl: string): Promise<TranscriptSegment[]> {
  try {
    // Fast-path: if the transcript panel is already open, scrape directly.
    // DOM scraping cannot provide timestamps, so segments get startMs: 0.
    const transcriptRenderer = document.querySelector('ytd-transcript-renderer')
    if (transcriptRenderer) {
      const domSegs = transcriptRenderer.querySelectorAll(
        'ytd-transcript-segment-renderer yt-formatted-string.segment-text',
      )
      if (domSegs.length > 0) {
        const segments: TranscriptSegment[] = []
        for (const seg of domSegs) {
          const text = seg.textContent?.trim()
          if (text) segments.push(normalizeSegment(text, 0))
        }
        if (segments.length > 0) return segments
      }
    }

    // json3 first (yt-dlp's default, widest availability)
    const url = prepareTimedtextUrl(captionTrackUrl, 'json3')
    const segments = await fetchTimedtextSegments(url)
    if (segments.length > 0) return segments

    // Retry with kind=asr for auto-generated captions.
    if (!url.searchParams.has('kind')) {
      url.searchParams.set('kind', 'asr')
      const asrSegments = await fetchTimedtextSegments(url)
      if (asrSegments.length > 0) return asrSegments
    }

    // Last resort: try explicit XML format
    const xmlUrl = prepareTimedtextUrl(captionTrackUrl, 'xml')
    xmlUrl.searchParams.delete('kind')
    return await fetchTimedtextSegments(xmlUrl)
  } catch {
    return []
  }
}

function formatTimestamp(ms: number): string {
  const totalSeconds = Math.floor(ms / 1000)
  const hours = Math.floor(totalSeconds / 3600)
  const minutes = Math.floor((totalSeconds % 3600) / 60)
  const seconds = totalSeconds % 60
  if (hours > 0) {
    return `${hours}:${String(minutes).padStart(2, '0')}:${String(seconds).padStart(2, '0')}`
  }
  return `${minutes}:${String(seconds).padStart(2, '0')}`
}

export function extractHashtags(description: string): string[] {
  const matches = description.match(/#[\w\u00C0-\u024F]+/g)
  if (!matches) return []
  const seen = new Set<string>()
  const result: string[] = []
  for (const tag of matches) {
    const lower = tag.toLowerCase()
    if (!seen.has(lower)) {
      seen.add(lower)
      result.push(tag)
    }
  }
  return result
}

function formatViewCount(raw: string): string {
  const n = parseInt(raw, 10)
  if (!Number.isFinite(n)) return raw
  if (n >= 1_000_000_000) return `${(n / 1_000_000_000).toFixed(1).replace(/\.0$/, '')}B views`
  if (n >= 1_000_000) return `${(n / 1_000_000).toFixed(1).replace(/\.0$/, '')}M views`
  if (n >= 1_000) return `${(n / 1_000).toFixed(1).replace(/\.0$/, '')}K views`
  return `${n.toLocaleString()} views`
}

function formatDurationHuman(seconds: number): string {
  const h = Math.floor(seconds / 3600)
  const m = Math.floor((seconds % 3600) / 60)
  const s = seconds % 60
  if (h > 0) return `${h}:${String(m).padStart(2, '0')}:${String(s).padStart(2, '0')}`
  return `${m}:${String(s).padStart(2, '0')}`
}

// ---- Candidate-based paragraphizer ----

function countWords(text: string): number {
  return text.split(/\s+/).filter(Boolean).length
}

// Sentence boundary: punctuation optionally followed by closing quote/bracket.
function endsSentence(text: string): boolean {
  return /[.!?]['")\]]*\s*$/.test(text)
}

// Clause boundary (weaker fallback): comma, semicolon, colon.
function endsClause(text: string): boolean {
  return /[,;:]['")\]]*\s*$/.test(text)
}

// Groups normalised transcript segments into reading paragraphs.
//
// Signal hierarchy:
//   Hard break  — speaker change (newSpeaker) or long silence → flush entire current paragraph
//   Soft break  — sentence boundary once target word count reached → split at that boundary
//   Safety break — paragraph exceeds hard cap → split at best candidate (sentence → clause → current)
//
// Uses true silence gap (curr.startMs − prev.endMs) when endMs is available,
// falling back to start-to-start otherwise.
function paragraphize(
  segments: TranscriptSegment[],
  silenceGapMs = 4000,
  targetWords = 80,
  maxWords = 150,
): TranscriptSegment[][] {
  if (segments.length === 0) return []

  const result: TranscriptSegment[][] = []
  let current: TranscriptSegment[] = []
  let currentWords = 0
  let sentenceCandIdx = -1 // index in `current` of last sentence-ending segment
  let clauseCandIdx = -1 // index in `current` of last clause-ending segment (fallback)

  // Re-scan current for the best existing candidate positions after a partial flush.
  function scanCandidates(): void {
    sentenceCandIdx = -1
    clauseCandIdx = -1
    for (let j = 0; j < current.length; j++) {
      const segment = current[j]
      if (!segment) continue
      const t = segment.text
      if (endsSentence(t)) {
        sentenceCandIdx = j
        clauseCandIdx = j
      } else if (endsClause(t)) {
        clauseCandIdx = j
      }
    }
  }

  // Flush current[0..idx] as a paragraph; carry current[idx+1..] into the next.
  function flushAtIdx(idx: number): void {
    result.push(current.slice(0, idx + 1))
    current = current.slice(idx + 1)
    currentWords = current.reduce((sum, s) => sum + countWords(s.text), 0)
    scanCandidates()
  }

  function flushAll(): void {
    if (current.length > 0) result.push(current)
    current = []
    currentWords = 0
    sentenceCandIdx = -1
    clauseCandIdx = -1
  }

  function bestBreakIdx(): number {
    if (sentenceCandIdx >= 0) return sentenceCandIdx
    if (clauseCandIdx >= 0) return clauseCandIdx
    return current.length - 1
  }

  for (let i = 0; i < segments.length; i++) {
    const seg = segments[i]
    if (!seg) continue
    const prev = i > 0 ? (segments[i - 1] ?? null) : null

    // True silence: time from end of previous segment to start of this one.
    // Falls back to start-to-start when endMs is unavailable.
    const trueGap = prev ? seg.startMs - (prev.endMs ?? prev.startMs) : 0

    // Hard breaks: flush the entire current paragraph before this segment.
    if (current.length > 0 && (trueGap > silenceGapMs || seg.newSpeaker === true)) {
      flushAll()
    }

    current.push(seg)
    currentWords += countWords(seg.text)

    // Update candidate pointers.
    if (endsSentence(seg.text)) {
      sentenceCandIdx = current.length - 1
      clauseCandIdx = current.length - 1
    } else if (endsClause(seg.text)) {
      clauseCandIdx = current.length - 1
    }

    // Soft break: reached target and there's a clean sentence boundary to split on.
    if (currentWords >= targetWords && sentenceCandIdx >= 0) {
      flushAtIdx(sentenceCandIdx)
      continue
    }

    // Safety break: exceeded hard cap — use best available boundary.
    if (currentWords >= maxWords) {
      flushAtIdx(bestBreakIdx())
    }
  }

  if (current.length > 0) result.push(current)
  return result
}

export interface YouTubeReaderHtmlOptions {
  videoId: string
  description: string
  channelName: string
  viewCount: string | undefined
  durationSeconds: number | undefined
  segments: TranscriptSegment[]
}

export function buildYouTubeReaderHtml(opts: YouTubeReaderHtmlOptions): string {
  const escapedId = escapeHtml(opts.videoId)
  const descriptionHtml = escapeHtml(opts.description).replace(/\n/g, '<br>')
  const channelInitial = opts.channelName
    ? escapeHtml(opts.channelName.charAt(0).toUpperCase())
    : '?'
  const channelNameHtml = escapeHtml(opts.channelName || 'Unknown channel')

  // Channel stats line: view count + duration
  const statParts: string[] = []
  if (opts.viewCount) statParts.push(escapeHtml(formatViewCount(opts.viewCount)))
  if (opts.durationSeconds) statParts.push(escapeHtml(formatDurationHuman(opts.durationSeconds)))
  const statsHtml = statParts.join('<span class="yt-stat-dot"></span>')

  let html = `<div class="yt-embed">
  <iframe width="560" height="315" src="https://www.youtube.com/embed/${escapedId}" frameborder="0" allowfullscreen></iframe>
</div>
<div class="yt-channel-header">
  <div class="yt-channel-avatar">${channelInitial}</div>
  <div class="yt-channel-info">
    <span class="yt-channel-name">${channelNameHtml}</span>
    <div class="yt-video-stats">${statsHtml}</div>
  </div>
</div>
<div class="yt-description">${descriptionHtml}</div>`

  if (opts.segments.length > 0) {
    // Normalise: split any mid-segment >> speaker markers, then paragraphize.
    const normalized = splitOnSpeakerMarkers(opts.segments)
    const paragraphs = paragraphize(normalized)
    const paragraphsHtml = paragraphs
      .map((para) => {
        const spans = para
          .map(
            (seg) =>
              `<span class="t-seg" data-t="${escapeHtml(formatTimestamp(seg.startMs))}">${escapeHtml(seg.text)}</span>`,
          )
          .join(' ')
        return `<p>${spans}</p>`
      })
      .join('\n')

    html += `
<section class="yt-transcript">
  <h2>Transcript</h2>
  <div class="transcript-flow">${paragraphsHtml}</div>
</section>`
  }

  return html
}
