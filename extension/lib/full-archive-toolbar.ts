import { escapeHtml } from '@/lib/html'
import { nodePath } from '@/lib/dom-range'
import {
  clearProjectedHighlights,
  projectHighlights,
  type ProjectedHighlight,
} from '@/lib/highlight-projection'

import { toolbarMarkup, triageIcon, triageLabel } from './full-archive-toolbar-markup'
import { toolbarStyles } from './full-archive-toolbar-styles'

interface ToolbarEntry {
  library_entry_id: string
  document_id?: string
  title?: string
  triage_state?: string
  is_favorite?: boolean
  saved_at?: string
  url?: string
}

export interface ToolbarState {
  view:
    | 'checking'
    | 'connecting'
    | 'saving'
    | 'saved'
    | 'disconnected'
    | 'auth-error'
    | 'unsupported'
    | 'error'
    | 'already-saved'
  serverUrl?: string
  readerUrl?: string
  url?: string
  step?: string
  message?: string
  entry?: ToolbarEntry
  tags?: string[]
  note?: string
  highlights?: ProjectedHighlight[]
}

const TOOLBAR_HOST_ID = 'indelible-toolbar-host'
let docListenersInstalled = false
let autoHighlightEnabled = false
let autoHighlightListenersInstalled = false
let autoHighlightTimer: ReturnType<typeof setTimeout> | undefined
let pendingAutoHighlightKey: string | undefined
let lastAutoHighlightKey: string | undefined

export function renderToolbar(rawState: unknown): void {
  const state = parseToolbarState(rawState)
  const root = ensureToolbarRoot()
  const wasOpen = root.querySelector('.bar.is-open') !== null
  const noteDraft = state.view === 'saved' ? readOpenNoteDraft(root) : undefined
  root.innerHTML = toolbarStyles() + toolbarMarkup(state)
  restoreOpenNoteDraft(root, noteDraft)
  const bar = root.querySelector('.bar')
  if (wasOpen) {
    bar?.classList.add('is-open')
  } else {
    requestAnimationFrame(() => {
      bar?.classList.add('is-open')
    })
  }
  bindToolbarEvents(root, state)
  syncHighlightProjection(root, state)
}

function readOpenNoteDraft(root: ShadowRoot): string | undefined {
  const panel = root.querySelector<HTMLElement>('.note-panel')
  if (!panel || panel.style.display === 'none') return undefined
  return root.querySelector<HTMLTextAreaElement>('.note-textarea')?.value
}

function restoreOpenNoteDraft(root: ShadowRoot, draft: string | undefined): void {
  if (draft === undefined) return
  const textarea = root.querySelector<HTMLTextAreaElement>('.note-textarea')
  if (!textarea) return
  textarea.value = draft
  root.querySelector<HTMLElement>('.note-panel')?.style.setProperty('display', 'block')
  root.querySelector<HTMLElement>('.js-note-btn')?.classList.add('panel-open')
}

function parseToolbarState(rawState: unknown): ToolbarState {
  if (typeof rawState !== 'object' || rawState === null) return { view: 'checking' }
  const record = rawState as Record<string, unknown>
  const view = typeof record.view === 'string' ? record.view : 'checking'
  const entry = parseToolbarEntry(record.entry)
  return {
    view: isToolbarView(view) ? view : 'checking',
    serverUrl: typeof record.serverUrl === 'string' ? record.serverUrl : undefined,
    readerUrl: typeof record.readerUrl === 'string' ? record.readerUrl : undefined,
    url: typeof record.url === 'string' ? record.url : undefined,
    step: typeof record.step === 'string' ? record.step : undefined,
    message: typeof record.message === 'string' ? record.message : undefined,
    entry,
    tags: Array.isArray(record.tags)
      ? record.tags.filter((tag): tag is string => typeof tag === 'string')
      : undefined,
    note: typeof record.note === 'string' ? record.note : undefined,
    highlights: Array.isArray(record.highlights)
      ? record.highlights.map(parseProjectedHighlight).filter((h): h is ProjectedHighlight => !!h)
      : undefined,
  }
}

function parseProjectedHighlight(value: unknown): ProjectedHighlight | undefined {
  if (typeof value === 'string') return { text_content: value }
  if (typeof value !== 'object' || value === null) return undefined

  const record = value as Record<string, unknown>
  const textContent = record.text_content
  if (typeof textContent !== 'string') return undefined

  const sourceLocator = parseSourceLocator(record.source_locator, textContent)
  return {
    id: typeof record.id === 'string' ? record.id : undefined,
    color: typeof record.color === 'string' ? record.color : undefined,
    text_content: textContent,
    source_locator: sourceLocator,
  }
}

