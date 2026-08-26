// @vitest-environment jsdom
import { beforeEach, describe, expect, it, vi } from 'vitest'

import {
  clearProjectedHighlights,
  projectHighlights,
  type ProjectedHighlight,
} from '../lib/highlight-projection'

describe('highlight projection', () => {
  beforeEach(() => {
    document.head.innerHTML = ''
    document.body.innerHTML = ''
  })

  it('projects a highlight from a parent-element DOM location', () => {
    const debug = vi.spyOn(console, 'debug').mockImplementation(() => {})
    document.body.innerHTML = '<p>Hello brave world.</p>'

    const highlight: ProjectedHighlight = {
      id: 'h_1',
      color: 'yellow',
      text_content: 'brave',
      source_locator: {
        type: 'web_page_dom_range',
        url: 'https://example.com/article',
        location: '1/0:6,1/0:11',
        offset: 6,
        text_content: 'brave',
      },
    }

    expect(projectHighlights([highlight], document)).toEqual({ placed: 1, unplaced: 0 })

    const mark = document.querySelector('mark.indelible-projected-highlight')
    expect(mark?.textContent).toBe('brave')
    expect(document.body.textContent).toBe('Hello brave world.')
    expect(debug).not.toHaveBeenCalled()
    debug.mockRestore()
  })

  it('covers overlapping highlights fully by nesting marks', () => {
    document.body.innerHTML = '<p>abcdefghij</p>'

    const result = projectHighlights(
      [
        { id: 'a', text_content: 'cdefgh' },
        { id: 'b', text_content: 'fghi' },
      ],
      document,
    )

    expect(result).toEqual({ placed: 2, unplaced: 0 })
    const byId = (id: string) =>
      Array.from(document.querySelectorAll(`mark[data-indelible-highlight-id="${id}"]`))
        .map((mark) => mark.textContent)
        .join('')
    expect(byId('a')).toBe('cdefgh')
    expect(byId('b')).toBe('fghi')
    expect(document.body.textContent).toBe('abcdefghij')
  })

  it('uses source offset to choose between repeated text matches', () => {
    document.body.innerHTML = `<p>Alpha target.</p><p>${'filler text. '.repeat(10)}</p><p>Beta target.</p>`
    const secondTargetOffset = document.body.textContent?.lastIndexOf('target') ?? 0

    const highlight: ProjectedHighlight = {
      id: 'h_2',
      text_content: 'target',
      source_locator: {
        type: 'web_page_dom_range',
        url: 'https://example.com/article',
        location: '9/9:0,9/9:6',
        offset: secondTargetOffset,
        text_content: 'target',
      },
    }

    expect(projectHighlights([highlight], document)).toEqual({ placed: 1, unplaced: 0 })

    const paragraphs = Array.from(document.querySelectorAll('p'))
    expect(paragraphs[0]?.querySelector('mark')).toBeNull()
    expect(paragraphs[2]?.querySelector('mark')?.textContent).toBe('target')
  })

  it('falls back from a stale source locator and matches normalized repeated text context', () => {
    document.body.innerHTML = '<p>Alpha target   phrase.</p><p>Beta target   phrase.</p>'

    const highlight: ProjectedHighlight = {
      id: 'h_4',
      text_content: 'target phrase',
      source_locator: {
        type: 'web_page_dom_range',
        url: 'https://example.com/article',
        location: '99/0:0,99/0:13',
        offset: document.body.textContent?.lastIndexOf('target') ?? 0,
        text_content: 'target phrase',
        prefix: 'Beta',
        suffix: '.',
      },
    }

    expect(projectHighlights([highlight], document)).toEqual({ placed: 1, unplaced: 0 })

    const paragraphs = Array.from(document.querySelectorAll('p'))
    expect(paragraphs[0]?.querySelector('mark')).toBeNull()
    expect(paragraphs[1]?.querySelector('mark')?.textContent).toBe('target   phrase')
  })

  it('clears projected marks without changing page text', () => {
    document.body.innerHTML = '<p>Hello brave world.</p>'
    projectHighlights([{ id: 'h_3', text_content: 'brave' }], document)

    clearProjectedHighlights(document)

    expect(document.querySelector('mark.indelible-projected-highlight')).toBeNull()
    expect(document.body.innerHTML).toBe('<p>Hello brave world.</p>')
  })

  it('projects a reader-created highlight whose citation numbering differs from the page', () => {
    document.body.innerHTML =
      '<p>Paxson filed notes daily.<sup>[34]</sup> The notes were ordered.</p>'

    const result = projectHighlights(
      [{ id: 'h_5', text_content: 'Paxson filed notes daily.39 The notes were ordered.' }],
      document,
    )

    expect(result).toEqual({ placed: 1, unplaced: 0 })
    expect(document.querySelector('mark')?.textContent).toContain('Paxson filed notes daily.')
  })

  it('counts an unhinted repeated phrase as unplaced instead of guessing', () => {
    document.body.innerHTML = '<p>Alpha target.</p><p>Beta target.</p>'

    expect(projectHighlights([{ id: 'h_6', text_content: 'target' }], document)).toEqual({
      placed: 0,
      unplaced: 1,
    })
    expect(document.querySelector('mark')).toBeNull()
  })

  it('keeps earlier highlights intact when several share one text node', () => {
    document.body.innerHTML = '<p>one two three four five</p>'

    const result = projectHighlights(
      [
        { id: 'a', text_content: 'one' },
        { id: 'b', text_content: 'three' },
        { id: 'c', text_content: 'five' },
      ],
      document,
    )

    expect(result).toEqual({ placed: 3, unplaced: 0 })
    expect(Array.from(document.querySelectorAll('mark')).map((mark) => mark.textContent)).toEqual([
      'one',
      'three',
      'five',
    ])
    expect(document.body.textContent).toBe('one two three four five')
  })
})
