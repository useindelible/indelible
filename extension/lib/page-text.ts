import type { TextIndex } from '../../shared/highlight-source'

const NON_CONTENT_SELECTOR =
  'script, style, noscript, textarea, input, select, option, [contenteditable="true"], #indelible-toolbar-host'

/** Includes text inside projected highlight marks; wrapping never changes the character sequence. */
export function isPageTextNode(node: Text): boolean {
  const parent = node.parentElement
  if (!parent) return false
  return parent.closest(NON_CONTENT_SELECTOR) === null
}

export function rangeToOffsets(
  index: TextIndex,
  range: Range,
): { start: number; end: number } | undefined {
  const start = boundaryOffset(index, range.startContainer, range.startOffset, false)
  const end = boundaryOffset(index, range.endContainer, range.endOffset, true)
  if (start === undefined || end === undefined || end < start) return undefined
  return { start, end }
}

function boundaryOffset(
  index: TextIndex,
  container: Node,
  offset: number,
  preferEnd: boolean,
): number | undefined {
  if (container.nodeType === Node.TEXT_NODE) {
    const run = index.runs.find((candidate) => candidate.node === container)
    return run ? run.start + Math.min(offset, run.node.length) : undefined
  }

  const children = Array.from(container.childNodes)
  const doc = container.ownerDocument
  if (!doc) return undefined
  const probe = doc.createRange()
  if (preferEnd) {
    const before = children[offset - 1]
    if (!before) return firstRunAtOrAfter(index, container)
    probe.selectNodeContents(before)
    return lastRunInside(index, probe)
  }
  const after = children[offset]
  if (!after) return lastRunInsideNode(index, container)
  probe.selectNodeContents(after)
  return firstRunInside(index, probe)
}

function firstRunInside(index: TextIndex, probe: Range): number | undefined {
  const run = index.runs.find((candidate) => probe.intersectsNode(candidate.node))
  return run?.start
}

function lastRunInside(index: TextIndex, probe: Range): number | undefined {
  const run = [...index.runs].reverse().find((candidate) => probe.intersectsNode(candidate.node))
  return run?.end
}

function firstRunAtOrAfter(index: TextIndex, container: Node): number | undefined {
  const run = index.runs.find(
    (candidate) =>
      container.compareDocumentPosition(candidate.node) & Node.DOCUMENT_POSITION_CONTAINED_BY,
  )
  return run?.start
}

function lastRunInsideNode(index: TextIndex, container: Node): number | undefined {
  const run = [...index.runs]
    .reverse()
    .find(
      (candidate) =>
        container.compareDocumentPosition(candidate.node) & Node.DOCUMENT_POSITION_CONTAINED_BY,
    )
  return run?.end
}
