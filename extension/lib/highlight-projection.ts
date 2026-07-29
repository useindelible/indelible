import {
  boundaryAt,
  buildTextIndex,
  findBestTextMatch,
  normalizeWhitespace,
  parseDomRangeLocation,
  type SourceLocatorPayload,
} from '../../shared/highlight-source'

export type { SourceLocatorPayload } from '../../shared/highlight-source'

export interface ProjectedHighlight {
  id?: string
  color?: string
  text_content: string
  source_locator?: SourceLocatorPayload
}

const MARK_ATTR = 'data-indelible-highlight-id'
const STYLE_ID = 'indelible-highlight-projection-style'

export function projectHighlights(
  highlights: ProjectedHighlight[],
  doc: Document = document,
): number {
  clearProjectedHighlights(doc)

  const projectable = highlights.filter((highlight) => highlight.text_content.trim().length > 0)
  if (projectable.length === 0) return 0

  ensureProjectionStyle(doc)

  let projectedCount = 0
  for (const [index, highlight] of projectable.entries()) {
    const range = resolveHighlightRange(highlight, doc)
    if (!range || range.collapsed) continue

    const wrapped = wrapTextRange(range, highlight, index, doc)
    if (wrapped > 0) projectedCount += 1
  }

  return projectedCount
}

export function clearProjectedHighlights(doc: Document = document): void {
  const marks = Array.from(doc.querySelectorAll<HTMLElement>(`[${MARK_ATTR}]`))
  for (const mark of marks) {
    const parent = mark.parentNode
    if (!parent) continue
    while (mark.firstChild) {
      parent.insertBefore(mark.firstChild, mark)
    }
    parent.removeChild(mark)
    parent.normalize()
  }
}

function resolveHighlightRange(highlight: ProjectedHighlight, doc: Document): Range | undefined {
  const locator = highlight.source_locator
  if (locator?.location) {
    const located = resolveRangeFromLocation(locator, doc)
    if (located && rangeMatchesHighlight(located, highlight.text_content)) {
      return located
    }
  }

  return resolveRangeFromText(highlight, doc)
}

function resolveRangeFromLocation(locator: SourceLocatorPayload, doc: Document): Range | undefined {
  const parsed = parseDomRangeLocation(locator.location)
  if (!parsed) return undefined

  const startNode = resolveNodePath(parsed.startPath, doc)
  const endNode = resolveNodePath(parsed.endPath, doc)
  if (!startNode || !endNode) return undefined

  const start = resolveBoundary(startNode, parsed.startOffset, false)
  const end = resolveBoundary(endNode, parsed.endOffset, true)
  if (!start || !end) return undefined

  try {
    const range = doc.createRange()
    range.setStart(start.node, start.offset)
    range.setEnd(end.node, end.offset)
    return range
  } catch {
    return undefined
  }
}

function resolveNodePath(path: string, doc: Document): Node | undefined {
  let current: Node | undefined = doc.documentElement
  if (!path) return current

  for (const rawPart of path.split('/')) {
    if (!current) return undefined
    const index = Number(rawPart)
    if (!Number.isInteger(index) || index < 0) return undefined
    current = current.childNodes[index]
  }

  return current
}

function resolveBoundary(
  node: Node,
  offset: number,
  preferEnd: boolean,
): { node: Text; offset: number } | undefined {
  if (node.nodeType === Node.TEXT_NODE) {
    const text = node as Text
    return { node: text, offset: clamp(offset, 0, text.length) }
  }

  const element = node.nodeType === Node.ELEMENT_NODE ? (node as Element) : node.parentElement
  if (!element) return undefined

  const index = buildTextIndex(element, shouldIndexTextNode)
  return boundaryAt(index.runs, clamp(offset, 0, index.text.length), preferEnd)
}