function parseSourceLocator(
  value: unknown,
  textContent: string,
): ProjectedHighlight['source_locator'] {
  if (typeof value !== 'object' || value === null) return undefined
  const record = value as Record<string, unknown>
  return {
    type: 'web_page_dom_range',
    url: stringField(record, 'url') ?? '',
    location: stringField(record, 'location') ?? '',
    offset: numberField(record, 'offset'),
    text_content: stringField(record, 'text_content') ?? textContent,
    prefix: stringField(record, 'prefix'),
    suffix: stringField(record, 'suffix'),
  }
}

function stringField(record: Record<string, unknown>, key: string): string | undefined {
  const value = record[key]
  return typeof value === 'string' ? value : undefined
}

function numberField(record: Record<string, unknown>, key: string): number | undefined {
  const value = record[key]
  return typeof value === 'number' ? value : undefined
}

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === 'object' && value !== null
}

function parseToolbarEntry(value: unknown): ToolbarEntry | undefined {
  if (typeof value !== 'object' || value === null) return undefined
  const record = value as Record<string, unknown>
  if (typeof record.library_entry_id !== 'string') return undefined
  return {
    library_entry_id: record.library_entry_id,
    document_id: typeof record.document_id === 'string' ? record.document_id : undefined,
    title: typeof record.title === 'string' ? record.title : undefined,
    triage_state: typeof record.triage_state === 'string' ? record.triage_state : undefined,
    is_favorite: typeof record.is_favorite === 'boolean' ? record.is_favorite : undefined,
    saved_at: typeof record.saved_at === 'string' ? record.saved_at : undefined,
    url: typeof record.url === 'string' ? record.url : undefined,
  }
}

function isToolbarView(value: string): value is ToolbarState['view'] {
  return [
    'checking',
    'connecting',
    'saving',
    'saved',
    'disconnected',
    'auth-error',
    'unsupported',
    'error',
    'already-saved',
  ].includes(value)
}

function ensureToolbarRoot(): ShadowRoot {
  let host = document.getElementById(TOOLBAR_HOST_ID)
  if (!host) {
    host = document.createElement('div')
    host.id = TOOLBAR_HOST_ID
    document.documentElement.append(host)
  }
  return host.shadowRoot ?? host.attachShadow({ mode: 'open' })
}

// ── SVG icons ────────────────────────────────────────────────────────────────
function installDocumentListeners(): void {
  if (docListenersInstalled) return
  docListenersInstalled = true

  document.addEventListener('keydown', (e) => {
    if (e.key !== 'Escape') return
    const hostRoot = document.getElementById(TOOLBAR_HOST_ID)?.shadowRoot
    if (!hostRoot) return
    const menu = hostRoot.querySelector<HTMLElement>('.triage-menu')
    if (menu && menu.style.display !== 'none') {
      menu.style.display = 'none'
      hostRoot.querySelector<HTMLElement>('.js-triage')?.classList.remove('triage-open')
      return
    }
    hostRoot.querySelector<HTMLElement>('.tag-panel')?.style.setProperty('display', 'none')
    hostRoot.querySelector<HTMLElement>('.note-panel')?.style.setProperty('display', 'none')
    hostRoot.querySelector<HTMLElement>('.js-tag-btn')?.classList.remove('panel-open')
    hostRoot.querySelector<HTMLElement>('.js-note-btn')?.classList.remove('panel-open')
  })

  document.addEventListener('click', (e) => {
    const hostRoot = document.getElementById(TOOLBAR_HOST_ID)?.shadowRoot
    if (!hostRoot) return
    const menu = hostRoot.querySelector<HTMLElement>('.triage-menu')
    if (!menu || menu.style.display === 'none') return
    const triageBtn = hostRoot.querySelector<HTMLElement>('.js-triage')
    const path = e.composedPath()
    if (!path.includes(menu) && !(triageBtn && path.includes(triageBtn))) {
      menu.style.display = 'none'
      triageBtn?.classList.remove('triage-open')
    }
  })
}

