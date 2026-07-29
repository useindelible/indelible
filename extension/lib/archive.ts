// btoa() rejects code points > 255, so we re-encode through UTF-8 bytes first.
export function encodeBase64Utf8(str: string): string {
  const bytes = new TextEncoder().encode(str)
  let binary = ''
  for (const byte of bytes) {
    binary += String.fromCharCode(byte)
  }
  return btoa(binary)
}

export function stripDataUriPrefix(dataUri: string): string {
  const commaIndex = dataUri.indexOf(',')
  if (commaIndex === -1) return dataUri
  return dataUri.slice(commaIndex + 1)
}

export interface FullArchiveRequestBody {
  url: string
  canonical_url?: string
  title: string
  reader_html: string
  html_base64: string
  lead_image_url?: string
  excerpt?: string
  author?: string
  language?: string
  published_at?: string
  item_type?: string
}

export function buildFullArchiveBody(
  url: string,
  canonicalUrl: string | undefined,
  title: string,
  readerHtml: string,
  htmlBase64: string,
  leadImageUrl?: string,
  excerpt?: string,
  author?: string,
  language?: string,
  publishedAt?: string,
  itemType?: string,
): FullArchiveRequestBody {
  const body: FullArchiveRequestBody = {
    url,
    title,
    reader_html: readerHtml,
    html_base64: htmlBase64,
  }
  if (canonicalUrl !== undefined) body.canonical_url = canonicalUrl
  if (leadImageUrl !== undefined) body.lead_image_url = leadImageUrl
  if (excerpt !== undefined) body.excerpt = excerpt
  if (author !== undefined) body.author = author
  if (language !== undefined) body.language = language
  if (publishedAt !== undefined) body.published_at = publishedAt
  if (itemType !== undefined) body.item_type = itemType
  return body
}

export interface ReaderSaveFallbackBody {
  url: string
  canonical_url?: string
  title: string
  reader_html: string
  lead_image_url?: string
  excerpt?: string
  author?: string
  item_type?: string
}

export function buildReaderSaveFallbackBody(
  url: string,
  canonicalUrl: string | undefined,
  title: string,
  readerHtml: string,
  leadImageUrl?: string,
  excerpt?: string,
  author?: string,
  itemType?: string,
): ReaderSaveFallbackBody {
  const body: ReaderSaveFallbackBody = { url, title, reader_html: readerHtml }
  if (canonicalUrl !== undefined) {
    body.canonical_url = canonicalUrl
  }
  if (leadImageUrl !== undefined) {
    body.lead_image_url = leadImageUrl
  }
  if (excerpt !== undefined) body.excerpt = excerpt
  if (author !== undefined) body.author = author
  if (itemType !== undefined) body.item_type = itemType
  return body
}
