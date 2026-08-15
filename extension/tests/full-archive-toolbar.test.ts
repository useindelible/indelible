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

function serverUrlInput(): HTMLInputElement {
  const input = toolbarRoot().querySelector<HTMLInputElement>('[data-role="server-url"]')
  if (!input) throw new Error('server url input was not rendered')
  return input
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

  it('shows the unreachable endpoint with an editable address and a sign-out escape hatch', () => {
    renderToolbar({ view: 'unreachable', serverUrl: 'http://localhost:38481' })

    expect(toolbarRoot().textContent).toContain('Indelible is unreachable')
    expect(serverUrlInput().value).toBe('http://localhost:38481')
    expect(toolbarRoot().querySelector('[data-action="retry"]')).not.toBeNull()
    expect(toolbarRoot().querySelector('[data-action="sign-out"]')).not.toBeNull()
    expect(toolbarRoot().textContent).not.toContain('Authenticate your workspace')
  })

  it('persists an edited server address and resumes the save when the session still works', async () => {
    sendMessage
      .mockResolvedValueOnce({ success: true })
      .mockResolvedValueOnce({ success: true, data: { status: 'connected' } })
      .mockResolvedValueOnce({ success: true })
    renderToolbar({ view: 'unreachable', serverUrl: 'http://localhost:38481' })
    serverUrlInput().value = 'http://localhost:38999'

    toolbarRoot().querySelector<HTMLButtonElement>('[data-action="retry"]')?.click()

    await vi.waitFor(() => expect(sendMessage).toHaveBeenCalledWith({ action: 'toolbar:save' }))
    expect(sendMessage).toHaveBeenNthCalledWith(1, {
      action: 'toolbar:set-server-url',
      serverUrl: 'http://localhost:38999',
    })
    expect(sendMessage).toHaveBeenNthCalledWith(2, { action: 'auth:status' })
  })

  it('re-checks the unchanged server address without rewriting it', async () => {
    sendMessage.mockResolvedValueOnce({
      success: true,
      data: { status: 'error', message: 'Indelible server is unreachable' },
    })
    renderToolbar({ view: 'unreachable', serverUrl: 'http://localhost:38481' })

    toolbarRoot().querySelector<HTMLButtonElement>('[data-action="retry"]')?.click()

    await vi.waitFor(() =>
      expect(toolbarRoot().textContent).toContain('Indelible server is unreachable'),
    )
    expect(sendMessage).toHaveBeenCalledTimes(1)
    expect(sendMessage).toHaveBeenCalledWith({ action: 'auth:status' })
    expect(toolbarRoot().querySelector<HTMLButtonElement>('[data-action="retry"]')?.disabled).toBe(
      false,
    )
  })

  it('falls back to the connect flow when the retained session is no longer accepted', async () => {
    sendMessage.mockResolvedValueOnce({ success: true, data: { status: 'disconnected' } })
    renderToolbar({ view: 'unreachable', serverUrl: 'http://localhost:38481' })

    toolbarRoot().querySelector<HTMLButtonElement>('[data-action="retry"]')?.click()

    await vi.waitFor(() => expect(toolbarRoot().textContent).toContain('Connect Indelible'))
    expect(serverUrlInput().value).toBe('http://localhost:38481')
  })

  it('reports a rejected server address instead of re-checking the session', async () => {
    sendMessage.mockResolvedValueOnce({
      success: false,
      error: 'Server URL must use http or https',
    })
    renderToolbar({ view: 'unreachable', serverUrl: 'http://localhost:38481' })
    serverUrlInput().value = 'ftp://localhost:38999'

    toolbarRoot().querySelector<HTMLButtonElement>('[data-action="retry"]')?.click()

    await vi.waitFor(() =>
      expect(toolbarRoot().textContent).toContain('Server URL must use http or https'),
    )
    expect(sendMessage).toHaveBeenCalledTimes(1)
  })

  it('signs out from the unreachable state so a stale session can be replaced', async () => {
    sendMessage.mockResolvedValueOnce({ success: true })
    renderToolbar({ view: 'unreachable', serverUrl: 'http://localhost:38481' })

    toolbarRoot().querySelector<HTMLButtonElement>('[data-action="sign-out"]')?.click()

    await vi.waitFor(() => expect(toolbarRoot().textContent).toContain('Connect Indelible'))
    expect(sendMessage).toHaveBeenCalledWith({ action: 'auth:logout' })
  })

  it('keeps the session visible when sign-out fails', async () => {
    sendMessage.mockResolvedValueOnce({ success: false, error: 'Error: Storage is unavailable' })
    renderToolbar({ view: 'unreachable', serverUrl: 'http://localhost:38481' })

    toolbarRoot().querySelector<HTMLButtonElement>('[data-action="sign-out"]')?.click()

    await vi.waitFor(() => expect(toolbarRoot().textContent).toContain('Storage is unavailable'))
    expect(toolbarRoot().textContent).toContain('Indelible is unreachable')
  })

  it('still refreshes an already-saved page without touching the connection', async () => {
    renderToolbar({ view: 'already-saved', serverUrl: 'http://localhost:38481' })

    toolbarRoot().querySelector<HTMLButtonElement>('[data-action="refresh"]')?.click()

    await vi.waitFor(() => expect(sendMessage).toHaveBeenCalledWith({ action: 'toolbar:save' }))
  })
})
