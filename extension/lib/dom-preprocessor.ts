export interface DomPreprocessResult {
  removedElements: number
}

export interface CaptureDomCleanup {
  removedElements: number
  restore(): void
}

const STRONG_CONSENT_SELECTORS = [
  '#onetrust-consent-sdk',
  '#onetrust-banner-sdk',
  '#onetrust-pc-sdk',
  '#CybotCookiebotDialog',
  '#cookiebanner',
  '#cookie-banner',
  '#cookie-consent',
  '#cookie-notice',
  '#gdpr-consent',
  '#privacy-banner',
  '#consent-banner',
  '#cc-main',
  '#qc-cmp2-container',
  '#usercentrics-root',
  '#didomi-host',
  '#sp_message_container',
  '[id^="sp_message_container_"]',
  '#cmplz-cookiebanner-container',
  '#BorlabsCookieBox',
  '#trustarcNoticeFrame',
  '#transcend-consent-manager',
  '#CybotCookiebotDialogBodyUnderlay',
  '#iubenda-cs-banner',
  '#iubenda-cs-container',
  '.cookie-banner',
  '.cookie-consent',
  '.cookie-notice',
  '.consent-banner',
  '.gdpr-banner',
  '.cc-banner',
  '.cc-window',
  '.fc-consent-root',
  '.cky-consent-container',
  '.osano-cm-window',
  '.sp_message_container',
  '.klaro',
  '.truste_box_overlay',
  '.qc-cmp2-container',
]

const GENERIC_CONSENT_SELECTORS = [
  '[id*="cookie" i]',
  '[id*="consent" i]',
  '[id*="gdpr" i]',
  '[id*="privacy" i]',
  '[class*="cookie" i]',
  '[class*="consent" i]',
  '[class*="gdpr" i]',
  '[class*="privacy" i]',
  '[aria-label*="cookie" i]',
  '[aria-label*="consent" i]',
  '[aria-label*="privacy" i]',
  '[role="dialog"]',
  '[role="alertdialog"]',
  '[aria-modal="true"]',
  '[style*="position: fixed" i]',
  '[style*="position:fixed" i]',
  '[style*="position: sticky" i]',
  '[style*="position:sticky" i]',
]

const TEXT_MATCH_CANDIDATE_SELECTOR = [
  'aside',
  'dialog',
  'section',
  'div',
  'form',
  '[role="dialog"]',
  '[role="alertdialog"]',
  '[aria-modal="true"]',
].join(',')

const ARTICLE_CONTAINER_SELECTOR = [
  'article',
  'main',
  '[role="main"]',
  '.post-content',
  '.entry-content',
  '.article-body',
  '.article-content',
  '.step',
  '.step-body',
  '.instructable',
].join(',')

const INLINE_TEXT_WRAPPER_SELECTOR = [
  'mdspan',
  'span[class*="mdspan" i]',
  'span[class*="annotation" i][datatext]',
  'span[class*="annotation" i][data-text]',
  'span[class*="comment" i][datatext]',
  'span[class*="comment" i][data-text]',
  'span[class*="highlight" i][datatext]',
  'span[class*="highlight" i][data-text]',
].join(',')

const BLOCK_CHILD_SELECTOR = [
  'address',
  'article',
  'aside',
  'blockquote',
  'details',
  'dialog',
  'div',
  'dl',
  'fieldset',
  'figcaption',
  'figure',
  'footer',
  'form',
  'h1',
  'h2',
  'h3',
  'h4',
  'h5',
  'h6',
  'header',
  'hr',
  'li',
  'main',
  'nav',
  'ol',
  'p',
  'pre',
  'section',
  'table',
  'ul',
].join(',')

const CONSENT_UI_ATTRIBUTE_PATTERN =
  /(?:^|[^a-z0-9])(?:cookie|consent|gdpr)[^a-z0-9]*(?:banner|modal|dialog|notice|manager|overlay|popup|box|preferences)$/i

export function preprocessDocumentForReadableExtraction(doc: Document): DomPreprocessResult {
  const result: DomPreprocessResult = { removedElements: 0 }

  unwrapInlineTextWrappers(doc)
  removeStrongConsentElements(doc, result)
  removeGenericConsentElements(doc, result)
  removeConsentTextBlocks(doc, result)

  return result
}

