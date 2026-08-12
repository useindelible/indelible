// @vitest-environment jsdom
import { describe, expect, it } from 'vitest'

import { extractReadableContent } from '../lib/readable-extraction'

describe('readable extraction', () => {
  it('maps Defuddle content and metadata into the capture payload shape', () => {
    const doc = parseHtml(
      `<!doctype html>
      <html lang="en">
        <head>
          <title>Defuddle Metadata Test</title>
          <meta name="author" content="Ada Lovelace">
          <meta name="description" content="A compact summary of the article.">
          <meta property="article:published_time" content="2026-05-20T12:00:00Z">
          <meta property="og:image" content="https://cdn.example.com/cover.jpg">
        </head>
        <body>
          <main>
            <article>
              <h1>Defuddle Metadata Test</h1>
              <p>${articleParagraph.repeat(8)}</p>
              <p>${articleParagraph.repeat(8)}</p>
              <p>${articleParagraph.repeat(8)}</p>
            </article>
          </main>
        </body>
      </html>`,
    )

    const result = extractReadableContent(doc)

    expect(result.readerHtml).toContain('durable readable extraction')
    expect(result.author).toBe('Ada Lovelace')
    expect(result.excerpt).toBe('A compact summary of the article.')
    expect(result.language).toBe('en')
    expect(result.publishedAt).toBe('2026-05-20T12:00:00Z')
    expect(result.leadImageUrl).toBe('https://cdn.example.com/cover.jpg')
    expect(result.wordCount).toBeGreaterThan(50)
    expect(result.readingTimeMinutes).toBeGreaterThanOrEqual(1)
  })

  it('preprocesses consent banners before Defuddle extracts content', () => {
    const doc = parseHtml(
      `<!doctype html>
      <html>
        <body>
          <div class="privacy-modal">
            <h2>Cookie preferences</h2>
            <p>Your privacy is important to us and so is an optimal experience.</p>
            <p>Third Party Services include Google Analytics and Advertising.</p>
          </div>
          <main>
            <article>
              <h1>Workshop Project</h1>
              <p>${articleParagraph.repeat(8)}</p>
              <p>${articleParagraph.repeat(8)}</p>
              <p>${articleParagraph.repeat(8)}</p>
            </article>
          </main>
        </body>
      </html>`,
    )

    const result = extractReadableContent(doc)

    expect(result.readerHtml).toContain('durable readable extraction')
    expect(result.readerHtml).not.toContain('Cookie preferences')
    expect(result.readerHtml).not.toContain('Google Analytics')
  })

  it('prefers a person contributor when structured author metadata names the site', () => {
    const doc = parseHtml(
      `<!doctype html>
      <html>
        <head>
          <title>Workshop Project - Example Maker Site</title>
          <meta property="og:site_name" content="Example Maker Site">
          <script type="application/ld+json">
            {
              "@context": "https://schema.org",
              "@type": "Article",
              "headline": "Workshop Project",
              "author": { "@type": "Organization", "name": "Example Maker Site" },
              "publisher": { "@type": "Organization", "name": "Example Maker Site" },
              "contributor": { "@type": "Person", "name": "AahanSharma" }
            }
          </script>
        </head>
        <body>
          <main>
            <article>
              <h1>Workshop Project</h1>
              <div class="article-byline">By AahanSharma</div>
              <p>${articleParagraph.repeat(24)}</p>
            </article>
          </main>
        </body>
      </html>`,
    )

    const result = extractReadableContent(doc)

    expect(result.author).toBe('AahanSharma')
  })
})

const articleParagraph =
  'This durable readable extraction article explains how to capture complex web pages, preserve primary article paragraphs, and remove distracting interface chrome without losing meaningful body content. '

function parseHtml(html: string): Document {
  return new DOMParser().parseFromString(html, 'text/html')
}
