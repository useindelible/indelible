import { boundaryAt, buildTextIndex } from '../../shared/highlight-source'
import type { CaptureMessage } from './capture'
import { nodePath } from './dom-range'
import { clearProjectedHighlights } from './highlight-projection'
import { isPageTextNode, rangeToOffsets } from './page-text'

const CONTEXT_CHARS = 80

/** Clears projected marks so the location describes the page as first loaded. */
export function captureSelection(doc: Document): CaptureMessage {
  const selection = doc.getSelection()
  if (!selection || selection.rangeCount === 0 || selection.toString().trim().length === 0) {
    return { action: 'capture:error', message: 'No selected text found' }
  }

  const range = selection.getRangeAt(0)
  const raw = selection.toString()
  const text = raw.trim()
  const live = buildTextIndex(doc.body, isPageTextNode)
  const rawOffsets = rangeToOffsets(live, range)
  if (!rawOffsets) {
    return { action: 'capture:error', message: 'No selected text found' }
  }
  const start = rawOffsets.start + (raw.length - raw.trimStart().length)
  const end = rawOffsets.end - (raw.length - raw.trimEnd().length)

  clearProjectedHighlights(doc)
  const pristine = buildTextIndex(doc.body, isPageTextNode)
  const startBoundary = boundaryAt(pristine.runs, start, false)
  const endBoundary = boundaryAt(pristine.runs, end, true)
  if (!startBoundary || !endBoundary) {
    return { action: 'capture:error', message: 'No selected text found' }
  }

  const location = `${nodePath(startBoundary.node)}:${startBoundary.offset},${nodePath(
    endBoundary.node,
  )}:${endBoundary.offset}`
  const prefix = pristine.text.slice(Math.max(0, start - CONTEXT_CHARS), start).trim()
  const suffix = pristine.text.slice(end, end + CONTEXT_CHARS).trim()

  return {
    action: 'selection:result',
    payload: {
      text,
      sourceLocator: {
        type: 'web_page_dom_range',
        url: doc.location.href,
        location,
        offset: start,
        text_content: text,
        prefix: prefix || undefined,
        suffix: suffix || undefined,
      },
    },
  }
}