function bindToolbarEvents(root: ShadowRoot, state: ToolbarState): void {
  installDocumentListeners()

  const entry = state.entry

  function dismiss(): void {
    const bar = root.querySelector<HTMLElement>('.bar')
    if (bar) {
      bar.style.transform = 'translateY(-110%)'
      bar.style.transition = 'transform 200ms ease'
    }
    setTimeout(() => document.getElementById(TOOLBAR_HOST_ID)?.remove(), 220)
  }

  function closeTriage(): void {
    root.querySelector<HTMLElement>('.triage-menu')?.style.setProperty('display', 'none')
    root.querySelector<HTMLElement>('.js-triage')?.classList.remove('triage-open')
  }

  function closeAllPanels(): void {
    root.querySelector<HTMLElement>('.tag-panel')?.style.setProperty('display', 'none')
    root.querySelector<HTMLElement>('.note-panel')?.style.setProperty('display', 'none')
    root.querySelector<HTMLElement>('.js-tag-btn')?.classList.remove('panel-open')
    root.querySelector<HTMLElement>('.js-note-btn')?.classList.remove('panel-open')
  }

  // Dismiss / minimize
  root.querySelector('.js-dismiss')?.addEventListener('click', dismiss)
  root.querySelector('.js-minimize')?.addEventListener('click', dismiss)

  // Connect (disconnected state)
  const doConnect = async (): Promise<void> => {
    const input = root.querySelector<HTMLInputElement>('[data-role="server-url"]')
    const serverUrl = input?.value.trim() || state.serverUrl || 'https://useindelible.com'
    renderToolbar({ view: 'connecting', serverUrl })

    try {
      const response = (await browser.runtime.sendMessage({
        action: 'toolbar:connect',
        serverUrl,
      })) as { success?: boolean; error?: string } | undefined
      if (response?.success) return

      renderToolbar({
        view: 'auth-error',
        serverUrl,
        message:
          response?.error?.replace(/^Error:\s*/, '') || 'Authorization could not be started.',
      })
    } catch (error) {
      renderToolbar({
        view: 'auth-error',
        serverUrl,
        message:
          error instanceof Error
            ? error.message
            : 'Authorization could not be started. Please try again.',
      })
    }
  }
  root.querySelector('[data-action="connect"]')?.addEventListener('click', () => {
    void doConnect()
  })
  root
    .querySelector<HTMLInputElement>('[data-role="server-url"]')
    ?.addEventListener('keydown', (e) => {
      if (e.key === 'Enter') void doConnect()
    })

  root.querySelector('[data-action="refresh"]')?.addEventListener('click', () => {
    void browser.runtime.sendMessage({ action: 'toolbar:save' })
  })

  if (!entry) return
  const libraryEntryId = entry.library_entry_id

  root
    .querySelector<HTMLButtonElement>('.js-reprocess-btn')
    ?.addEventListener('click', async () => {
      const documentId = entry.document_id
      const button = root.querySelector<HTMLButtonElement>('.js-reprocess-btn')
      const status = root.querySelector<HTMLElement>('.js-reprocess-status')
      if (!documentId || !button || !status || button.disabled) return
      button.disabled = true
      status.textContent = 'Queuing'
      try {
        const response = (await browser.runtime.sendMessage({
          action: 'toolbar:reprocess-document',
          documentId,
        })) as unknown
        const record = isRecord(response) ? response : undefined
        const data = record && isRecord(record.data) ? record.data : undefined
        if (!record || record.success !== true || !data) {
          throw new Error(typeof record?.error === 'string' ? record.error : 'Retry failed')
        }
        const queued = data.queued === true
        const retryAfter =
          typeof data.retry_after_seconds === 'number' ? data.retry_after_seconds : 0
        status.textContent = queued
          ? 'Queued'
          : retryAfter > 0
            ? `Retry in ${retryAfter}s`
            : 'Running'
        window.setTimeout(
          () => {
            button.disabled = false
            status.textContent = ''
          },
          (queued ? 300 : retryAfter > 0 ? retryAfter : 30) * 1000,
        )
      } catch (error) {
        button.disabled = false
        status.textContent = error instanceof Error ? error.message : 'Retry failed'
      }
    })

  syncAutoHighlightToggle(root)

  // Auto-highlight toggle
  root.querySelector('.js-toggle')?.addEventListener('click', () => {
    const toggle = root.querySelector<HTMLElement>('.js-toggle')
    if (!toggle) return
    setAutoHighlightEnabled(root, toggle.classList.contains('off'))
  })

  // Tag panel toggle
  root.querySelector('.js-tag-btn')?.addEventListener('click', () => {
    const tagPanel = root.querySelector<HTMLElement>('.tag-panel')
    const tagBtn = root.querySelector<HTMLElement>('.js-tag-btn')
    if (!tagPanel) return
    const isOpen = tagPanel.style.display !== 'none'
    root.querySelector<HTMLElement>('.note-panel')?.style.setProperty('display', 'none')
    root.querySelector<HTMLElement>('.js-note-btn')?.classList.remove('panel-open')
    tagPanel.style.display = isOpen ? 'none' : 'block'
    tagBtn?.classList.toggle('panel-open', !isOpen)
    if (!isOpen)
      requestAnimationFrame(() => root.querySelector<HTMLInputElement>('.tag-input')?.focus())
  })

  // Note panel toggle
  root.querySelector('.js-note-btn')?.addEventListener('click', () => {
    const notePanel = root.querySelector<HTMLElement>('.note-panel')
    const noteBtn = root.querySelector<HTMLElement>('.js-note-btn')
    if (!notePanel) return
    const isOpen = notePanel.style.display !== 'none'
    root.querySelector<HTMLElement>('.tag-panel')?.style.setProperty('display', 'none')
    root.querySelector<HTMLElement>('.js-tag-btn')?.classList.remove('panel-open')
    notePanel.style.display = isOpen ? 'none' : 'block'
    noteBtn?.classList.toggle('panel-open', !isOpen)
    if (!isOpen)
      requestAnimationFrame(() =>
        root.querySelector<HTMLTextAreaElement>('.note-textarea')?.focus(),
      )
  })

  // Panel close buttons (both tag and note panels)
  root.querySelectorAll('.js-panel-close').forEach((btn) => {
    btn.addEventListener('click', closeAllPanels)
  })

  // Cancel note
  root.querySelector('.js-cancel-note')?.addEventListener('click', () => {
    root.querySelector<HTMLElement>('.note-panel')?.style.setProperty('display', 'none')
    root.querySelector<HTMLElement>('.js-note-btn')?.classList.remove('panel-open')
  })

  // Save note
  root.querySelector('.js-save-note')?.addEventListener('click', () => {
    const body = root.querySelector<HTMLTextAreaElement>('.note-textarea')?.value ?? ''
    root.querySelector<HTMLElement>('.note-panel')?.style.setProperty('display', 'none')
    root.querySelector<HTMLElement>('.js-note-btn')?.classList.remove('panel-open')
    void browser.runtime.sendMessage({
      action: 'toolbar:set-note',
      libraryEntryId,
      body,
    })
  })

  // Add tag — button click
  root.querySelector('.js-add-tag')?.addEventListener('click', () => addTagFromInput())

  // Add tag — Enter key in input
  root.querySelector<HTMLInputElement>('.tag-input')?.addEventListener('keydown', (e) => {
    if (e.key === 'Enter') {
      e.preventDefault()
      addTagFromInput()
    }
  })

  function addTagFromInput(): void {
    const input = root.querySelector<HTMLInputElement>('.tag-input')
    const container = root.querySelector<HTMLElement>('.tag-chips')
    if (!input || !container) return
    const value = input.value.trim()
    if (!value) return
    input.value = ''
    const chip = document.createElement('span')
    chip.className = 'tag-chip'
    chip.dataset.tag = value
    chip.innerHTML = `${escapeHtml(value)}<button class="tag-remove" title="Remove">×</button>`
    container.appendChild(chip)
    syncTags()
    input.focus()
  }

  // Remove tag — event delegation
  root.querySelector('.tag-chips')?.addEventListener('click', (e) => {
    const btn = (e.target as Element).closest('.tag-remove')
    if (!btn) return
    btn.closest('.tag-chip')?.remove()
    syncTags()
  })

  function syncTags(): void {
    const tags = Array.from(root.querySelectorAll<HTMLElement>('.tag-chip'))
      .map((chip) => chip.dataset.tag ?? '')
      .filter(Boolean)
    void browser.runtime.sendMessage({
      action: 'toolbar:set-tags',
      libraryEntryId,
      tags,
    })
  }

  // Favorite / star
  root.querySelector('.js-star-btn')?.addEventListener('click', () => {
    const starBtn = root.querySelector<HTMLElement>('.js-star-btn')
    if (!starBtn) return
    const nowFavorited = starBtn.classList.toggle('starred')
    void browser.runtime.sendMessage({
      action: 'toolbar:patch-item',
      libraryEntryId,
      patch: { is_favorite: nowFavorited },
    })
  })

  // Triage dropdown open/close
  root.querySelector('.js-triage')?.addEventListener('click', (e) => {
    e.stopPropagation()
    const menu = root.querySelector<HTMLElement>('.triage-menu')
    const btn = root.querySelector<HTMLElement>('.js-triage')
    if (!menu || !btn) return
    const isOpen = menu.style.display !== 'none'
    if (isOpen) {
      closeTriage()
      return
    }
    const rect = btn.getBoundingClientRect()
    menu.style.top = `${rect.bottom + 4}px`
    menu.style.left = `${rect.left}px`
    menu.style.display = 'block'
    btn.classList.add('triage-open')
  })

  // Triage item selection
  root.querySelector('.triage-menu')?.addEventListener('click', (e) => {
    e.stopPropagation()
    const clicked = (e.target as Element).closest<HTMLElement>('.triage-item')
    if (!clicked) return
    const value = clicked.dataset.value ?? ''
    if (!value) return
    root.querySelectorAll('.triage-item').forEach((el) => el.classList.remove('active'))
    clicked.classList.add('active')
    const label = root.querySelector<HTMLElement>('.triage-label')
    if (label) label.textContent = triageLabel(value)
    const iconSpan = root.querySelector<HTMLElement>('.triage-ic')
    if (iconSpan) iconSpan.innerHTML = triageIcon(value)
    closeTriage()
    void browser.runtime.sendMessage({
      action: 'toolbar:patch-item',
      libraryEntryId,
      patch: { triage_state: value },
    })
  })
}

