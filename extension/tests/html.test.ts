import { describe, expect, it } from 'vitest'

import { escapeAttr, escapeHtml, safeHttpUrl } from '../lib/html'

describe('escapeHtml', () => {
  it('uses one stable entity contract for text and attribute interpolation', () => {
    expect(escapeHtml(`Rock & Roll <"Sam's">`)).toBe(
      'Rock &amp; Roll &lt;&quot;Sam&#39;s&quot;&gt;',
    )
  })

  it('keeps attribute escaping and URL scheme validation centralized', () => {
    expect(escapeAttr('https://example.com/?a=1&b=2')).toBe('https://example.com/?a=1&amp;b=2')
    expect(safeHttpUrl('https://example.com/?a=1&b=2')).toBe('https://example.com/?a=1&amp;b=2')
    expect(safeHttpUrl('javascript:alert(1)')).toBe('#')
    expect(safeHttpUrl(undefined)).toBe('#')
  })
})
