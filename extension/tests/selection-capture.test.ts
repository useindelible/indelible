// @vitest-environment jsdom
import { beforeEach, describe, expect, it } from 'vitest'

import { projectHighlights } from '../lib/highlight-projection'
import { captureSelection } from '../lib/selection-capture'

function select(node: Text, start: number, end: number): void {
  const range = document.createRange()
  range.setStart(node, start)
  range.setEnd(node, end)
  const selection = window.getSelection()!
  selection.removeAllRanges()
  selection.addRange(range)
}

function lastTextNode(el: Element): Text {
  return el.lastChild as Text
}

describe('captureSelection', () => {
  beforeEach(() => {
    document.body.innerHTML = ''
    window.getSelection()?.removeAllRanges()
  })

  it('describes the location on the unprojected page even when marks are present', () => {
    document.body.innerHTML = '<p>Hello brave world.</p>'
    const p = document.querySelector('p')!
    const textNode = p.firstChild as Text
    select(textNode, 12, 17)
    const pristine = captureSelection(document)

    document.body.innerHTML = '<p>Hello brave world.</p>'
    projectHighlights([{ id: 'h', text_content: 'brave' }], document)
    const worldNode = lastTextNode(document.querySelector('p')!)
    select(worldNode, worldNode.data.indexOf('world'), worldNode.data.indexOf('world') + 5)
    const projected = captureSelection(document)

    expect(projected.action).toBe('selection:result')
    if (pristine.action !== 'selection:result' || projected.action !== 'selection:result') return
    expect(projected.payload.sourceLocator.location).toBe(pristine.payload.sourceLocator.location)
    expect(projected.payload.sourceLocator.offset).toBe(12)
    expect(document.querySelector('mark')).toBeNull()
  })

  it('ignores script text for offsets and context', () => {
    document.body.innerHTML = '<script>var x = "zzzz";</script><p>Alpha beta.</p>'
    const textNode = document.querySelector('p')!.firstChild as Text
    select(textNode, 6, 10)

    const result = captureSelection(document)

    expect(result.action).toBe('selection:result')
    if (result.action !== 'selection:result') return
    expect(result.payload.sourceLocator.offset).toBe(6)
    expect(result.payload.sourceLocator.prefix).toBe('Alpha')
    expect(result.payload.sourceLocator.suffix).toBe('.')
  })

  it('trims surrounding whitespace out of the text and offsets', () => {
    document.body.innerHTML = '<p>one  two three</p>'
    const textNode = document.querySelector('p')!.firstChild as Text
    select(textNode, 3, 9)

    const result = captureSelection(document)

    expect(result.action).toBe('selection:result')
    if (result.action !== 'selection:result') return
    expect(result.payload.text).toBe('two')
    expect(result.payload.sourceLocator.offset).toBe(5)
    expect(result.payload.sourceLocator.location).toBe('1/0/0:5,1/0/0:8')
  })
})
