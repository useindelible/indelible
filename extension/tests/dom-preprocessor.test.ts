// @vitest-environment jsdom
import { describe, expect, it } from 'vitest'

import {
  beginCaptureDomCleanup,
  preprocessDocumentForReadableExtraction,
} from '../lib/dom-preprocessor'
import { extractReadableContent } from '../lib/readable-extraction'

describe('DOM preprocessor', () => {
  it('unwraps inline annotation elements without dropping article text', () => {
    const doc = parseHtml(
      `<!doctype html>
      <html>
        <body>
          <main>
            <article>
              <h1>How to Refactor Code with Claude Code</h1>
              <p><mdspan datatext="el1781069058624" class="mdspan-comment">Claude Code and other coding agents</mdspan> are amazing at quickly implementing a lot of code.</p>
            </article>
          </main>
        </body>
      </html>`,
    )

    const result = preprocessDocumentForReadableExtraction(doc)
    const text = doc.body.textContent ?? ''

    expect(result.removedElements).toBe(0)
    expect(doc.querySelector('mdspan')).toBeNull()
    expect(text).toContain(
      'Claude Code and other coding agents are amazing at quickly implementing a lot of code.',
    )
  })

  it('removes the Instructables Autodesk privacy banner before extraction', () => {
    const doc = parseHtml(
      `<!doctype html>
      <html>
        <body>
          <div id="adsk-privacy-preferences" style="display: none">
            <h2>Cookie preferences</h2>
            <p>Your privacy is important to us and so is an optimal experience.</p>
            <p>May we collect and use your data?</p>
            <p>Learn more about the Third Party Services we use and our Privacy Statement.</p>
            <section>
              <h3>THIRD PARTY SERVICES</h3>
              <p>Google Analytics (Strictly Necessary)</p>
              <p>Google Analytics (Web Analytics)</p>
              <p>Google Analytics (Advertising)</p>
            </section>
            <p>Are you sure you want a less customized experience?</p>
            <p>Your experience. Your choice.</p>
          </div>
          <main>
            <article>
              <h1>How to Build a Copper Lamp</h1>
              <p>This project shows how to bend copper pipe, assemble the base, wire the socket, and finish the lamp safely.</p>
              <p>The first step is measuring the pipe and marking each bend so the lamp has a stable shape on the table.</p>
              <p>The final step is testing the lamp with a low wattage bulb before placing it on a desk.</p>
            </article>
          </main>
        </body>
      </html>`,
    )

    const result = preprocessDocumentForReadableExtraction(doc)
    const text = doc.body.textContent ?? ''

    expect(result.removedElements).toBeGreaterThan(0)
    expect(text).not.toContain('Cookie preferences')
    expect(text).not.toContain('Google Analytics (Advertising)')
    expect(text).toContain('How to Build a Copper Lamp')
    expect(text).toContain('bend copper pipe')
  })

  it('does not remove matching text when it is inside article content', () => {
    const doc = parseHtml(
      `<!doctype html>
      <html>
        <body>
          <main>
            <article>
              <h1>Designing Better Cookie Preferences</h1>
              <p>Cookie preferences can be explained clearly without dark patterns.</p>
              <p>Your privacy is important to us is a phrase many teams use, but product teams should make the choice concrete.</p>
              <p>This article compares consent language, privacy statement placement, and third party services disclosures.</p>
            </article>
          </main>
        </body>
      </html>`,
    )

    const result = preprocessDocumentForReadableExtraction(doc)
    const text = doc.body.textContent ?? ''

    expect(result.removedElements).toBe(0)
    expect(text).toContain('Designing Better Cookie Preferences')
    expect(text).toContain('third party services disclosures')
  })

  it('lets Defuddle extract the article instead of the consent block', () => {
    const articleParagraph =
      'This instructable walks through the materials, measurements, assembly process, and finishing steps for a practical workshop project. '
    const doc = parseHtml(
      `<!doctype html>
      <html>
        <body>
          <div class="privacy-modal">
            <h2>Cookie preferences</h2>
            <p>Your privacy is important to us and so is an optimal experience. May we collect and use your data?</p>
            <p>Third Party Services include Google Analytics (Strictly Necessary), Google Analytics (Web Analytics), and Google Analytics (Advertising).</p>
            <p>Your experience. Your choice. Are you sure you want a less customized experience?</p>
          </div>
          <main>
            <article>
              <h1>Workshop Project</h1>
              <p>${articleParagraph.repeat(8)}</p>
              <p>${articleParagraph.repeat(8)}</p>
            </article>
          </main>
        </body>
      </html>`,
    )

    const article = extractReadableContent(doc)

    expect(article.readerHtml).toContain('materials, measurements, assembly process')
    expect(article.readerHtml).not.toContain('Cookie preferences')
    expect(article.readerHtml).not.toContain('Google Analytics')
  })

  it('removes every confirmed named CMP host and underlay', () => {
    const doc = parseHtml(`<!doctype html><html><body>
      <div id="sp_message_container_123"><iframe title="Consent"></iframe></div>
      <div id="cmplz-cookiebanner-container">Complianz</div>
      <div id="BorlabsCookieBox">Cookie preferences <button>Accept</button></div>
      <div id="trustarcNoticeFrame">TrustArc</div>
      <div class="klaro">Klaro</div>
      <div id="transcend-consent-manager">Transcend</div>
      <div id="CybotCookiebotDialogBodyUnderlay">Cookiebot underlay</div>
      <main><article><h1>Article</h1><p>Primary content remains readable.</p></article></main>
    </body></html>`)

    const result = preprocessDocumentForReadableExtraction(doc)

    expect(result.removedElements).toBe(7)
    expect(doc.querySelector('[id^="sp_message_container_"]')).toBeNull()
    expect(doc.querySelector('#cmplz-cookiebanner-container')).toBeNull()
    expect(doc.querySelector('#BorlabsCookieBox')).toBeNull()
    expect(doc.querySelector('#trustarcNoticeFrame')).toBeNull()
    expect(doc.querySelector('.klaro')).toBeNull()
    expect(doc.querySelector('#transcend-consent-manager')).toBeNull()
    expect(doc.querySelector('#CybotCookiebotDialogBodyUnderlay')).toBeNull()
    expect(doc.querySelector('article')?.textContent).toContain('Primary content')
  })

  it('traverses open shadow roots without removing article content', () => {
    const doc = parseHtml(`<!doctype html><html><body>
      <div id="cmp-host"></div>
      <article><p>Cookies are the subject of this article.</p></article>
    </body></html>`)
    const host = doc.querySelector('#cmp-host') as HTMLElement
    const shadow = host.attachShadow({ mode: 'open' })
    shadow.innerHTML = '<div id="cmplz-cookiebanner-container">Cookie preferences</div>'

    preprocessDocumentForReadableExtraction(doc)

    expect(shadow.querySelector('#cmplz-cookiebanner-container')).toBeNull()
    expect(doc.querySelector('article')?.textContent).toContain('Cookies are the subject')
  })

  it('temporarily hides a large overlay, clears scroll locks, then restores the live page', () => {
    const doc =
      parseHtml(`<!doctype html><html style="overflow:hidden"><body style="position:fixed">
      <main><article><p>Primary article content.</p></article></main>
      <div id="marketing-interstitial" style="position:fixed;z-index:2000">Subscribe</div>
    </body></html>`)
    const overlay = doc.querySelector('#marketing-interstitial') as HTMLElement
    Object.defineProperty(doc, 'defaultView', { value: window, configurable: true })
    Object.defineProperty(overlay, 'getBoundingClientRect', {
      value: () => ({ width: 900, height: 700, top: 0, left: 0, right: 900, bottom: 700 }),
    })
    Object.defineProperty(window, 'innerWidth', { value: 1000, configurable: true })
    Object.defineProperty(window, 'innerHeight', { value: 800, configurable: true })

    const cleanup = beginCaptureDomCleanup(doc, 'temporary')

    expect(cleanup.removedElements).toBe(1)
    expect(overlay.style.getPropertyValue('display')).toBe('none')
    expect(doc.documentElement.style.getPropertyValue('overflow')).toBe('auto')
    cleanup.restore()
    expect(overlay.getAttribute('style')).toBe('position:fixed;z-index:2000')
    expect(doc.documentElement.getAttribute('style')).toBe('overflow:hidden')
    expect(doc.body.getAttribute('style')).toBe('position:fixed')
  })

  it('preserves unknown consent-like elements that do not meet every visual overlay threshold', () => {
    const doc = parseHtml(`<!doctype html><html style="overflow:hidden"><body>
      <main><article><p>Primary article content.</p></article></main>
      <aside id="cookie-small" style="position:fixed;z-index:2000">Cookie preferences</aside>
      <aside id="cookie-low" style="position:fixed;z-index:999">Cookie preferences</aside>
      <aside id="cookie-static" style="position:static;z-index:5000">Cookie preferences</aside>
    </body></html>`)
    Object.defineProperty(doc, 'defaultView', { value: window, configurable: true })
    Object.defineProperty(window, 'innerWidth', { value: 1000, configurable: true })
    Object.defineProperty(window, 'innerHeight', { value: 800, configurable: true })
    for (const element of Array.from(doc.querySelectorAll('aside'))) {
      Object.defineProperty(element, 'getBoundingClientRect', {
        value: () => ({ width: 100, height: 100, top: 0, left: 0, right: 100, bottom: 100 }),
      })
    }

    const cleanup = beginCaptureDomCleanup(doc, 'temporary')

    expect(cleanup.removedElements).toBe(0)
    expect(doc.documentElement.getAttribute('style')).toBe('overflow:hidden')
    expect(doc.querySelectorAll('aside')).toHaveLength(3)
  })

  it('never removes a primary article even when its geometry resembles an overlay', () => {
    const doc = parseHtml(`<!doctype html><html><body>
      <main style="position:fixed;z-index:3000"><article><h1>GDPR policy analysis</h1></article></main>
    </body></html>`)
    const main = doc.querySelector('main') as HTMLElement
    Object.defineProperty(doc, 'defaultView', { value: window, configurable: true })
    Object.defineProperty(window, 'innerWidth', { value: 1000, configurable: true })
    Object.defineProperty(window, 'innerHeight', { value: 800, configurable: true })
    Object.defineProperty(main, 'getBoundingClientRect', {
      value: () => ({ width: 1000, height: 800, top: 0, left: 0, right: 1000, bottom: 800 }),
    })

    const cleanup = beginCaptureDomCleanup(doc, 'permanent')

    expect(cleanup.removedElements).toBe(0)
    expect(doc.querySelector('article')?.textContent).toContain('GDPR policy analysis')
  })

  it('uses compound consent UI signals without joining separate class tokens', () => {
    const doc = parseHtml(`<!doctype html><html><body>
      <aside id="cmp-cookie-dialog">Settings</aside>
      <aside class="consentManager">Settings</aside>
      <aside aria-label="Cookie preferences">Settings</aside>
      <aside class="recipe cookie box">Cookie recipe box</aside>
      <aside class="consent-manager-guide">Consent manager guide</aside>
      <aside id="privacy-policy">Privacy policy reference</aside>
      <aside class="consent">We use cookies</aside>
    </body></html>`)

    const result = preprocessDocumentForReadableExtraction(doc)

    expect(result.removedElements).toBe(3)
    expect(doc.querySelector('#cmp-cookie-dialog')).toBeNull()
    expect(doc.querySelector('.consentManager')).toBeNull()
    expect(doc.querySelector('[aria-label="Cookie preferences"]')).toBeNull()
    expect(doc.querySelector('.recipe')).not.toBeNull()
    expect(doc.querySelector('.consent-manager-guide')).not.toBeNull()
    expect(doc.querySelector('#privacy-policy')).not.toBeNull()
    expect(doc.querySelector('.consent')).not.toBeNull()
  })

  it('never hides or removes document roots that resemble consent containers', () => {
    for (const mode of ['temporary', 'permanent'] as const) {
      const doc = parseHtml(`<!doctype html><html class="cookie-banner"><body id="cookie-banner">
        <main><article><p>Primary article content.</p></article></main>
      </body></html>`)
      const originalHtml = doc.documentElement.outerHTML

      const cleanup = beginCaptureDomCleanup(doc, mode)

      expect(cleanup.removedElements).toBe(0)
      expect(doc.documentElement.outerHTML).toBe(originalHtml)
      expect(doc.querySelector('article')?.textContent).toContain('Primary article content')
      cleanup.restore()
      expect(doc.documentElement.outerHTML).toBe(originalHtml)
    }
  })
})

function parseHtml(html: string): Document {
  return new DOMParser().parseFromString(html, 'text/html')
}
