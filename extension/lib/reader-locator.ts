import { buildTextIndex, resolveTextAnchor } from '../../shared/highlight-source'
import type { LocatorPayload } from '@/lib/api'

export interface ReaderLocatorInput {
  readableHtml: string
  text: string
  prefix?: string
  suffix?: string
}

function isReaderTextNode(node: Text): boolean {
  return node.parentElement?.closest('script, style, noscript, template') === null
}

export function resolveReaderLocator(
  input: ReaderLocatorInput,
  parser: DOMParser,
): LocatorPayload | undefined {
  const doc = parser.parseFromString(input.readableHtml, 'text/html')
  const index = buildTextIndex(doc.body, isReaderTextNode)
  const resolution = resolveTextAnchor(index.text, {
    text: input.text,
    context: { prefix: input.prefix, suffix: input.suffix },
  })
  if (resolution.kind !== 'placed' || resolution.end <= resolution.start) return undefined
  return { type: 'html', start_offset: resolution.start, end_offset: resolution.end }
}
