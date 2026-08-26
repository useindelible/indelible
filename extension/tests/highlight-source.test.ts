import { describe, expect, it } from 'vitest'

import {
  findBestTextMatch,
  normalizeForMatch,
  resolveTextAnchor,
} from '../../shared/highlight-source'

describe('normalizeForMatch', () => {
  it('collapses whitespace and keeps span maps', () => {
    const r = normalizeForMatch('a  b\n\nc')
    expect(r.text).toBe('a b c')
    expect(r.starts).toEqual([0, 1, 3, 4, 6])
    expect(r.ends).toEqual([1, 3, 4, 6, 7])
  })

  it('drops bracketed citations and attributes them to the preceding character', () => {
    const r = normalizeForMatch('form.[35] French')
    expect(r.text).toBe('form. French')
    expect(r.ends[4]).toBe('form.[35]'.length)
  })

  it('drops bare renumbered citations glued to sentence punctuation', () => {
    const r = normalizeForMatch('drawers.39 The notes')
    expect(r.text).toBe('drawers. The notes')
    expect(r.ends[7]).toBe('drawers.39'.length)
  })

  it('drops a mid-sentence bare citation the same way as a bracketed one', () => {
    expect(normalizeForMatch('final form.39 french theorist').text).toBe(
      'final form. french theorist',
    )
    expect(normalizeForMatch('final form.[35] french theorist').text).toBe(
      'final form. french theorist',
    )
  })

  it('keeps decimals, versions, section numbers and unglued numbers', () => {
    expect(normalizeForMatch('version 3.14 released').text).toBe('version 3.14 released')
    expect(normalizeForMatch('Section 2.3 covers').text).toBe('Section 2.3 covers')
    expect(normalizeForMatch('pp. 12 and').text).toBe('pp. 12 and')
    expect(normalizeForMatch('in 1980.').text).toBe('in 1980.')
    expect(normalizeForMatch('Call 911. Then').text).toBe('Call 911. Then')
  })

  it('removes a space before an apostrophe and folds quotes and dashes', () => {
    expect(normalizeForMatch("Sertillanges ' book").text).toBe("Sertillanges' book")
    expect(normalizeForMatch('“Card file” – it’s').text).toBe(`"Card file" - it's`)
  })
})

describe('findBestTextMatch / resolveTextAnchor', () => {
  const page = 'Intro. Paxson filed notes daily.[34] The notes were ordered. End.'

  it('matches reader text with a renumbered bare citation and covers the page citation', () => {
    const m = findBestTextMatch(page, 'Paxson filed notes daily.39 The notes were ordered.')!
    expect(page.slice(m.start, m.end)).toBe('Paxson filed notes daily.[34] The notes were ordered.')
  })

  it('matches across an inserted space before an apostrophe', () => {
    const src = "Antonin Sertillanges ' book The Life"
    const m = findBestTextMatch(src, "Sertillanges' book")!
    expect(src.slice(m.start, m.end)).toBe("Sertillanges ' book")
  })

  it('uses context to pick among repeats', () => {
    const src = 'Alpha target. Beta target.'
    expect(findBestTextMatch(src, 'target', { prefix: 'Beta', suffix: '.' })!.start).toBe(
      src.lastIndexOf('target'),
    )
  })

  it('context outranks a badly drifted offset', () => {
    const src = 'Alpha target. ' + 'filler text. '.repeat(200) + 'Beta target.'
    const m = findBestTextMatch(src, 'target', { offset: 6, prefix: 'Beta', suffix: '.' })!
    expect(m.start).toBe(src.lastIndexOf('target'))
  })

  it('uses the nearest offset only when the runner-up is clearly farther', () => {
    const far = 'Alpha target. ' + 'filler text. '.repeat(10) + 'Beta target.'
    expect(findBestTextMatch(far, 'target', { offset: far.lastIndexOf('target') - 5 })!.start).toBe(
      far.lastIndexOf('target'),
    )
    expect(
      findBestTextMatch('Alpha target. Beta target.', 'target', { offset: 20 }),
    ).toBeUndefined()
  })

  it('refuses to guess between unhinted repeats', () => {
    expect(findBestTextMatch('Alpha target. Beta target.', 'target')).toBeUndefined()
    expect(resolveTextAnchor('Alpha target. Beta target.', { text: 'target' })).toEqual({
      kind: 'ambiguous',
    })
  })

  it('accepts a hint whose text still matches, falls back when it does not', () => {
    const src = 'Alpha target. Beta target.'
    expect(
      resolveTextAnchor(src, { text: 'Beta target', hint: { start: 14, end: 25 } }),
    ).toMatchObject({ kind: 'placed', via: 'hint' })
    expect(
      resolveTextAnchor(src, { text: 'Beta target', hint: { start: 0, end: 11 } }),
    ).toMatchObject({ kind: 'placed', via: 'search', start: 14 })
    expect(resolveTextAnchor(src, { text: 'absent' })).toEqual({ kind: 'missing' })
  })
})