export function beginCaptureDomCleanup(
  doc: Document,
  mode: 'temporary' | 'permanent',
): CaptureDomCleanup {
  const candidates = collectStrongConsentCandidates(doc)
  const win = doc.defaultView
  const primaryContent = doc.querySelector(ARTICLE_CONTAINER_SELECTOR)

  if (win) {
    for (const element of queryAllRoots(doc, 'body *')) {
      if (isInsideArticleContainer(element)) continue
      if (primaryContent && element.contains(primaryContent)) continue
      if (isLargeVisualOverlay(element, win)) candidates.add(element)
    }
  }

  const restorers: Array<() => void> = []
  let removedElements = 0
  for (const element of candidates) {
    if (
      !element.isConnected ||
      element === doc.documentElement ||
      element === doc.body ||
      isInsideArticleContainer(element)
    ) {
      continue
    }
    if (mode === 'temporary') {
      const originalStyle = element.getAttribute('style')
      restorers.push(() => restoreStyleAttribute(element, originalStyle))
      if (element instanceof HTMLElement || element instanceof SVGElement) {
        element.style.setProperty('display', 'none', 'important')
      }
    } else {
      element.remove()
    }
    removedElements += 1
  }

  if (removedElements > 0) {
    for (const root of [doc.documentElement, doc.body]) {
      if (!root) continue
      const originalStyle = root.getAttribute('style')
      if (mode === 'temporary') {
        restorers.push(() => restoreStyleAttribute(root, originalStyle))
      }
      root.style.setProperty('overflow', 'auto', 'important')
      root.style.setProperty('position', 'static', 'important')
      root.style.setProperty('top', 'auto', 'important')
      root.style.setProperty('width', 'auto', 'important')
      root.style.setProperty('height', 'auto', 'important')
    }
  }

  return {
    removedElements,
    restore() {
      if (mode !== 'temporary') return
      for (const restore of restorers.reverse()) restore()
    },
  }
}

function unwrapInlineTextWrappers(doc: Document): void {
  for (const element of queryAllRoots(doc, INLINE_TEXT_WRAPPER_SELECTOR)) {
    if (!shouldUnwrapInlineTextWrapper(element)) continue
    unwrapElement(element)
  }
}

function removeStrongConsentElements(doc: Document, result: DomPreprocessResult): void {
  for (const selector of STRONG_CONSENT_SELECTORS) {
    for (const element of queryAllRoots(doc, selector)) {
      if (isInsideArticleContainer(element)) continue
      removeElement(element, result)
    }
  }
}

function removeGenericConsentElements(doc: Document, result: DomPreprocessResult): void {
  for (const selector of GENERIC_CONSENT_SELECTORS) {
    for (const element of queryAllRoots(doc, selector)) {
      if (isInsideArticleContainer(element)) continue
      if (consentTextScore(element.textContent ?? '') < 3 && !hasConsentAttributeSignal(element)) {
        continue
      }
      removeElement(element, result)
    }
  }
}

function removeConsentTextBlocks(doc: Document, result: DomPreprocessResult): void {
  const candidates = queryAllRoots(doc, TEXT_MATCH_CANDIDATE_SELECTOR)
    .filter((element) => {
      if (isInsideArticleContainer(element)) return false
      const textLength = normalizedText(element.textContent ?? '').length
      return textLength >= 80 && textLength <= 6000
    })
    .sort(
      (a, b) =>
        normalizedText(a.textContent ?? '').length - normalizedText(b.textContent ?? '').length,
    )

  for (const element of candidates) {
    if (!element.isConnected) continue
    if (consentTextScore(element.textContent ?? '') >= 4) {
      removeElement(element, result)
    }
  }
}

function safeQuerySelectorAll(root: ParentNode, selector: string): Element[] {
  try {
    return Array.from(root.querySelectorAll(selector))
  } catch {
    return []
  }
}

function queryAllRoots(doc: Document, selector: string): Element[] {
  const matches: Element[] = []
  const roots: ParentNode[] = [doc]
  for (let index = 0; index < roots.length; index += 1) {
    const root = roots[index]
    if (!root) continue
    matches.push(...safeQuerySelectorAll(root, selector))
    for (const element of safeQuerySelectorAll(root, '*')) {
      if (element.shadowRoot) roots.push(element.shadowRoot)
    }
  }
  return Array.from(new Set(matches))
}

