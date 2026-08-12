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
  const author = preferredArticleAuthor(sourceDoc, nonEmpty(result.author))

  return {
    readerHtml: result.content ?? '',
    excerpt: nonEmpty(result.description),
    author,
    language: nonEmpty(result.language),
    wordCount,
    readingTimeMinutes: wordCount ? Math.max(1, Math.ceil(wordCount / 200)) : undefined,
    publishedAt: nonEmpty(result.published),
    leadImageUrl,
  }
}

function preferredArticleAuthor(
  doc: Document,
  extractedAuthor: string | undefined,
): string | undefined {
  const person = structuredPersonCredit(doc)
  if (!person) return extractedAuthor

  const siteName = nonEmpty(
    doc.querySelector('meta[property="og:site_name"]')?.getAttribute('content') ?? undefined,
  )
  if (!extractedAuthor || (siteName && sameCredit(extractedAuthor, siteName))) return person
  return extractedAuthor
}

function structuredPersonCredit(doc: Document): string | undefined {
  for (const script of doc.querySelectorAll<HTMLScriptElement>(
    'script[type="application/ld+json"]',
  )) {
    let value: unknown
    try {
      value = JSON.parse(script.textContent ?? '')
    } catch {
      continue
    }
    for (const entity of structuredEntities(value)) {
      if (!isArticleEntity(entity)) continue
      for (const property of ['author', 'contributor'] as const) {
        const credits = Array.isArray(entity[property]) ? entity[property] : [entity[property]]
        for (const credit of credits) {
          if (!isRecord(credit) || !hasSchemaType(credit, 'Person')) continue
          const name = typeof credit['name'] === 'string' ? nonEmpty(credit['name']) : undefined
          if (name) return name
        }
      }
    }
  }
  return undefined
}

function structuredEntities(value: unknown): Record<string, unknown>[] {
  if (Array.isArray(value)) return value.flatMap(structuredEntities)
  if (!isRecord(value)) return []
  const graph = value['@graph']
  return graph === undefined ? [value] : [value, ...structuredEntities(graph)]
}

function isArticleEntity(value: Record<string, unknown>): boolean {
  return ['Article', 'NewsArticle', 'BlogPosting', 'HowTo'].some((type) =>
    hasSchemaType(value, type),
  )
}

function hasSchemaType(value: Record<string, unknown>, expected: string): boolean {
  const type = value['@type']
  return Array.isArray(type) ? type.includes(expected) : type === expected
}

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === 'object' && value !== null
}

function sameCredit(left: string, right: string): boolean {
  return left.trim().localeCompare(right.trim(), undefined, { sensitivity: 'base' }) === 0
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
