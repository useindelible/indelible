import Defuddle from 'defuddle'

import { extractCoverUrl } from './cover-image'
import { preprocessDocumentForReadableExtraction } from './dom-preprocessor'

export interface ReadableExtractionResult {
  readerHtml: string
  excerpt?: string
  author?: string
  language?: string
  wordCount?: number
  readingTimeMinutes?: number
  publishedAt?: string
  leadImageUrl?: string
}

export function extractReadableContent(sourceDoc: Document): ReadableExtractionResult {
  const docClone = sourceDoc.cloneNode(true) as Document
  preprocessDocumentForReadableExtraction(docClone)

  const result = new Defuddle(docClone, {
    url: sourceUrl(sourceDoc),
    useAsync: false,
  }).parse()

  const wordCount = positiveNumber(result.wordCount)
  const leadImageUrl = nonEmpty(result.image) ?? extractCoverUrl(sourceDoc)

  return {
    readerHtml: result.content ?? '',
    excerpt: nonEmpty(result.description),
    author: nonEmpty(result.author),
    language: nonEmpty(result.language),
    wordCount,
    readingTimeMinutes: wordCount ? Math.max(1, Math.ceil(wordCount / 200)) : undefined,
    publishedAt: nonEmpty(result.published),
    leadImageUrl,
  }
}

function nonEmpty(value: string | undefined): string | undefined {
  const trimmed = value?.trim()
  return trimmed ? trimmed : undefined
}

function positiveNumber(value: number | undefined): number | undefined {
  return typeof value === 'number' && Number.isFinite(value) && value > 0 ? value : undefined
}

function sourceUrl(doc: Document): string {
  return doc.location?.href ?? doc.URL ?? doc.baseURI
}