function collectStrongConsentCandidates(doc: Document): Set<Element> {
  const candidates = new Set<Element>()
  for (const selector of STRONG_CONSENT_SELECTORS) {
    for (const element of queryAllRoots(doc, selector)) candidates.add(element)
  }
  return candidates
}

function isLargeVisualOverlay(element: Element, win: Window): boolean {
  if (!(element instanceof HTMLElement || element instanceof SVGElement)) return false
  const style = win.getComputedStyle(element)
  if (
    style.display === 'none' ||
    style.visibility === 'hidden' ||
    (style.opacity !== '' && Number(style.opacity) === 0)
  ) {
    return false
  }
  if (style.position !== 'fixed' && style.position !== 'sticky') return false
  const zIndex = Number.parseInt(style.zIndex, 10)
  if (!Number.isFinite(zIndex) || zIndex < 1000) return false
  const rect = element.getBoundingClientRect()
  const viewportArea = Math.max(1, win.innerWidth * win.innerHeight)
  const coveredArea = Math.max(0, rect.width) * Math.max(0, rect.height)
  return coveredArea / viewportArea >= 0.35
}

function restoreStyleAttribute(element: Element, value: string | null): void {
  if (value === null) element.removeAttribute('style')
  else element.setAttribute('style', value)
}

function shouldUnwrapInlineTextWrapper(element: Element): boolean {
  if (
    !element.isConnected ||
    element === element.ownerDocument.documentElement ||
    element === element.ownerDocument.body
  ) {
    return false
  }

  const tagName = element.tagName.toLowerCase()
  if (tagName !== 'mdspan' && tagName !== 'span') return false
  if (element.matches('button, canvas, iframe, input, math, option, script, select, style, svg')) {
    return false
  }
  if (element.querySelector(BLOCK_CHILD_SELECTOR) !== null) return false
  if (!normalizedText(element.textContent ?? '')) return false

  if (tagName === 'mdspan') return true

  const signal = [
    element.getAttribute('class'),
    element.getAttribute('datatext'),
    element.getAttribute('data-text'),
  ]
    .filter(Boolean)
    .join(' ')

  return /\b(mdspan|annotation|comment|highlight)\b/i.test(signal)
}

function unwrapElement(element: Element): void {
  const parent = element.parentNode
  if (!parent) return

  while (element.firstChild) {
    parent.insertBefore(element.firstChild, element)
  }
  parent.removeChild(element)
}

function removeElement(element: Element, result: DomPreprocessResult): void {
  if (
    !element.isConnected ||
    element === element.ownerDocument.documentElement ||
    element === element.ownerDocument.body
  ) {
    return
  }
  element.remove()
  result.removedElements += 1
}

function isInsideArticleContainer(element: Element): boolean {
  let current: Element | null = element
  while (current) {
    if (current.closest(ARTICLE_CONTAINER_SELECTOR)) return true
    const root = current.getRootNode()
    current = root instanceof ShadowRoot ? root.host : null
  }
  return false
}

function hasConsentAttributeSignal(element: Element): boolean {
  const classTokens = (element.getAttribute('class') ?? '').split(/\s+/).filter(Boolean)
  const signals = [element.id, element.getAttribute('aria-label') ?? '', ...classTokens]
  return signals.some((signal) => CONSENT_UI_ATTRIBUTE_PATTERN.test(signal))
}

function consentTextScore(value: string): number {
  const text = normalizedText(value)
  if (!text) return 0

  let score = 0
  if (/cookie preferences/i.test(text)) score += 3
  if (/your privacy is important to us/i.test(text)) score += 3
  if (/may we collect and use your data/i.test(text)) score += 3
  if (/third party services/i.test(text)) score += 2
  if (/your experience\.?\s+your choice/i.test(text)) score += 2
  if (/less customized experience/i.test(text)) score += 2
  if (/google analytics\s*\(/i.test(text)) score += 2
  if (/privacy statement/i.test(text)) score += 1
  if (/\b(cookie|cookies|consent|privacy|gdpr)\b/i.test(text)) score += 1
  if (/\b(accept|reject|manage|preferences|customi[sz]e|third[- ]party)\b/i.test(text)) score += 1

  return score
}

function normalizedText(value: string): string {
  return value.replace(/\s+/g, ' ').trim()
}
