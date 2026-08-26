import { encodeBase64Utf8 } from '@/lib/archive'
import type { CaptureMessage, CapturePayload } from '@/lib/capture'
import { getPageData } from 'single-file-core/single-file.js'
import {
  isYouTubePage,
  extractYouTubeData,
  fetchTranscript,
  buildYouTubeReaderHtml,
  type TranscriptSegment,
} from '@/lib/youtube'
import { classifyExtensionUrl } from '@/lib/content-type'
import { extractReadableContent } from '@/lib/readable-extraction'
import { beginCaptureDomCleanup } from '@/lib/dom-preprocessor'
import { nodePath } from '@/lib/dom-range'
import { extractCoverUrl } from '@/lib/cover-image'
import { renderToolbar } from '@/lib/full-archive-toolbar'
import { escapeAttr, escapeHtml } from '@/lib/html'
import { resolveReaderLocator, type ReaderLocatorInput } from '@/lib/reader-locator'

function captureError(err: unknown): CaptureMessage {
  return { action: 'capture:error', message: err instanceof Error ? err.message : String(err) }
}

export default defineContentScript({
  registration: 'runtime',
  runAt: 'document_idle',

  async main() {
    const globalWindow = window as typeof window & {
      __indelibleFullArchiveContentScriptInstalled?: boolean
    }
    if (globalWindow.__indelibleFullArchiveContentScriptInstalled) return
    globalWindow.__indelibleFullArchiveContentScriptInstalled = true

    browser.runtime.onMessage.addListener(
      (
        message: unknown,
        _sender,
        sendResponse: (response: unknown) => void,
      ): boolean | undefined => {
        const record =
          typeof message === 'object' && message !== null
            ? (message as Record<string, unknown>)
            : undefined
        switch (record?.action) {
          case 'indelible:ping':
            sendResponse({ success: true })
            return true
          case 'toolbar:render':
            renderToolbar(record.state)
            sendResponse({ success: true })
            return true
          case 'capture:run':
            runCapture()
              .then(sendResponse)
              .catch((err: unknown) => sendResponse(captureError(err)))
            return true
          case 'selection:capture':
            try {
              sendResponse(captureSelection())
            } catch (err) {
              sendResponse(captureError(err))
            }
            return true
          case 'locator:resolve':
            sendResponse({
              action: 'locator:result',
              locator: resolveReaderLocator(record.payload as ReaderLocatorInput, new DOMParser()),
            } satisfies CaptureMessage)
            return true
          default:
            return undefined
        }
      },
    )
  },
})

