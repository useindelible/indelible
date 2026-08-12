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

  it('rejects application routes on the configured server origin', () => {
    const serverUrl = 'http://localhost:38473'

    expect(canExtensionSaveUrl('http://localhost:38473/', serverUrl)).toBe(false)
    expect(canExtensionSaveUrl('http://localhost:38473/login', serverUrl)).toBe(false)
    expect(canExtensionSaveUrl('http://localhost:38473/reader/doc_123', serverUrl)).toBe(false)
    expect(canExtensionSaveUrl('http://localhost:38473/preferences/account', serverUrl)).toBe(false)
    expect(canExtensionSaveUrl('http://localhost:38473/library', serverUrl)).toBe(false)
  })

  it('allows external web origins and rejects unsupported URLs', () => {
    const serverUrl = 'http://localhost:38473'

    expect(canExtensionSaveUrl('https://example.com', serverUrl)).toBe(true)
    expect(canExtensionSaveUrl('http://localhost:38474/article', serverUrl)).toBe(true)
    expect(canExtensionSaveUrl('chrome://settings', serverUrl)).toBe(false)
    expect(canExtensionSaveUrl('not-a-url', serverUrl)).toBe(false)
  })
})
