// @vitest-environment jsdom
import { beforeEach, describe, expect, it, vi } from 'vitest'

import { renderToolbar } from '../lib/full-archive-toolbar'

const sendMessage = vi.fn()

vi.stubGlobal('browser', {
  runtime: { sendMessage },
})

function savedState(note = 'Stored note') {
  return {
    view: 'saved',
    readerUrl: 'https://example.com/reader',
    entry: {
      library_entry_id: 'lib_1',
      document_id: 'doc_1',
      title: 'Example article',
    },
    note,
    highlights: [],
  }
}

function toolbarRoot(): ShadowRoot {
  const root = document.getElementById('indelible-toolbar-host')?.shadowRoot
  if (!root) throw new Error('toolbar root was not rendered')
  return root
}

describe('full archive toolbar', () => {
  beforeEach(() => {
    vi.clearAllMocks()
    vi.useRealTimers()
    document.getElementById('indelible-toolbar-host')?.remove()
    document.body.innerHTML = '<p id="article">Alpha beta</p>'
  })

  it('preserves an open unsaved note across saved-state rerenders', () => {
    renderToolbar(savedState())
    toolbarRoot().querySelector<HTMLElement>('.js-note-btn')?.click()
    const textarea = toolbarRoot().querySelector<HTMLTextAreaElement>('.note-textarea')
    if (!textarea) throw new Error('note textarea was not rendered')
    textarea.value = 'Unsaved draft'

    renderToolbar(savedState('Older stored note'))

    expect(toolbarRoot().querySelector<HTMLTextAreaElement>('.note-textarea')?.value).toBe(
      'Unsaved draft',
    )
    expect(toolbarRoot().querySelector<HTMLElement>('.note-panel')?.style.display).toBe('block')
  })

  it('clears pending auto-highlight state when extension messaging fails', async () => {
    vi.useFakeTimers()
    sendMessage.mockRejectedValue(new Error('extension context invalidated'))
    renderToolbar(savedState())
    toolbarRoot().querySelector<HTMLElement>('.js-toggle')?.click()

    const article = document.getElementById('article')
    if (!article) throw new Error('article fixture was not rendered')
    const range = document.createRange()
    range.selectNodeContents(article)
    const selection = window.getSelection()
    selection?.removeAllRanges()
    selection?.addRange(range)

    document.dispatchEvent(new MouseEvent('mouseup'))
    await vi.advanceTimersByTimeAsync(200)
    document.dispatchEvent(new MouseEvent('mouseup'))
    await vi.advanceTimersByTimeAsync(200)

    expect(sendMessage).toHaveBeenCalledTimes(2)
  })

  it('shows a retryable error when browser-managed authorization cannot start', async () => {
    sendMessage
      .mockResolvedValueOnce({ success: false, error: 'Error: Browser rejected auth' })
      .mockResolvedValueOnce({ success: true })
    renderToolbar({ view: 'disconnected', serverUrl: 'http://localhost:38473' })

    toolbarRoot().querySelector<HTMLButtonElement>('[data-action="connect"]')?.click()
    expect(toolbarRoot().textContent).toContain('Connecting Indelible')

    await vi.waitFor(() => expect(toolbarRoot().textContent).toContain('Browser rejected auth'))
    expect(
      toolbarRoot().querySelector<HTMLButtonElement>('[data-action="connect"]')?.textContent,
    ).toBe('Try again')

    toolbarRoot().querySelector<HTMLButtonElement>('[data-action="connect"]')?.click()
    expect(toolbarRoot().textContent).toContain('Connecting Indelible')
    await vi.waitFor(() => expect(sendMessage).toHaveBeenCalledTimes(2))
  })

  it('shows the unreachable endpoint and retries without asking to reconnect', async () => {
    renderToolbar({ view: 'unreachable', serverUrl: 'http://localhost:38481' })

    expect(toolbarRoot().textContent).toContain('Indelible is unreachable')
    expect(toolbarRoot().textContent).toContain('http://localhost:38481')
    expect(toolbarRoot().textContent).not.toContain('Authenticate your workspace')
    toolbarRoot().querySelector<HTMLButtonElement>('[data-action="refresh"]')?.click()
    await vi.waitFor(() => expect(sendMessage).toHaveBeenCalledWith({ action: 'toolbar:save' }))
  })
})