async function runCapture(): Promise<CaptureMessage> {
  // Step 1: thumbnail fires in background before this content script is invoked,
  // so we skip it here and report progress for the steps we own.

  void browser.runtime.sendMessage({
    action: 'capture:progress',
    step: 'extracting',
  } satisfies CaptureMessage)

  let readerHtml = ''
  let excerpt: string | undefined
  let author: string | undefined
  let language: string | undefined
  let wordCount: number | undefined
  let readingTimeMinutes: number | undefined
  let publishedAt: string | undefined
  let leadImageUrl: string | undefined
  let itemType: string | undefined
  let videoDurationSeconds: number | undefined
  const canonicalUrl = extractCanonicalPageUrl()
  const classified = classifyExtensionUrl(document.location.href)
  itemType = classified.itemType

  if (isYouTubePage()) {
    const ytData = await extractYouTubeData()
    if (ytData) {
      let segments: TranscriptSegment[] = []
      if (ytData.captionTrackUrl) {
        segments = await fetchTranscript(ytData.captionTrackUrl)
      }
      readerHtml = buildYouTubeReaderHtml({
        videoId: ytData.videoId,
        description: ytData.description,
        channelName: ytData.channelName,
        viewCount: ytData.viewCount,
        durationSeconds: ytData.durationSeconds,
        segments,
      })
      author = ytData.channelName || undefined
      excerpt = ytData.description ? ytData.description.slice(0, 300) : undefined
      leadImageUrl = ytData.thumbnailUrl
      itemType = 'video'
      videoDurationSeconds = ytData.durationSeconds
    }
  } else if (classified.itemType === 'pdf') {
    readerHtml = buildPdfReaderHtml(document.location.href, document.title)
    excerpt = 'PDF document'
  } else if (classified.itemType === 'tweet') {
    const tweet = extractTweetContent()
    readerHtml = buildTweetReaderHtml(tweet)
    excerpt = tweet.text.slice(0, 300) || undefined
    author = tweet.author
    leadImageUrl = tweet.imageUrl
  } else if (classified.itemType === 'video') {
    const video = extractVideoMetadata()
    readerHtml = buildVideoReaderHtml(video)
    excerpt = video.description?.slice(0, 300)
    author = video.author
    leadImageUrl = video.imageUrl
  } else {
    try {
      const extraction = extractReadableContent(document)
      readerHtml = extraction.readerHtml
      excerpt = extraction.excerpt
      author = extraction.author
      language = extraction.language
      wordCount = extraction.wordCount
      readingTimeMinutes = extraction.readingTimeMinutes
      publishedAt = extraction.publishedAt
      leadImageUrl = extraction.leadImageUrl
    } catch {
      // Generic extraction failure is non-fatal — proceed with empty reader HTML.
    }
  }

  void browser.runtime.sendMessage({
    action: 'capture:progress',
    step: 'singlefile',
  } satisfies CaptureMessage)

  let htmlBase64 = ''
  let singleFileFailed = false
  const captureCleanup = beginCaptureDomCleanup(document, 'temporary')
  try {
    const pageData = await getPageData({
      removeHiddenElements: true,
      removeUnusedStyles: true,
      removeUnusedFonts: true,
      removeImports: true,
      blockScripts: true,
      blockAudios: true,
      blockVideos: true,
      compressHTML: false,
      removeAlternativeFonts: true,
      removeAlternativeMedias: true,
      // Leaving this false causes SingleFile to fetch every srcset/<picture> candidate
      // for each image (webp, avif, 2x, 3x…). On image-heavy pages this was the dominant
      // bottleneck — capturing a novel chapter took minutes instead of seconds.
      removeAlternativeImages: true,
      // Dedup requires hashing all image blobs; not worth the cost for archive fidelity.
      groupDuplicateImages: false,
      // Skip individual resources larger than 5 MB to avoid one hero image stalling the whole capture.
      maxResourceSizeEnabled: true,
      maxResourceSize: 5,
      // Iframes (ads, embeds) trigger a full recursive SingleFile pass each — skip them.
      removeFrames: true,
    })
    const rawSize = new Blob([pageData.content]).size
    const MAX_MONOLITH_BYTES = 200 * 1024 * 1024
    if (rawSize > MAX_MONOLITH_BYTES) {
      return {
        action: 'capture:error',
        message: 'monolith-too-large',
        payload: {
          url: document.location.href,
          canonicalUrl,
          title: document.title,
          readerHtml,
          htmlBase64: '',
        },
      }
    }
    htmlBase64 = encodeBase64Utf8(pageData.content)
  } catch {
    singleFileFailed = true
  } finally {
    captureCleanup.restore()
  }

  const payload: CapturePayload = {
    url: document.location.href,
    canonicalUrl,
    title: document.title,
    readerHtml,
    htmlBase64,
    leadImageUrl,
    excerpt,
    author,
    language,
    wordCount,
    readingTimeMinutes,
    publishedAt,
    itemType,
    videoDurationSeconds,
  }

  if (singleFileFailed) {
    return {
      action: 'capture:error',
      message: 'singlefile-failed',
      payload: { ...payload, htmlBase64: '' },
    }
  }

  return {
    action: 'capture:result',
    payload,
  }
}

function extractCanonicalPageUrl(): string | undefined {
  const canonical = document.querySelector<HTMLLinkElement>('link[rel~="canonical"][href]')?.href
  if (canonical) return canonical
  const ogUrl = document.querySelector<HTMLMetaElement>('meta[property="og:url"][content]')?.content
  return ogUrl || undefined
}

