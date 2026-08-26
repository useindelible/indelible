// @vitest-environment jsdom
import { describe, expect, it } from 'vitest'

import { resolveReaderLocator } from '../lib/reader-locator'

const html = `<article><h1>Zettelkasten</h1><p><a href="#">Antonin Sertillanges</a> ' book <i>The Intellectual Life</i> (1921) outlines.<sup id="ind-fnref:28"><a href="#ind-fn:28">28</a></sup></p><p>Later edited in a final form.<sup><a href="#">40</a></sup> French theorist.</p></article>`

describe('resolveReaderLocator', () => {
  const parser = new DOMParser()

  it('resolves live-page text with bracketed citations against readable html', () => {
    const loc = resolveReaderLocator(
      { readableHtml: html, text: 'Later edited in a final form.[35]' },
      parser,
    )
    expect(loc).toBeDefined()
    const body = parser.parseFromString(html, 'text/html').body.textContent!
    expect(body.slice(loc!.start_offset, loc!.end_offset)).toBe('Later edited in a final form.40')
  })

  it('resolves across a spaced apostrophe', () => {
    const loc = resolveReaderLocator(
      { readableHtml: html, text: "Sertillanges' book The Intellectual Life" },
      parser,
    )
    expect(loc).toBeDefined()
    expect(loc!.end_offset).toBeGreaterThan(loc!.start_offset)
  })

  it('returns undefined for page chrome that is not in the article', () => {
    expect(
      resolveReaderLocator({ readableHtml: html, text: 'Article Talk Read Edit' }, parser),
    ).toBeUndefined()
  })
})
