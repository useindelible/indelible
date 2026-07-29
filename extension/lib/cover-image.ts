/**
 * Cover/lead image extraction for a captured page. Precedence mirrors the reader:
 * og:image -> twitter:image -> the first substantial in-article <img>.
 */

const META_SELECTORS = [
  'meta[property="og:image"][content]',
  'meta[name="twitter:image"][content]',
  'meta[name="twitter:image:src"][content]',
] as const

const ARTICLE_CONTAINERS = [
  'article',
  '[role="main"]',
  'main',
  '.post-content',
  '.entry-content',
  '.article-body',
  'body',
] as const

export function extractCoverUrl(doc: Document): string | undefined {
  const metaImage = metaCoverImage(doc)
  // og:image/twitter:image are only trustworthy when the head metadata still describes the page.
  // Some SPAs (e.g. Instructables) swap article content on client-side navigation but leave
  // <head> og:image/canonical pointing at a previously-viewed page; trusting that stale og:image
  // attaches the wrong cover. When the declared identity disagrees with the actual URL, fall back
  // to an in-article image, which reflects the visible content.
  if (
    metaImage &&
    headMetadataDescribesCurrentPage(declaredIdentity(doc), currentUrl(doc), doc.baseURI)
  ) {
    return metaImage
  }
  return firstArticleImage(doc)
}

/**
 * True when the page's declared identity (rel=canonical, else og:url) resolves to its actual URL,
 * or when either is absent. A mismatch signals stale SPA head metadata, so og:image must not be
 * trusted. Host is compared ignoring a leading `www.` and paths ignoring a trailing slash.
 */
export function headMetadataDescribesCurrentPage(
  declared: string | undefined,
  current: string | undefined,
  base?: string,
): boolean {
  if (!declared || !current) return true
  try {
    return sameDocumentIdentity(new URL(declared, base), new URL(current))
  } catch {
    return true
  }
}

function declaredIdentity(doc: Document): string | undefined {
  return (
    doc.querySelector('link[rel~="canonical"][href]')?.getAttribute('href') ??
    doc.querySelector('meta[property="og:url"][content]')?.getAttribute('content') ??
    undefined
  )
}

function currentUrl(doc: Document): string | undefined {
  return doc.location?.href ?? doc.URL ?? undefined
}

function metaCoverImage(doc: Document): string | undefined {
  for (const selector of META_SELECTORS) {
    const content = doc.querySelector(selector)?.getAttribute('content')
    if (content?.startsWith('http')) return content
  }
  return undefined
}

function sameDocumentIdentity(a: URL, b: URL): boolean {
  const key = (u: URL) =>
    `${u.host.replace(/^www\./i, '')}${u.pathname.replace(/\/+$/, '')}`.toLowerCase()
  return key(a) === key(b)
}

function firstArticleImage(doc: Document): string | undefined {
  for (const container of ARTICLE_CONTAINERS) {
    const el = doc.querySelector(container)
    if (!el) continue

    const imgs = el.querySelectorAll<HTMLImageElement>('img[src]')
    for (const img of imgs) {
      const src = img.getAttribute('src') ?? ''
      if (!src.startsWith('http')) continue
      // Skip images with explicit tiny dimensions (tracking pixels, icons).
      if (img.width > 0 && img.width < 100) continue
      if (img.height > 0 && img.height < 100) continue
      return src
    }

    if (imgs.length > 0) break
  }

  return undefined
}