function resolveRangeFromText(highlight: ProjectedHighlight, doc: Document): Range | undefined {
  const needle = highlight.text_content.trim()
  if (!needle) return undefined

  const root = doc.body ?? doc.documentElement
  const index = buildTextIndex(root, shouldIndexTextNode)
  const match = findBestTextMatch(index.text, needle, highlight.source_locator)
  if (!match) return undefined

  const start = boundaryAt(index.runs, match.start, false)
  const end = boundaryAt(index.runs, match.end, true)
  if (!start || !end) return undefined

  try {
    const range = doc.createRange()
    range.setStart(start.node, start.offset)
    range.setEnd(end.node, end.offset)
    return range
  } catch {
    return undefined
  }
}

function rangeMatchesHighlight(range: Range, textContent: string): boolean {
  const rangeText = normalizeWhitespace(range.toString())
  const highlightText = normalizeWhitespace(textContent)
  return rangeText === highlightText || rangeText.includes(highlightText)
}

function shouldIndexTextNode(node: Text): boolean {
  const parent = node.parentElement
  if (!parent) return false
  return (
    parent.closest(
      `script, style, noscript, textarea, input, select, option, [contenteditable="true"], [${MARK_ATTR}]`,
    ) === null
  )
}

function wrapTextRange(
  range: Range,
  highlight: ProjectedHighlight,
  index: number,
  doc: Document,
): number {
  const root =
    range.commonAncestorContainer.nodeType === Node.TEXT_NODE
      ? range.commonAncestorContainer.parentNode
      : range.commonAncestorContainer
  if (!root) return 0

  const textNodes = collectTextNodesInRange(root, range, doc)
  let wrapped = 0
  for (const textNode of textNodes.reverse()) {
    const start = textNode === range.startContainer ? range.startOffset : 0
    const end = textNode === range.endContainer ? range.endOffset : textNode.length
    if (start >= end) continue

    wrapTextNodeSlice(textNode, start, end, highlight, index, doc)
    wrapped += 1
  }

  return wrapped
}

function collectTextNodesInRange(root: Node, range: Range, doc: Document): Text[] {
  const nodes: Text[] = []
  const walker = doc.createTreeWalker(root, NodeFilter.SHOW_TEXT, {
    acceptNode(node) {
      if (!shouldIndexTextNode(node as Text)) return NodeFilter.FILTER_REJECT
      return range.intersectsNode(node) ? NodeFilter.FILTER_ACCEPT : NodeFilter.FILTER_REJECT
    },
  })

  let node = walker.nextNode()
  while (node) {
    nodes.push(node as Text)
    node = walker.nextNode()
  }

  return nodes
}

function wrapTextNodeSlice(
  textNode: Text,
  start: number,
  end: number,
  highlight: ProjectedHighlight,
  index: number,
  doc: Document,
): void {
  const parent = textNode.parentNode
  if (!parent) return

  const after = textNode.splitText(end)
  const selected = textNode.splitText(start)
  const mark = doc.createElement('mark')
  mark.setAttribute(MARK_ATTR, highlight.id ?? String(index))
  mark.className = 'indelible-projected-highlight'
  mark.dataset.indelibleHighlightColor = highlight.color ?? 'yellow'
  mark.appendChild(selected)
  parent.insertBefore(mark, after)
}

function ensureProjectionStyle(doc: Document): void {
  if (doc.getElementById(STYLE_ID)) return

  const style = doc.createElement('style')
  style.id = STYLE_ID
  style.textContent = `
    mark.indelible-projected-highlight {
      background: rgba(255, 214, 10, 0.42) !important;
      color: inherit !important;
      border-radius: 2px !important;
      box-shadow: 0 0 0 1px rgba(255, 190, 0, 0.18) !important;
      padding: 0.02em 0 !important;
    }
  `
  ;(doc.head ?? doc.documentElement).appendChild(style)
}

function clamp(value: number, min: number, max: number): number {
  return Math.min(max, Math.max(min, value))
}
