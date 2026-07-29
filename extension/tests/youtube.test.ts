import { describe, it, expect, vi, beforeEach } from 'vitest'

describe('youtube', () => {
  beforeEach(() => {
    vi.restoreAllMocks()
  })

  describe('isYouTubePage', () => {
    async function loadWithLocation(hostname: string, search: string) {
      vi.stubGlobal('window', {
        location: { hostname, search },
      })
      return (await import('../lib/youtube')).isYouTubePage
    }

    it('returns true for www.youtube.com with v param', async () => {
      const isYouTubePage = await loadWithLocation('www.youtube.com', '?v=dQw4w9WgXcQ')
      expect(isYouTubePage()).toBe(true)
    })

    it('returns true for youtube.com without www', async () => {
      const isYouTubePage = await loadWithLocation('youtube.com', '?v=abc123')
      expect(isYouTubePage()).toBe(true)
    })

    it('returns false when v param is missing', async () => {
      const isYouTubePage = await loadWithLocation('www.youtube.com', '?list=PLxxx')
      expect(isYouTubePage()).toBe(false)
    })

    it('returns false for non-youtube domains', async () => {
      const isYouTubePage = await loadWithLocation('example.com', '?v=abc')
      expect(isYouTubePage()).toBe(false)
    })

    it('returns false for youtube subdomains that are not www', async () => {
      const isYouTubePage = await loadWithLocation('music.youtube.com', '?v=abc')
      expect(isYouTubePage()).toBe(false)
    })
  })

  describe('escapeHtml via buildYouTubeReaderHtml', () => {
    it('escapes HTML special characters in description', async () => {
      const { buildYouTubeReaderHtml } = await import('../lib/youtube')
      const html = buildYouTubeReaderHtml({
        videoId: 'testId',
        description: '<script>alert("xss")</script>',
        channelName: 'Test',
        viewCount: undefined,
        durationSeconds: undefined,
        segments: [],
      })
      expect(html).not.toContain('<script>')
      expect(html).toContain('&lt;script&gt;')
    })

    it('escapes HTML special characters in transcript segments', async () => {
      const { buildYouTubeReaderHtml } = await import('../lib/youtube')
      const html = buildYouTubeReaderHtml({
        videoId: 'testId',
        description: '',
        channelName: 'Test',
        viewCount: undefined,
        durationSeconds: undefined,
        segments: [{ startMs: 0, text: '<img onerror="alert(1)">' }],
      })
      expect(html).not.toContain('<img onerror')
      expect(html).toContain('&lt;img onerror=')
    })

    it('escapes the video ID to prevent injection', async () => {
      const { buildYouTubeReaderHtml } = await import('../lib/youtube')
      const html = buildYouTubeReaderHtml({
        videoId: '" onload="alert(1)',
        description: 'desc',
        channelName: 'Test',
        viewCount: undefined,
        durationSeconds: undefined,
        segments: [],
      })
      expect(html).toContain('&quot; onload=&quot;alert(1)')
      expect(html).not.toContain('" onload="alert(1)')
    })
  })

  describe('buildYouTubeReaderHtml', () => {
    function buildOpts(overrides: Record<string, unknown> = {}) {
      return {
        videoId: 'id',
        description: 'desc',
        channelName: 'TED',
        viewCount: undefined as string | undefined,
        durationSeconds: undefined as number | undefined,
        segments: [] as Array<{ startMs: number; text: string }>,
        ...overrides,
      }
    }

    it('produces iframe embed with correct video ID', async () => {
      const { buildYouTubeReaderHtml } = await import('../lib/youtube')
      const html = buildYouTubeReaderHtml(
        buildOpts({ videoId: 'dQw4w9WgXcQ', description: 'A description' }),
      )
      expect(html).toContain('https://www.youtube.com/embed/dQw4w9WgXcQ')
      expect(html).toContain('<iframe')
      expect(html).toContain('allowfullscreen')
    })

    it('renders channel header with avatar and name', async () => {
      const { buildYouTubeReaderHtml } = await import('../lib/youtube')
      const html = buildYouTubeReaderHtml(buildOpts({ channelName: 'TED' }))
      expect(html).toContain('class="yt-channel-avatar">T</div>')
      expect(html).toContain('class="yt-channel-name">TED</span>')
    })

    it('renders view count and duration in stats', async () => {
      const { buildYouTubeReaderHtml } = await import('../lib/youtube')
      const html = buildYouTubeReaderHtml(
        buildOpts({
          viewCount: '77000000',
          durationSeconds: 1203,
        }),
      )
      expect(html).toContain('77M views')
      expect(html).toContain('20:03')
    })

    it('converts newlines to <br> in description', async () => {
      const { buildYouTubeReaderHtml } = await import('../lib/youtube')
      const html = buildYouTubeReaderHtml(buildOpts({ description: 'line1\nline2\nline3' }))
      expect(html).toContain('line1<br>line2<br>line3')
    })

    it('includes transcript section with timestamped spans in paragraphs', async () => {
      const { buildYouTubeReaderHtml } = await import('../lib/youtube')
      const html = buildYouTubeReaderHtml(
        buildOpts({
          segments: [
            { startMs: 0, text: 'Hello world' },
            { startMs: 2000, text: 'Second segment' },
          ],
        }),
      )
      expect(html).toContain('<section class="yt-transcript">')
      expect(html).toContain('<h2>Transcript</h2>')
      expect(html).toContain('<span class="t-seg" data-t="0:00">Hello world</span>')
      expect(html).toContain('<span class="t-seg" data-t="0:02">Second segment</span>')
      expect(html).toContain('<p>')
    })

    it('groups segments into paragraphs on timing gaps', async () => {
      const { buildYouTubeReaderHtml } = await import('../lib/youtube')
      const html = buildYouTubeReaderHtml(
        buildOpts({
          segments: [
            { startMs: 0, text: 'First' },
            { startMs: 1000, text: 'Still first paragraph' },
            { startMs: 10000, text: 'New paragraph after gap' },
          ],
        }),
      )
      const pMatches = html.match(/<p>/g)
      expect(pMatches?.length).toBe(2)
    })

    it('formats timestamps with hours when needed', async () => {
      const { buildYouTubeReaderHtml } = await import('../lib/youtube')
      const html = buildYouTubeReaderHtml(
        buildOpts({
          segments: [{ startMs: 3661000, text: 'After one hour' }],
        }),
      )
      expect(html).toContain('data-t="1:01:01"')
    })

    it('omits transcript section when segments array is empty', async () => {
      const { buildYouTubeReaderHtml } = await import('../lib/youtube')
      const html = buildYouTubeReaderHtml(buildOpts())
      expect(html).not.toContain('yt-transcript')
      expect(html).not.toContain('Transcript')
    })

    it('breaks into new paragraph at sentence boundary once target word count is reached', async () => {
      const { buildYouTubeReaderHtml } = await import('../lib/youtube')
      // Build enough segments to cross targetWords (80) — each ~7 words, no long silence
      // Sentence ends at segment 12 (after ~84 words), all gaps < 4000ms
      const segments = Array.from({ length: 20 }, (_, i) => ({
        startMs: i * 2000,
        endMs: i * 2000 + 2000,
        text:
          i === 11
            ? 'This is the end of that sentence.' // sentence boundary at ~84 words
            : 'This is a segment without a break',
      }))
      const html = buildYouTubeReaderHtml(buildOpts({ segments }))
      const pCount = (html.match(/<p>/g) ?? []).length
      expect(pCount).toBeGreaterThan(1)
    })

    it('breaks into new paragraph immediately on speaker change', async () => {
      const { buildYouTubeReaderHtml } = await import('../lib/youtube')
      const segments = [
        { startMs: 0, endMs: 2000, text: 'First speaker says something here.' },
        {
          startMs: 2000,
          endMs: 4000,
          text: '>> Second speaker responds now.',
          newSpeaker: undefined,
        },
      ]
      const html = buildYouTubeReaderHtml(buildOpts({ segments }))
      // The >> should be stripped from display and a paragraph break created
      expect(html).not.toContain('&gt;&gt;')
      const pCount = (html.match(/<p>/g) ?? []).length
      expect(pCount).toBe(2)
    })

    it('splits mid-segment >> markers before paragraphizing', async () => {
      const { buildYouTubeReaderHtml } = await import('../lib/youtube')
      const segments = [
        { startMs: 0, endMs: 2000, text: 'Jeffrey Epstein. >> It was an incident.' },
      ]
      const html = buildYouTubeReaderHtml(buildOpts({ segments }))
      // Stripped from display
      expect(html).not.toContain('&gt;&gt;')
      // Split into two paragraphs (sentence end → new paragraph for speaker)
      const pCount = (html.match(/<p>/g) ?? []).length
      expect(pCount).toBe(2)
    })

    it('applies safety break (hard cap) when no sentence punctuation exists', async () => {
      const { buildYouTubeReaderHtml } = await import('../lib/youtube')
      // 30 segments × 8 words = 240 words total, no punctuation, no silence.
      // Hard cap (150 words) fires at segment 19 (152 words), flushing a first paragraph.
      // Remaining 11 segments form a second paragraph at end of loop.
      const segments = Array.from({ length: 30 }, (_, i) => ({
        startMs: i * 2000,
        endMs: i * 2000 + 2000,
        text: 'one two three four five six seven eight',
      }))
      const html = buildYouTubeReaderHtml(buildOpts({ segments }))
      const pCount = (html.match(/<p>/g) ?? []).length
      expect(pCount).toBeGreaterThan(1)
    })
  })

  describe('extractHashtags', () => {
    it('extracts unique hashtags from description', async () => {
      const { extractHashtags } = await import('../lib/youtube')
      const tags = extractHashtags('Check out #education and #creativity #education')
      expect(tags).toEqual(['#education', '#creativity'])
    })

    it('returns empty array when no hashtags', async () => {
      const { extractHashtags } = await import('../lib/youtube')
      expect(extractHashtags('No tags here')).toEqual([])
    })

    it('handles hashtags with unicode characters', async () => {
      const { extractHashtags } = await import('../lib/youtube')
      const tags = extractHashtags('#café #naïve')
      expect(tags).toEqual(['#café', '#naïve'])
    })
  })

  describe('extractYtPlayerResponse', () => {
    it('extracts player response from script tags', async () => {
      const playerData = {
        videoDetails: { videoId: 'abc123', title: 'Test Video' },
      }
      const scriptContent = `var ytInitialPlayerResponse = ${JSON.stringify(playerData)};`

      const mockDoc = {
        querySelectorAll: vi
          .fn()
          .mockReturnValue([
            { textContent: 'unrelated script content' },
            { textContent: scriptContent },
          ]),
      }
      vi.stubGlobal('document', mockDoc)

      const { extractYtPlayerResponse } = await import('../lib/youtube')
      const result = extractYtPlayerResponse()
      expect(result).not.toBeNull()
      expect(result?.videoDetails?.videoId).toBe('abc123')
      expect(result?.videoDetails?.title).toBe('Test Video')
    })

    it('returns null when no player response script found', async () => {
      const mockDoc = {
        querySelectorAll: vi.fn().mockReturnValue([{ textContent: 'var unrelated = true;' }]),
      }
      vi.stubGlobal('document', mockDoc)

      const { extractYtPlayerResponse } = await import('../lib/youtube')
      expect(extractYtPlayerResponse()).toBeNull()
    })

    it('returns null when JSON is malformed', async () => {
      const mockDoc = {
        querySelectorAll: vi
          .fn()
          .mockReturnValue([{ textContent: 'var ytInitialPlayerResponse = {invalid json};' }]),
      }
      vi.stubGlobal('document', mockDoc)

      const { extractYtPlayerResponse } = await import('../lib/youtube')
      expect(extractYtPlayerResponse()).toBeNull()
    })

    it('handles nested braces in JSON correctly', async () => {
      const playerData = {
        videoDetails: {
          videoId: 'xyz',
          thumbnail: { thumbnails: [{ url: 'http://img.com/1.jpg', width: 120 }] },
        },
      }
      const scriptContent = `var ytInitialPlayerResponse = ${JSON.stringify(playerData)};var next = true;`

      const mockDoc = {
        querySelectorAll: vi.fn().mockReturnValue([{ textContent: scriptContent }]),
      }
      vi.stubGlobal('document', mockDoc)

      const { extractYtPlayerResponse } = await import('../lib/youtube')
      const result = extractYtPlayerResponse()
      expect(result?.videoDetails?.videoId).toBe('xyz')
    })

    it('correctly parses when title or description contains brace characters', async () => {
      const playerData = {
        videoDetails: {
          videoId: 'brace-test',
          title: 'How to use {curly braces} in code',
          shortDescription: 'A description with } and { inside',
        },
      }
      const scriptContent = `var ytInitialPlayerResponse = ${JSON.stringify(playerData)};`

      const mockDoc = {
        querySelectorAll: vi.fn().mockReturnValue([{ textContent: scriptContent }]),
      }
      vi.stubGlobal('document', mockDoc)

      const { extractYtPlayerResponse } = await import('../lib/youtube')
      const result = extractYtPlayerResponse()
      expect(result?.videoDetails?.videoId).toBe('brace-test')
      expect((result?.videoDetails as Record<string, unknown>).title).toBe(
        'How to use {curly braces} in code',
      )
    })
  })

  describe('extractYouTubeData', () => {
    it('returns null when v param is missing', async () => {
      vi.stubGlobal('window', {
        location: { hostname: 'www.youtube.com', search: '' },
      })
      const mockDoc = {
        querySelectorAll: vi.fn().mockReturnValue([]),
        querySelector: vi.fn().mockReturnValue(null),
        title: 'YouTube',
      }
      vi.stubGlobal('document', mockDoc)

      const { extractYouTubeData } = await import('../lib/youtube')
      expect(await extractYouTubeData()).toBeNull()
    })

    it('extracts data from player response', async () => {
      vi.stubGlobal('window', {
        location: { hostname: 'www.youtube.com', search: '?v=abc123' },
      })

      const playerData = {
        videoDetails: {
          videoId: 'abc123',
          title: 'Test Video',
          author: 'Test Channel',
          shortDescription: 'A test video description',
          lengthSeconds: '240',
          thumbnail: {
            thumbnails: [
              { url: 'http://img.com/small.jpg', width: 120 },
              { url: 'http://img.com/large.jpg', width: 1280 },
            ],
          },
        },
        captions: {
          playerCaptionsTracklistRenderer: {
            captionTracks: [{ baseUrl: 'http://youtube.com/timedtext?lang=en', vssId: '.en' }],
          },
        },
      }

      const mockDoc = {
        querySelectorAll: vi
          .fn()
          .mockReturnValue([
            { textContent: `var ytInitialPlayerResponse = ${JSON.stringify(playerData)};` },
          ]),
        title: 'Test Video - YouTube',
      }
      vi.stubGlobal('document', mockDoc)

      const { extractYouTubeData } = await import('../lib/youtube')
      const result = await extractYouTubeData()

      expect(result).not.toBeNull()
      expect(result!.videoId).toBe('abc123')
      expect(result!.title).toBe('Test Video')
      expect(result!.channelName).toBe('Test Channel')
      expect(result!.description).toBe('A test video description')
      expect(result!.durationSeconds).toBe(240)
      expect(result!.thumbnailUrl).toBe('http://img.com/large.jpg')
      expect(result!.captionTrackUrl).toBe('http://youtube.com/timedtext?lang=en')
    })

    it('prefers English caption tracks', async () => {
      vi.stubGlobal('window', {
        location: { hostname: 'www.youtube.com', search: '?v=abc123' },
      })

      const playerData = {
        videoDetails: {
          videoId: 'abc123',
          title: 'Test',
          author: 'Channel',
          shortDescription: 'desc',
          lengthSeconds: '60',
        },
        captions: {
          playerCaptionsTracklistRenderer: {
            captionTracks: [
              { baseUrl: 'http://yt.com/timedtext?lang=fr', vssId: '.fr' },
              { baseUrl: 'http://yt.com/timedtext?lang=en', vssId: 'a.en' },
              { baseUrl: 'http://yt.com/timedtext?lang=de', vssId: '.de' },
            ],
          },
        },
      }

      const mockDoc = {
        querySelectorAll: vi
          .fn()
          .mockReturnValue([
            { textContent: `var ytInitialPlayerResponse = ${JSON.stringify(playerData)};` },
          ]),
        title: 'Test',
      }
      vi.stubGlobal('document', mockDoc)

      const { extractYouTubeData } = await import('../lib/youtube')
      const result = await extractYouTubeData()
      expect(result!.captionTrackUrl).toBe('http://yt.com/timedtext?lang=en')
    })

    it('falls back to meta tags when player response is absent and API fails', async () => {
      vi.stubGlobal('window', {
        location: { hostname: 'www.youtube.com', search: '?v=fallback' },
      })

      const mockDoc = {
        querySelectorAll: vi.fn().mockReturnValue([]),
        querySelector: vi.fn().mockImplementation((selector: string) => {
          if (selector === 'meta[property="og:description"]') {
            return { getAttribute: () => 'OG description text' }
          }
          if (selector === 'link[itemprop="name"]') {
            return { getAttribute: () => 'Channel From Meta' }
          }
          if (selector === 'meta[itemprop="duration"]') {
            return { getAttribute: () => 'PT4M33S' }
          }
          return null
        }),
        title: 'Fallback Video - YouTube',
      }
      vi.stubGlobal('document', mockDoc)

      // Player API also fails
      vi.stubGlobal('fetch', vi.fn().mockResolvedValue({ ok: false }))

      const { extractYouTubeData } = await import('../lib/youtube')
      const result = await extractYouTubeData()

      expect(result).not.toBeNull()
      expect(result!.videoId).toBe('fallback')
      expect(result!.title).toBe('Fallback Video - YouTube')
      expect(result!.channelName).toBe('Channel From Meta')
      expect(result!.description).toBe('OG description text')
      expect(result!.durationSeconds).toBe(273)
      expect(result!.captionTrackUrl).toBe(
        'https://www.youtube.com/api/timedtext?v=fallback&lang=en&fmt=xml',
      )
    })

    it('fetches fresh player data via API when embedded response is stale (SPA navigation)', async () => {
      vi.stubGlobal('window', {
        location: { hostname: 'www.youtube.com', search: '?v=current-video' },
      })

      const stalePlayerData = {
        videoDetails: {
          videoId: 'original-video',
          title: 'Original Video',
          author: 'Some Channel',
          shortDescription: 'Original description',
          lengthSeconds: '187',
        },
      }

      const freshApiResponse = {
        videoDetails: {
          videoId: 'current-video',
          title: 'Current Video',
          author: 'Current Channel',
          shortDescription: 'Current description from API',
          lengthSeconds: '322',
          thumbnail: {
            thumbnails: [{ url: 'http://img.com/thumb.jpg', width: 1280 }],
          },
        },
        captions: {
          playerCaptionsTracklistRenderer: {
            captionTracks: [{ baseUrl: 'http://yt.com/timedtext?signed=abc', vssId: '.en' }],
          },
        },
      }

      const mockDoc = {
        querySelectorAll: vi.fn().mockReturnValue([
          {
            textContent: `var ytInitialPlayerResponse = ${JSON.stringify(stalePlayerData)};`,
          },
        ]),
        querySelector: vi.fn().mockReturnValue(null),
        title: 'Current Video Title - YouTube',
      }
      vi.stubGlobal('document', mockDoc)

      const fetchMock = vi.fn().mockResolvedValue({
        ok: true,
        json: () => Promise.resolve(freshApiResponse),
      })
      vi.stubGlobal('fetch', fetchMock)

      const { extractYouTubeData } = await import('../lib/youtube')
      const result = await extractYouTubeData()

      expect(result).not.toBeNull()
      expect(result!.videoId).toBe('current-video')
      expect(result!.title).toBe('Current Video')
      expect(result!.channelName).toBe('Current Channel')
      expect(result!.description).toBe('Current description from API')
      expect(result!.durationSeconds).toBe(322)
      expect(result!.thumbnailUrl).toBe('http://img.com/thumb.jpg')
      expect(result!.captionTrackUrl).toBe('http://yt.com/timedtext?signed=abc')

      // Verify the player API was called with correct video ID
      const playerCall = fetchMock.mock.calls.find(
        (c: unknown[]) => typeof c[0] === 'string' && (c[0] as string).includes('/player'),
      )
      expect(playerCall).toBeTruthy()
      const body = JSON.parse(playerCall?.[1]?.body as string)
      expect(body.videoId).toBe('current-video')
      expect(body.context.client.clientName).toBe('IOS')
    })

    it('falls back to meta tags when both embedded and API player responses fail', async () => {
      vi.stubGlobal('window', {
        location: { hostname: 'www.youtube.com', search: '?v=current-video' },
      })

      const stalePlayerData = {
        videoDetails: {
          videoId: 'original-video',
          title: 'Original Video',
          author: 'Some Channel',
          shortDescription: 'Original description',
          lengthSeconds: '187',
        },
      }

      const mockDoc = {
        querySelectorAll: vi.fn().mockReturnValue([
          {
            textContent: `var ytInitialPlayerResponse = ${JSON.stringify(stalePlayerData)};`,
          },
        ]),
        querySelector: vi.fn().mockImplementation((selector: string) => {
          if (selector === 'meta[property="og:description"]') {
            return { getAttribute: () => 'Current video description' }
          }
          if (selector === 'meta[itemprop="duration"]') {
            return { getAttribute: () => 'PT8M30S' }
          }
          return null
        }),
        title: 'Current Video Title - YouTube',
      }
      vi.stubGlobal('document', mockDoc)

      // Player API fails
      vi.stubGlobal('fetch', vi.fn().mockResolvedValue({ ok: false }))

      const { extractYouTubeData } = await import('../lib/youtube')
      const result = await extractYouTubeData()

      expect(result).not.toBeNull()
      expect(result!.videoId).toBe('current-video')
      expect(result!.title).toBe('Current Video Title - YouTube')
      expect(result!.description).toBe('Current video description')
      expect(result!.durationSeconds).toBe(510)
      expect(result!.captionTrackUrl).toBe(
        'https://www.youtube.com/api/timedtext?v=current-video&lang=en&fmt=xml',
      )
    })
  })

  describe('fetchTranscript', () => {
    it('parses XML caption response with timestamps and decodes entities', async () => {
      const xmlResponse = `<?xml version="1.0" encoding="utf-8"?>
<transcript>
  <text start="0" dur="2">Hello &amp; welcome</text>
  <text start="2.5" dur="3">to the video</text>
</transcript>`

      vi.stubGlobal('document', {
        querySelector: vi.fn().mockReturnValue(null),
        createElement: vi.fn().mockImplementation(() => {
          const el = { innerHTML: '', value: '' }
          Object.defineProperty(el, 'value', {
            get() {
              const text = el.innerHTML
              return text
                .replace(/&amp;/g, '&')
                .replace(/&lt;/g, '<')
                .replace(/&gt;/g, '>')
                .replace(/&quot;/g, '"')
                .replace(/&#x27;/g, "'")
                .replace(/&#39;/g, "'")
            },
          })
          return el
        }),
      })

      vi.stubGlobal(
        'fetch',
        vi.fn().mockResolvedValue({
          ok: true,
          text: () => Promise.resolve(xmlResponse),
        }),
      )

      vi.stubGlobal(
        'DOMParser',
        class {
          parseFromString(...args: [string, string]) {
            const str = args[0]
            const texts: Array<{
              textContent: string
              getAttribute: (name: string) => string | null
            }> = []
            const regex = /<text\s+start="([^"]*)"[^>]*>([\s\S]*?)<\/text>/g
            let match
            while ((match = regex.exec(str)) !== null) {
              const start = match[1]!
              const content = match[2]!.trim()
              texts.push({
                textContent: content,
                getAttribute: (name: string) => (name === 'start' ? start : null),
              })
            }
            return {
              querySelectorAll: (sel: string) => {
                if (sel === 'text') return texts
                return []
              },
            }
          }
        },
      )

      const { fetchTranscript } = await import('../lib/youtube')
      const result = await fetchTranscript('https://www.youtube.com/api/timedtext?lang=en')
      expect(result).toEqual([
        { startMs: 0, text: 'Hello & welcome' },
        { startMs: 2500, text: 'to the video' },
      ])
    })

    it('parses JSON3 (srv3) caption response with timestamps and endMs', async () => {
      const json3Response = JSON.stringify({
        wireMagic: 'pb3',
        events: [
          { tStartMs: 0, dDurationMs: 2000, segs: [{ utf8: 'Hello world' }] },
          { tStartMs: 2000, dDurationMs: 3000, segs: [{ utf8: 'Second line' }] },
          { tStartMs: 5000, dDurationMs: 1000, segs: [{ utf8: '\n' }] },
        ],
      })

      vi.stubGlobal('document', {
        querySelector: vi.fn().mockReturnValue(null),
      })

      vi.stubGlobal(
        'fetch',
        vi.fn().mockResolvedValue({
          ok: true,
          text: () => Promise.resolve(json3Response),
        }),
      )

      const { fetchTranscript } = await import('../lib/youtube')
      const result = await fetchTranscript('https://www.youtube.com/api/timedtext?lang=en')
      expect(result).toEqual([
        { startMs: 0, endMs: 2000, text: 'Hello world' },
        { startMs: 2000, endMs: 5000, text: 'Second line' },
      ])
    })

    it('skips rolling-append events (aAppendMs) from JSON3', async () => {
      const json3Response = JSON.stringify({
        wireMagic: 'pb3',
        events: [
          { tStartMs: 0, dDurationMs: 2710, segs: [{ utf8: 'In 2005, the Palm Beach Police' }] },
          // Rolling-append separator — must be skipped
          {
            tStartMs: 2710,
            dDurationMs: 10,
            aAppendMs: 0,
            segs: [{ utf8: 'In 2005, the Palm Beach Police\n ' }],
          },
          {
            tStartMs: 2720,
            dDurationMs: 2150,
            segs: [{ utf8: 'Department in Florida received a phone' }],
          },
        ],
      })

      vi.stubGlobal('document', { querySelector: vi.fn().mockReturnValue(null) })
      vi.stubGlobal(
        'fetch',
        vi.fn().mockResolvedValue({
          ok: true,
          text: () => Promise.resolve(json3Response),
        }),
      )

      const { fetchTranscript } = await import('../lib/youtube')
      const result = await fetchTranscript('https://www.youtube.com/api/timedtext?lang=en')
      // Append event must not appear in output
      expect(result).toEqual([
        { startMs: 0, endMs: 2710, text: 'In 2005, the Palm Beach Police' },
        { startMs: 2720, endMs: 4870, text: 'Department in Florida received a phone' },
      ])
    })

    it('strips leading >> speaker markers and sets newSpeaker flag', async () => {
      const json3Response = JSON.stringify({
        wireMagic: 'pb3',
        events: [
          { tStartMs: 0, dDurationMs: 2000, segs: [{ utf8: 'by a man named Jeffrey Epstein.' }] },
          { tStartMs: 2000, dDurationMs: 1500, segs: [{ utf8: '>> It was an incident.' }] },
        ],
      })

      vi.stubGlobal('document', { querySelector: vi.fn().mockReturnValue(null) })
      vi.stubGlobal(
        'fetch',
        vi.fn().mockResolvedValue({
          ok: true,
          text: () => Promise.resolve(json3Response),
        }),
      )

      const { fetchTranscript } = await import('../lib/youtube')
      const result = await fetchTranscript('https://www.youtube.com/api/timedtext?lang=en')
      expect(result).toEqual([
        { startMs: 0, endMs: 2000, text: 'by a man named Jeffrey Epstein.' },
        { startMs: 2000, endMs: 3500, text: 'It was an incident.', newSpeaker: true },
      ])
    })

    it('fetches without forcing fmt=xml first', async () => {
      vi.stubGlobal('document', {
        querySelector: vi.fn().mockReturnValue(null),
      })

      const fetchMock = vi.fn().mockResolvedValue({
        ok: true,
        text: () => Promise.resolve(''),
      })
      vi.stubGlobal('fetch', fetchMock)

      vi.stubGlobal(
        'DOMParser',
        class {
          parseFromString() {
            return { querySelectorAll: () => [] }
          }
        },
      )

      const { fetchTranscript } = await import('../lib/youtube')
      await fetchTranscript('https://www.youtube.com/api/timedtext?lang=en')

      // First call should NOT have fmt=xml — use whatever YouTube returns
      const firstUrl = fetchMock.mock.calls[0]?.[0] as string
      expect(firstUrl).not.toContain('fmt=xml')
    })

    it('returns empty array on fetch failure', async () => {
      vi.stubGlobal('document', {
        querySelector: vi.fn().mockReturnValue(null),
      })
      vi.stubGlobal('fetch', vi.fn().mockRejectedValue(new Error('Network error')))

      const { fetchTranscript } = await import('../lib/youtube')
      const result = await fetchTranscript('https://example.com/timedtext')
      expect(result).toEqual([])
    })

    it('returns empty array on non-ok response', async () => {
      vi.stubGlobal('document', {
        querySelector: vi.fn().mockReturnValue(null),
      })
      vi.stubGlobal('fetch', vi.fn().mockResolvedValue({ ok: false, status: 404 }))

      const { fetchTranscript } = await import('../lib/youtube')
      const result = await fetchTranscript('https://example.com/timedtext')
      expect(result).toEqual([])
    })

    it('retries with kind=asr when initial fetch returns no segments', async () => {
      const asrJson = JSON.stringify({
        wireMagic: 'pb3',
        events: [{ tStartMs: 0, dDurationMs: 3000, segs: [{ utf8: 'auto caption text' }] }],
      })

      vi.stubGlobal('document', {
        querySelector: vi.fn().mockReturnValue(null),
      })

      const fetchMock = vi
        .fn()
        // First call (default format): returns empty
        .mockResolvedValueOnce({
          ok: true,
          text: () => Promise.resolve(''),
        })
        // Second call (kind=asr): returns captions
        .mockResolvedValueOnce({
          ok: true,
          text: () => Promise.resolve(asrJson),
        })
      vi.stubGlobal('fetch', fetchMock)

      const { fetchTranscript } = await import('../lib/youtube')
      const result = await fetchTranscript('https://www.youtube.com/api/timedtext?v=abc&lang=en')

      expect(result).toEqual([{ startMs: 0, endMs: 3000, text: 'auto caption text' }])
      expect(fetchMock).toHaveBeenCalledTimes(2)
      const secondUrl = fetchMock.mock.calls[1]?.[0] as string
      expect(secondUrl).toContain('kind=asr')
    })

    it('scrapes from open transcript panel as fast-path', async () => {
      const domSegments = [{ textContent: 'Hello world' }, { textContent: 'Second segment' }]
      const transcriptRenderer = {
        querySelectorAll: vi.fn().mockReturnValue(domSegments),
      }

      vi.stubGlobal('document', {
        querySelector: vi.fn().mockImplementation((sel: string) => {
          if (sel === 'ytd-transcript-renderer') return transcriptRenderer
          return null
        }),
      })

      const fetchMock = vi.fn()
      vi.stubGlobal('fetch', fetchMock)

      const { fetchTranscript } = await import('../lib/youtube')
      const result = await fetchTranscript('https://example.com/timedtext')
      expect(result).toEqual([
        { startMs: 0, text: 'Hello world' },
        { startMs: 0, text: 'Second segment' },
      ])
      expect(fetchMock).not.toHaveBeenCalled()
    })
  })
})
