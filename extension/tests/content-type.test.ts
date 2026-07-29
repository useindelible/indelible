import { describe, expect, it } from 'vitest'

import { canExtensionSaveUrl, classifyExtensionUrl } from '../lib/content-type'

describe('content type classifier', () => {
  it('classifies PDFs including arXiv PDFs', () => {
    expect(classifyExtensionUrl('https://example.com/file.pdf').itemType).toBe('pdf')
    expect(classifyExtensionUrl('https://arxiv.org/pdf/2401.12345').itemType).toBe('pdf')
  })

  it('classifies tweets', () => {
    expect(classifyExtensionUrl('https://x.com/sam/status/1234567890')).toEqual({
      itemType: 'tweet',
      platform: 'twitter',
    })
  })

  it('classifies video platforms', () => {
    expect(classifyExtensionUrl('https://www.youtube.com/watch?v=abc').itemType).toBe('video')
    expect(classifyExtensionUrl('https://vimeo.com/12345').itemType).toBe('video')
    expect(classifyExtensionUrl('https://www.twitch.tv/videos/12345').itemType).toBe('video')
  })

  it('only allows web URLs for saving', () => {
    expect(canExtensionSaveUrl('https://example.com')).toBe(true)
    expect(canExtensionSaveUrl('chrome://settings')).toBe(false)
  })
})