function syncHighlightProjection(root: ShadowRoot, state: ToolbarState): void {
  if (state.view !== 'saved') {
    clearProjectedHighlights()
    return
  }

  const highlights = state.highlights ?? []
  if (highlights.length === 0) {
    clearProjectedHighlights()
    return
  }

  const projectedCount = projectHighlights(highlights)
  if (projectedCount === 0) {
    clearProjectedHighlights()
  }
  syncAutoHighlightToggle(root)
}

function setAutoHighlightEnabled(root: ShadowRoot, enabled: boolean): void {
  autoHighlightEnabled = enabled
  if (enabled) installAutoHighlightListeners()
  if (!enabled) {
    pendingAutoHighlightKey = undefined
    if (autoHighlightTimer) clearTimeout(autoHighlightTimer)
    autoHighlightTimer = undefined
  }
  syncAutoHighlightToggle(root)
}

function syncAutoHighlightToggle(root: ShadowRoot): void {
  root.querySelector<HTMLElement>('.js-toggle')?.classList.toggle('off', !autoHighlightEnabled)
}

function installAutoHighlightListeners(): void {
  if (autoHighlightListenersInstalled) return
  autoHighlightListenersInstalled = true

  document.addEventListener('mouseup', scheduleAutoHighlightCapture)
  document.addEventListener('touchend', scheduleAutoHighlightCapture)
  document.addEventListener('keyup', (event) => {
    if (event.key === 'Shift' || event.key.startsWith('Arrow')) {
      scheduleAutoHighlightCapture()
    }
  })
  document.addEventListener('selectionchange', () => {
    if (!currentSelectionKey()) {
      lastAutoHighlightKey = undefined
    }
  })
}

