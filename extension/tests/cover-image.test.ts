// @vitest-environment jsdom
import { describe, expect, it } from 'vitest'

import { extractCoverUrl, headMetadataDescribesCurrentPage } from '../lib/cover-image'

function parseHtml(html: string): Document {
  return new DOMParser().parseFromString(html, 'text/html')
}

describe('headMetadataDescribesCurrentPage', () => {
  it('matches identical canonical and current URLs', () => {
    expect(
      headMetadataDescribesCurrentPage('https://www.example.com/a', 'https://www.example.com/a'),
    ).toBe(true)
  })

  it('ignores www and trailing-slash differences', () => {
    expect(
      headMetadataDescribesCurrentPage('https://example.com/a/', 'https://www.example.com/a'),
    ).toBe(true)
  })

  it('detects a different path as stale', () => {
    expect(
      headMetadataDescribesCurrentPage('https://www.example.com/a', 'https://www.example.com/b'),
    ).toBe(false)
  })

  it('assumes fresh when no identity is declared', () => {
    expect(headMetadataDescribesCurrentPage(undefined, 'https://www.example.com/b')).toBe(true)
  })
})

describe('extractCoverUrl', () => {
  it('uses og:image when no conflicting page identity is declared', () => {
    const doc = parseHtml(
      '<head><meta property="og:image" content="https://cdn.example.com/a.jpg"></head>' +
        '<body><article><img src="https://cdn.example.com/inline.jpg"></article></body>',
    )
    expect(extractCoverUrl(doc)).toBe('https://cdn.example.com/a.jpg')
  })

  it('ignores og:image when the declared canonical does not match the page URL', () => {
    const doc = parseHtml(
      '<head><link rel="canonical" href="https://www.example.com/a-different-article">' +
        '<meta property="og:image" content="https://cdn.example.com/stale.jpg"></head>' +
        '<body><article><img src="https://cdn.example.com/real.jpg"></article></body>',
    )
    expect(extractCoverUrl(doc)).toBe('https://cdn.example.com/real.jpg')
  })

  it('falls back to the first in-article image when no meta image exists', () => {
    const doc = parseHtml(
      '<body><article><p>intro</p><img src="https://cdn.example.com/f.jpg"></article></body>',
    )
    expect(extractCoverUrl(doc)).toBe('https://cdn.example.com/f.jpg')
  })

  it('returns undefined when there is no usable image', () => {
    const doc = parseHtml('<body><article><p>no images here</p></article></body>')
    expect(extractCoverUrl(doc)).toBeUndefined()
  })
})
