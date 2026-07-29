import { describe, it, expect } from 'vitest'
import {
  encodeBase64Utf8,
  stripDataUriPrefix,
  buildFullArchiveBody,
  buildReaderSaveFallbackBody,
} from '../lib/archive'

describe('archive', () => {
  describe('encodeBase64Utf8', () => {
    it('encodes ASCII text to base64', () => {
      const result = encodeBase64Utf8('hello')
      expect(result).toBe(btoa('hello'))
    })

    it('encodes multi-byte UTF-8 characters without throwing', () => {
      const result = encodeBase64Utf8('\u{1F600}')
      expect(typeof result).toBe('string')
      expect(result.length).toBeGreaterThan(0)
    })
  })

  describe('stripDataUriPrefix', () => {
    it('strips data URI prefix', () => {
      expect(stripDataUriPrefix('data:image/png;base64,AAAA')).toBe('AAAA')
    })

    it('returns input unchanged when no comma found', () => {
      expect(stripDataUriPrefix('no-prefix')).toBe('no-prefix')
    })
  })

  describe('buildFullArchiveBody', () => {
    it('builds body with required fields', () => {
      const body = buildFullArchiveBody(
        'https://example.com',
        undefined,
        'Title',
        '<p>content</p>',
        'base64data',
      )
      expect(body.url).toBe('https://example.com')
      expect(body.title).toBe('Title')
      expect(body.reader_html).toBe('<p>content</p>')
      expect(body.html_base64).toBe('base64data')
    })

    it('includes lead_image_url when provided', () => {
      const body = buildFullArchiveBody(
        'https://example.com',
        undefined,
        'Title',
        '<p>content</p>',
        'base64data',
        'https://example.com/image.jpg',
      )
      expect(body.lead_image_url).toBe('https://example.com/image.jpg')
    })

    it('includes optional metadata when provided', () => {
      const body = buildFullArchiveBody(
        'https://example.com',
        undefined,
        'Title',
        '<p>content</p>',
        'base64data',
        undefined,
        'An excerpt',
        'Author',
        'en',
        '2024-03-26',
        'article',
      )
      expect(body.excerpt).toBe('An excerpt')
      expect(body.author).toBe('Author')
      expect(body.language).toBe('en')
      expect(body.published_at).toBe('2024-03-26')
      expect(body.item_type).toBe('article')
    })

    it('includes canonical_url when provided', () => {
      const body = buildFullArchiveBody(
        'https://example.com?session=abc',
        'https://example.com',
        'Title',
        '<p>content</p>',
        'base64data',
      )
      expect(body.canonical_url).toBe('https://example.com')
    })
  })

  describe('buildReaderSaveFallbackBody', () => {
    it('builds body with required fields', () => {
      const body = buildReaderSaveFallbackBody(
        'https://example.com',
        undefined,
        'Title',
        '<p>reader</p>',
      )
      expect(body.url).toBe('https://example.com')
      expect(body.title).toBe('Title')
      expect(body.reader_html).toBe('<p>reader</p>')
    })

    it('includes optional metadata when provided', () => {
      const body = buildReaderSaveFallbackBody(
        'https://example.com',
        undefined,
        'Title',
        '<p>reader</p>',
        'https://example.com/image.jpg',
        'An excerpt',
        'Author',
        'article',
      )
      expect(body.lead_image_url).toBe('https://example.com/image.jpg')
      expect(body.excerpt).toBe('An excerpt')
      expect(body.author).toBe('Author')
      expect(body.item_type).toBe('article')
    })

    it('includes canonical_url when provided', () => {
      const body = buildReaderSaveFallbackBody(
        'https://example.com?session=abc',
        'https://example.com',
        'Title',
        '<p>reader</p>',
      )
      expect(body.canonical_url).toBe('https://example.com')
    })
  })
})