function scheduleAutoHighlightCapture(): void {
  if (!autoHighlightEnabled) return
  if (autoHighlightTimer) clearTimeout(autoHighlightTimer)
  autoHighlightTimer = setTimeout(() => {
    void submitAutoHighlightSelection()
  }, 180)
}

async function submitAutoHighlightSelection(): Promise<void> {
  if (!autoHighlightEnabled) return
  const key = currentSelectionKey()
  if (!key || key === pendingAutoHighlightKey || key === lastAutoHighlightKey) return

  pendingAutoHighlightKey = key
  try {
    const response = (await browser.runtime.sendMessage({
      action: 'toolbar:highlight-selection',
    })) as unknown
    if (isRecord(response) && response.success === true) {
      lastAutoHighlightKey = key
    }
  } catch {
    return
  } finally {
    if (pendingAutoHighlightKey === key) {
      pendingAutoHighlightKey = undefined
    }
  }
}

function currentSelectionKey(): string | undefined {
  const selection = window.getSelection()
  if (!selection || selection.rangeCount === 0) return undefined

  const text = selection.toString().trim()
  if (!text) return undefined

  const range = selection.getRangeAt(0)
  if (!document.body?.contains(range.commonAncestorContainer)) return undefined

  return `${nodePath(range.startContainer)}:${range.startOffset},${nodePath(range.endContainer)}:${
    range.endOffset
  }|${text}`
}