function captureSelection(): CaptureMessage {
  const selection = window.getSelection()
  if (!selection || selection.rangeCount === 0 || selection.toString().trim().length === 0) {
    return { action: 'capture:error', message: 'No selected text found' }
  }

  const range = selection.getRangeAt(0)
  const text = selection.toString().trim()
  const offset = flattenedTextOffset(range.startContainer, range.startOffset)
  const location = `${nodePath(range.startContainer)}:${range.startOffset},${nodePath(
    range.endContainer,
  )}:${range.endOffset}`
  const context = selectionContext(text, offset)

  return {
    action: 'selection:result',
    payload: {
      text,
      sourceLocator: {
        type: 'web_page_dom_range',
        url: document.location.href,
        location,
        offset,
        text_content: text,
        prefix: context.prefix,
        suffix: context.suffix,
      },
    },
  }
}

function flattenedTextOffset(startNode: Node, startOffset: number): number {
  const walker = document.createTreeWalker(document.body, NodeFilter.SHOW_TEXT)
  let offset = 0
  let node: Node | null = walker.nextNode()
  while (node) {
    if (node === startNode) return offset + startOffset
    offset += node.textContent?.length ?? 0
    node = walker.nextNode()
  }
  return offset
}

function selectionContext(text: string, offset: number): { prefix?: string; suffix?: string } {
  const allText = document.body?.textContent ?? ''
  const start = Math.max(0, offset - 80)
  const end = Math.min(allText.length, offset + text.length + 80)
  const prefix = allText.slice(start, offset).trim()
  const suffix = allText.slice(offset + text.length, end).trim()
  return {
    prefix: prefix || undefined,
    suffix: suffix || undefined,
  }
}

function extractTweetContent(): {
  text: string
  author?: string
  imageUrl?: string
} {
  const article = document.querySelector('article')
  const text = article?.textContent?.trim() || document.title
  const author =
    article?.querySelector('[data-testid="User-Name"]')?.textContent?.trim() ||
    document.querySelector('meta[name="author"]')?.getAttribute('content') ||
    undefined
  const imageUrl =
    article?.querySelector<HTMLImageElement>('img[src^="https://pbs.twimg.com/media/"]')?.src ||
    extractCoverUrl(document)
  return { text, author, imageUrl }
}

function buildTweetReaderHtml(tweet: { text: string; author?: string; imageUrl?: string }): string {
  const image = tweet.imageUrl ? `<img src="${escapeAttr(tweet.imageUrl)}" alt="" />` : ''
  const author = tweet.author ? `<p><strong>${escapeHtml(tweet.author)}</strong></p>` : ''
  return `<article><h1>${escapeHtml(document.title || 'Tweet')}</h1>${author}<p>${escapeHtml(
    tweet.text,
  )}</p>${image}</article>`
}

function extractVideoMetadata(): {
  title: string
  description?: string
  author?: string
  imageUrl?: string
} {
  return {
    title:
      document.querySelector('meta[property="og:title"]')?.getAttribute('content') ||
      document.title ||
      'Video',
    description:
      document.querySelector('meta[property="og:description"]')?.getAttribute('content') ||
      document.querySelector('meta[name="description"]')?.getAttribute('content') ||
      undefined,
    author: document.querySelector('meta[name="author"]')?.getAttribute('content') || undefined,
    imageUrl: extractCoverUrl(document),
  }
}

function buildVideoReaderHtml(video: {
  title: string
  description?: string
  author?: string
  imageUrl?: string
}): string {
  const image = video.imageUrl ? `<img src="${escapeAttr(video.imageUrl)}" alt="" />` : ''
  const author = video.author ? `<p><strong>${escapeHtml(video.author)}</strong></p>` : ''
  const description = video.description ? `<p>${escapeHtml(video.description)}</p>` : ''
  return `<article><h1>${escapeHtml(video.title)}</h1>${author}${description}${image}</article>`
}

function buildPdfReaderHtml(url: string, title: string): string {
  return `<article><h1>${escapeHtml(title || 'PDF')}</h1><p><a href="${escapeAttr(
    url,
  )}">Open source PDF</a></p></article>`
}
