// @vitest-environment jsdom
import { describe, expect, it } from 'vitest'

import { nodePath } from '../lib/dom-range'

describe('DOM range paths', () => {
  it('distinguishes sibling text nodes under the same element', () => {
    const paragraph = document.createElement('p')
    const first = document.createTextNode('Repeated text')
    const second = document.createTextNode('Repeated text')
    paragraph.append(first, second)
    document.body.replaceChildren(paragraph)

    expect(nodePath(first)).not.toBe(nodePath(second))
    expect(nodePath(first)).toMatch(/\/0$/)
    expect(nodePath(second)).toMatch(/\/1$/)
  })
})
