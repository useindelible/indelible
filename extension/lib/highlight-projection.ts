import {
  boundaryAt,
  buildTextIndex,
  parseDomRangeLocation,
  resolveTextAnchor,
  type AnchorContext,
  type AnchorResolution,
  type SourceLocatorPayload,
  type TextIndex,
} from '../../shared/highlight-source'
import { isPageTextNode, rangeToOffsets } from './page-text'

export type { SourceLocatorPayload } from '../../shared/highlight-source'

export interface ProjectedHighlight {
  id?: string
  color?: string
  text_content: string
  source_locator?: SourceLocatorPayload
  context?: AnchorContext
}

export interface ProjectionResult {
  placed: number
  unplaced: number
}

const MARK_ATTR = 'data-indelible-highlight-id'
const STYLE_ID = 'indelible-highlight-projection-style'

export function projectHighlights(
  highlights: ProjectedHighlight[],
  doc: Document = document,
): ProjectionResult {
  clearProjectedHighlights(doc)

  const projectable = highlights.filter((highlight) => highlight.text_content.trim().length > 0)
  const result: ProjectionResult = { placed: 0, unplaced: 0 }
  if (projectable.length === 0) return result

  ensureProjectionStyle(doc)

  const index = buildTextIndex(doc.body ?? doc.documentElement, isPageTextNode)
  const resolved = projectable.map((highlight, i) => ({
    highlight,
    i,
    span: resolveHighlightSpan(highlight, index, doc),
  }))
  result.unplaced += resolved.filter((entry) => !entry.span).length

  // Wrapping splits text nodes after the split point, so later spans are wrapped first.
  const placeable = resolved
    .filter(
      (entry): entry is typeof entry & { span: { start: number; end: number } } => !!entry.span,
    )
    .sort((a, b) => b.span.start - a.span.start)
  for (const { highlight, i, span } of placeable) {
    const range = spanToRange(index, span, doc)
    if (range && !range.collapsed && wrapTextRange(range, highlight, i, doc) > 0) {
      result.placed += 1
    } else {
      result.unplaced += 1
    }
  }

  return result
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

function resolveHighlightSpan(
  highlight: ProjectedHighlight,
  index: TextIndex,
  doc: Document,
): { start: number; end: number } | undefined {
  const hint = hintFromLocation(highlight.source_locator, index, doc)
  const context = highlight.context ?? highlight.source_locator
  const resolution = resolveTextAnchor(index.text, {
    text: highlight.text_content,
    hint,
    context: context
      ? { offset: context.offset, prefix: context.prefix, suffix: context.suffix }
      : undefined,
  })
  report(highlight.id, resolution)
  return resolution.kind === 'placed' ? { start: resolution.start, end: resolution.end } : undefined
}

function report(id: string | undefined, resolution: AnchorResolution): void {
  if (resolution.kind === 'placed' && resolution.via === 'hint') return
  console.debug('[Indelible] highlight anchor', {
    id,
    stage: resolution.kind === 'placed' ? 'search' : resolution.kind,
  })
}

function hintFromLocation(
  locator: SourceLocatorPayload | undefined,
  index: TextIndex,
  doc: Document,
): { start: number; end: number } | undefined {
  if (!locator?.location) return undefined
  const parsed = parseDomRangeLocation(locator.location)
  if (!parsed) return undefined

  const startNode = resolveNodePath(parsed.startPath, doc)
  const endNode = resolveNodePath(parsed.endPath, doc)
  if (!startNode || !endNode) return undefined

  try {
    const range = doc.createRange()
    range.setStart(startNode, clampOffset(startNode, parsed.startOffset))
    range.setEnd(endNode, clampOffset(endNode, parsed.endOffset))
    return rangeToOffsets(index, range)
  } catch {
    return undefined
  }
}

function clampOffset(node: Node, offset: number): number {
  const max = node.nodeType === Node.TEXT_NODE ? (node as Text).length : node.childNodes.length
  return Math.min(Math.max(0, offset), max)
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

function spanToRange(
  index: TextIndex,
  span: { start: number; end: number },
  doc: Document,
): Range | undefined {
  const start = boundaryAt(index.runs, span.start, false)
  const end = boundaryAt(index.runs, span.end, true)
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

function shouldWrapTextNode(node: Text): boolean {
  return isPageTextNode(node) && node.parentElement?.closest(`[${MARK_ATTR}]`) === null
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
      if (!shouldWrapTextNode(node as Text)) return NodeFilter.FILTER_REJECT
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
