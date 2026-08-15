import { describe, it, expect, beforeEach, vi } from 'vitest'

const EXTENSION_ID = 'indelible-ext-id'

type MessageHandler = (
  message: unknown,
  sender: { id?: string; url?: string; tab?: { id?: number; url?: string; title?: string } },
  sendResponse: (response: unknown) => void,
) => boolean | undefined

type ActionClickHandler = (tab: { id?: number; url?: string; title?: string }) => void

let registeredMessageHandler: MessageHandler | null = null
let registeredActionClickHandler: ActionClickHandler | null = null

const mockStorage: Record<string, unknown> = {}

vi.stubGlobal('browser', {
  runtime: {
    id: EXTENSION_ID,
    onInstalled: { addListener: vi.fn() },
    onMessage: {
      addListener: vi.fn((fn: MessageHandler) => {
        registeredMessageHandler = fn
      }),
    },
    onConnect: { addListener: vi.fn() },
  },
  contextMenus: {
    removeAll: vi.fn(async () => undefined),
    create: vi.fn(),
    onClicked: { addListener: vi.fn() },
  },
  commands: {
    onCommand: { addListener: vi.fn() },
  },
  action: {
    onClicked: {
      addListener: vi.fn((fn: ActionClickHandler) => {
        registeredActionClickHandler = fn
      }),
    },
    setBadgeText: vi.fn(async () => undefined),
    setBadgeBackgroundColor: vi.fn(async () => undefined),
    setTitle: vi.fn(async () => undefined),
  },
  storage: {
    local: {
      get: vi.fn(async (keys: string | string[]) => {
        const keyList = Array.isArray(keys) ? keys : [keys]
        const result: Record<string, unknown> = {}
        for (const key of keyList) {
          if (key in mockStorage) {
            result[key] = mockStorage[key]
          }
        }
        return result
      }),
      set: vi.fn(async (items: Record<string, unknown>) => {
        Object.assign(mockStorage, items)
      }),
      remove: vi.fn(async (keys: string[]) => {
        const keyList = Array.isArray(keys) ? keys : [keys]
        for (const key of keyList) {
          // eslint-disable-next-line @typescript-eslint/no-dynamic-delete
          delete mockStorage[key]
        }
      }),
    },
  },
  identity: {
    getRedirectURL: vi.fn(() => 'https://abcdefghijklmnop.chromiumapp.org/indelible'),
    launchWebAuthFlow: vi.fn(),
  },
  tabs: {
    create: vi.fn(async () => ({ id: 1 })),
    remove: vi.fn(async () => undefined),
    query: vi.fn(),
    executeScript: vi.fn(async () => undefined),
    sendMessage: vi.fn(),
  },
  scripting: {
    executeScript: vi.fn(async () => undefined),
  },
})

const fetchMock = vi.fn()
vi.stubGlobal('fetch', fetchMock)

vi.stubGlobal('defineBackground', (fn: () => void) => fn())

await import('../entrypoints/background')

import { clearPendingPkce } from '../lib/auth'
import { clearAccessTokenMemory, setAccessTokenMemory } from '../lib/api'

function getHandler(): MessageHandler {
  if (!registeredMessageHandler) {
    throw new Error('Message handler was not registered')
  }
  return registeredMessageHandler
}

function sendMessage(
  message: unknown,
  sender: { id?: string; url?: string; tab?: { id?: number; url?: string; title?: string } },
): Promise<{ success: boolean; error?: string; data?: unknown }> {
  return new Promise((resolve) => {
    getHandler()(message, sender, (response) => {
      resolve(response as { success: boolean; error?: string; data?: unknown })
    })
  })
}

function getActionClickHandler(): ActionClickHandler {
  if (!registeredActionClickHandler) {
    throw new Error('Action click handler was not registered')
  }
  return registeredActionClickHandler
}

function clearMockStorage(): void {
  for (const key of Object.keys(mockStorage)) {
    // eslint-disable-next-line @typescript-eslint/no-dynamic-delete
    delete mockStorage[key]
  }
}

const FUTURE_EXPIRY = Math.floor(Date.now() / 1000) + 3600

function makeRefreshResponse() {
  return new Response(
    JSON.stringify({
      access_token: 'jwt_new',
      refresh_token: 'indr_new',
      expires_at: FUTURE_EXPIRY,
      token_type: 'Bearer',
    }),
    { status: 200 },
  )
}

describe('background message handler', () => {
  beforeEach(async () => {
    clearMockStorage()
    vi.clearAllMocks()
    ;(browser.tabs.query as ReturnType<typeof vi.fn>).mockReset()
    ;(browser.tabs.sendMessage as ReturnType<typeof vi.fn>).mockReset()
    ;(browser.tabs.executeScript as ReturnType<typeof vi.fn>)
      .mockReset()
      .mockResolvedValue(undefined)
    ;(browser.scripting.executeScript as ReturnType<typeof vi.fn>)
      .mockReset()
      .mockResolvedValue(undefined)
    fetchMock.mockReset()
    clearAccessTokenMemory()
    await clearPendingPkce()
    ;(browser.identity.launchWebAuthFlow as ReturnType<typeof vi.fn>).mockImplementation(
      async ({ url }: { url: string }) => {
        const state = new URL(url).searchParams.get('state')
        return `https://abcdefghijklmnop.chromiumapp.org/indelible?code=auth_code_123&state=${state}`
      },
    )
  })

  describe('browser action', () => {
    it('injects the full-archive content script before rendering the toolbar', async () => {
      ;(browser.tabs.sendMessage as ReturnType<typeof vi.fn>)
        .mockRejectedValueOnce(new Error('No receiver'))
        .mockResolvedValueOnce({ success: true })

      getActionClickHandler()({
        id: 42,
        url: 'https://example.com/article',
        title: 'Example article',
      })

      await vi.waitFor(() => {
        expect(browser.scripting.executeScript).toHaveBeenCalledWith({
          target: { tabId: 42 },
          files: ['/content-scripts/full-archive.js'],
        })
      })
      expect(browser.tabs.executeScript).not.toHaveBeenCalled()
      expect(browser.tabs.sendMessage).toHaveBeenNthCalledWith(1, 42, {
        action: 'indelible:ping',
      })
      expect(browser.tabs.sendMessage).toHaveBeenNthCalledWith(
        2,
        42,
        expect.objectContaining({
          action: 'toolbar:render',
          state: expect.objectContaining({ view: 'disconnected' }),
        }),
      )
    })

    it('renders a retryable unreachable state while retaining the connected session', async () => {
      mockStorage['ind_refresh_token'] = 'indr_valid_token'
      mockStorage['ind_server_url'] = 'http://localhost:38481'
      fetchMock.mockRejectedValueOnce(new TypeError('Failed to fetch'))
      ;(browser.tabs.sendMessage as ReturnType<typeof vi.fn>).mockResolvedValue({ success: true })

      getActionClickHandler()({
        id: 42,
        url: 'https://example.com/article',
        title: 'Example article',
      })

      await vi.waitFor(() =>
        expect(browser.tabs.sendMessage).toHaveBeenCalledWith(
          42,
          expect.objectContaining({
            action: 'toolbar:render',
            state: expect.objectContaining({
              view: 'unreachable',
              serverUrl: 'http://localhost:38481',
            }),
          }),
        ),
      )
      expect(mockStorage['ind_refresh_token']).toBe('indr_valid_token')
    })
  })

  describe('capture admission', () => {
    it('continues into capture for an external web origin', async () => {
      mockStorage['ind_server_url'] = 'http://localhost:38473'
      ;(browser.tabs.query as ReturnType<typeof vi.fn>).mockResolvedValueOnce([
        {
          id: 42,
          url: 'https://example.com/article',
          title: 'Example article',
        },
      ])
      ;(browser.tabs.sendMessage as ReturnType<typeof vi.fn>).mockImplementation(
        async (_tabId: number, message: { action: string }) => {
          if (message.action === 'indelible:ping' || message.action === 'toolbar:render') {
            return { success: true }
          }
          if (message.action === 'capture:run') {
            return { action: 'capture:error', message: 'controlled-stop' }
          }
          return undefined
        },
      )

      const response = await sendMessage({ action: 'toolbar:save' }, { id: EXTENSION_ID })

      expect(response).toEqual({ success: false, error: 'controlled-stop' })
      expect(browser.tabs.sendMessage).toHaveBeenCalledWith(42, { action: 'capture:run' })
    })

    it('renders the recoverable unreachable view when a save fails because the server is down', async () => {
      mockStorage['ind_server_url'] = 'http://localhost:38481'
      ;(browser.tabs.query as ReturnType<typeof vi.fn>).mockResolvedValueOnce([
        {
          id: 42,
          url: 'https://example.com/article',
          title: 'Example article',
        },
      ])
      ;(browser.tabs.sendMessage as ReturnType<typeof vi.fn>).mockImplementation(
        async (_tabId: number, message: { action: string }) => {
          if (message.action === 'indelible:ping' || message.action === 'toolbar:render') {
            return { success: true }
          }
          if (message.action === 'capture:run') {
            return { action: 'capture:error', message: 'Indelible server is unreachable' }
          }
          return undefined
        },
      )

      await sendMessage({ action: 'toolbar:save' }, { id: EXTENSION_ID })

      expect(browser.tabs.sendMessage).toHaveBeenCalledWith(
        42,
        expect.objectContaining({
          action: 'toolbar:render',
          state: expect.objectContaining({
            view: 'unreachable',
            serverUrl: 'http://localhost:38481',
          }),
        }),
      )
    })

    it('rejects a configured-origin document returned after external-tab admission', async () => {
      mockStorage['ind_server_url'] = 'http://localhost:38473'
      setAccessTokenMemory('jwt_token', FUTURE_EXPIRY)
      ;(browser.tabs.query as ReturnType<typeof vi.fn>).mockResolvedValueOnce([
        {
          id: 42,
          url: 'https://example.com/article',
          title: 'Example article',
        },
      ])
      ;(browser.tabs.sendMessage as ReturnType<typeof vi.fn>).mockImplementation(
        async (_tabId: number, message: { action: string }) => {
          if (message.action === 'indelible:ping' || message.action === 'toolbar:render') {
            return { success: true }
          }
          if (message.action === 'capture:run') {
            mockStorage['ind_server_url'] = 'https://new-indelible.example.com'
            return {
              action: 'capture:result',
              payload: {
                url: 'https://new-indelible.example.com/library',
                title: 'Indelible library',
                readerHtml: '<main>Library</main>',
                htmlBase64: 'PG1haW4+TGlicmFyeTwvbWFpbj4=',
              },
            }
          }
          return undefined
        },
      )
      fetchMock.mockResolvedValue(new Response('unexpected write', { status: 500 }))

      const response = await sendMessage({ action: 'toolbar:save' }, { id: EXTENSION_ID })

      expect(response).toEqual({ success: false, error: 'This page cannot be saved' })
      expect(fetchMock).not.toHaveBeenCalled()
    })

    it('uses returned document metadata for direct reader fallback', async () => {
      let readerSaveBody: unknown

      mockStorage['ind_server_url'] = 'http://localhost:38473'
      setAccessTokenMemory('jwt_token', FUTURE_EXPIRY)
      ;(browser.tabs.query as ReturnType<typeof vi.fn>).mockResolvedValueOnce([
        {
          id: 42,
          url: 'https://example.com/article-a',
          title: 'Article A',
        },
      ])
      ;(browser.tabs.sendMessage as ReturnType<typeof vi.fn>).mockImplementation(
        async (_tabId: number, message: { action: string }) => {
          if (message.action === 'indelible:ping' || message.action === 'toolbar:render') {
            return { success: true }
          }
          if (message.action === 'capture:run') {
            return {
              action: 'capture:error',
              message: 'singlefile-failed',
              payload: {
                url: 'https://example.net/article-b',
                title: 'Article B',
                readerHtml: '<main>Article B</main>',
                htmlBase64: '',
              },
            }
          }
          return undefined
        },
      )
      fetchMock.mockImplementation(async (input: Request | string | URL, init?: RequestInit) => {
        const request = input instanceof Request ? input : new Request(input, init)
        if (request.url.endsWith('/api/v1/extension/reader-save')) {
          readerSaveBody = JSON.parse(await request.text())
        }
        return new Response('controlled-stop', { status: 500 })
      })

      await sendMessage({ action: 'toolbar:save' }, { id: EXTENSION_ID })

      expect(readerSaveBody).toMatchObject({
        url: 'https://example.net/article-b',
        title: 'Article B',
      })
      expect(fetchMock).toHaveBeenCalledOnce()
    })
  })

  it('rejects messages from untrusted senders', async () => {
    const response = await sendMessage({ action: 'auth:status' }, { id: 'some-other-extension-id' })
    expect(response.success).toBe(false)
    expect(response.error).toBe('Untrusted sender')
  })

  it('rejects messages with no sender id', async () => {
    const response = await sendMessage({ action: 'auth:status' }, {})
    expect(response.success).toBe(false)
    expect(response.error).toBe('Untrusted sender')
  })

  it('accepts messages from the extension itself', async () => {
    const response = await sendMessage({ action: 'auth:status' }, { id: EXTENSION_ID })
    expect(response.success).toBe(true)
  })

  it('rejects invalid message format from trusted sender', async () => {
    const response = await sendMessage('not-an-object', { id: EXTENSION_ID })
    expect(response.success).toBe(false)
    expect(response.error).toBe('Invalid message format')
  })

  it('rejects unknown action from trusted sender', async () => {
    const response = await sendMessage({ action: 'unknown:action' }, { id: EXTENSION_ID })
    expect(response.success).toBe(false)
    expect(response.error).toContain('Unknown action')
  })

  it('returns queued and cooldown document reprocess results to the toolbar', async () => {
    setAccessTokenMemory('jwt_token', FUTURE_EXPIRY)
    fetchMock.mockResolvedValueOnce(
      new Response(
        JSON.stringify({
          queued: false,
          job_type: 'document.reprocess',
          retry_after_seconds: 45,
        }),
        { status: 200, headers: { 'Content-Type': 'application/json' } },
      ),
    )

    const response = await sendMessage(
      { action: 'toolbar:reprocess-document', documentId: 'doc_123' },
      { id: EXTENSION_ID },
    )

    expect(response).toEqual({
      success: true,
      data: {
        queued: false,
        job_type: 'document.reprocess',
        retry_after_seconds: 45,
      },
    })
  })

  describe('auth:status', () => {
    it('returns disconnected state when no token is stored', async () => {
      const response = await sendMessage({ action: 'auth:status' }, { id: EXTENSION_ID })
      expect(response.success).toBe(true)

      const data = response.data as { status: string; serverUrl: string }
      expect(data.status).toBe('disconnected')
      expect(data.serverUrl).toBe('https://useindelible.com')
    })

    it('returns connected state with user info when server validates token', async () => {
      mockStorage['ind_refresh_token'] = 'indr_test_token'
      mockStorage['ind_connected_at'] = '2026-03-01T00:00:00.000Z'

      fetchMock.mockResolvedValueOnce(makeRefreshResponse())
      fetchMock.mockResolvedValueOnce(
        new Response(
          JSON.stringify({
            authenticated: true,
            user: {
              id: 'user_123',
              email: 'test@example.com',
              display_name: 'Test User',
            },
          }),
          { status: 200, headers: { 'Content-Type': 'application/json' } },
        ),
      )

      const response = await sendMessage({ action: 'auth:status' }, { id: EXTENSION_ID })
      expect(response.success).toBe(true)

      const data = response.data as {
        status: string
        connectedAt: string
        user?: { id: string; email: string; displayName: string }
      }
      expect(data.status).toBe('connected')
      expect(data.connectedAt).toBe('2026-03-01T00:00:00.000Z')
      expect(data.user).toEqual({
        id: 'user_123',
        email: 'test@example.com',
        displayName: 'Test User',
      })
      const statusCall = fetchMock.mock.calls.find(
        (call) => call[0] instanceof Request && call[0].url.endsWith('/api/v1/extension/status'),
      )
      expect(statusCall).toBeDefined()
      expect((statusCall![0] as Request).headers.get('Authorization')).toMatch(/^Bearer /)
    })
  })

  describe('auth:start', () => {
    it('launches browser-managed auth with the PKCE start URL', async () => {
      fetchMock.mockResolvedValueOnce(makeRefreshResponse())
      const response = await sendMessage(
        { action: 'auth:start', serverUrl: 'https://my-indelible.com' },
        { id: EXTENSION_ID },
      )
      expect(response.success).toBe(true)
      expect(browser.identity.launchWebAuthFlow).toHaveBeenCalledOnce()
      expect(browser.tabs.create).not.toHaveBeenCalled()

      const [{ url }] = (browser.identity.launchWebAuthFlow as ReturnType<typeof vi.fn>).mock
        .calls[0] as [{ url: string }]
      const parsed = new URL(url)
      expect(parsed.origin).toBe('https://my-indelible.com')
      expect(parsed.pathname).toBe('/api/v1/auth/extension/start')
      expect(parsed.searchParams.get('code_challenge')).toBeTruthy()
      expect(parsed.searchParams.get('state')).toBeTruthy()
      expect(parsed.searchParams.get('redirect_uri')).toBe(
        'https://abcdefghijklmnop.chromiumapp.org/indelible',
      )
    })

    it('fails when server URL is missing', async () => {
      const response = await sendMessage({ action: 'auth:start' }, { id: EXTENSION_ID })
      expect(response.success).toBe(false)
      expect(response.error).toBe('Server URL is required')
    })

    it('stores the server URL', async () => {
      fetchMock.mockResolvedValueOnce(makeRefreshResponse())
      await sendMessage(
        { action: 'auth:start', serverUrl: 'https://custom.example.com' },
        { id: EXTENSION_ID },
      )
      expect(mockStorage['ind_server_url']).toBe('https://custom.example.com')
    })
  })

  describe('auth:logout', () => {
    it('clears stored tokens', async () => {
      mockStorage['ind_refresh_token'] = 'indr_to_remove'
      mockStorage['ind_connected_at'] = '2026-03-01T00:00:00.000Z'

      fetchMock.mockResolvedValueOnce(new Response('', { status: 200 }))

      const response = await sendMessage({ action: 'auth:logout' }, { id: EXTENSION_ID })
      expect(response.success).toBe(true)
      expect(mockStorage['ind_refresh_token']).toBeUndefined()
      expect(mockStorage['ind_connected_at']).toBeUndefined()
    })

    it('clears stored tokens even when the server cannot be reached to revoke them', async () => {
      mockStorage['ind_refresh_token'] = 'indr_to_remove'
      mockStorage['ind_server_url'] = 'http://localhost:38481'
      fetchMock.mockRejectedValueOnce(new TypeError('Failed to fetch'))

      const response = await sendMessage({ action: 'auth:logout' }, { id: EXTENSION_ID })

      expect(response.success).toBe(true)
      expect(mockStorage['ind_refresh_token']).toBeUndefined()
    })
  })

  describe('toolbar:set-server-url', () => {
    it('persists a normalized server address', async () => {
      mockStorage['ind_server_url'] = 'http://localhost:38481'

      const response = await sendMessage(
        { action: 'toolbar:set-server-url', serverUrl: 'http://localhost:38999/' },
        { id: EXTENSION_ID },
      )

      expect(response.success).toBe(true)
      expect(mockStorage['ind_server_url']).toBe('http://localhost:38999')
    })

    it('rejects an invalid address and keeps the stored one', async () => {
      mockStorage['ind_server_url'] = 'http://localhost:38481'

      const response = await sendMessage(
        { action: 'toolbar:set-server-url', serverUrl: 'ftp://localhost:38999' },
        { id: EXTENSION_ID },
      )

      expect(response.success).toBe(false)
      expect(response.error).toContain('http or https')
      expect(mockStorage['ind_server_url']).toBe('http://localhost:38481')
    })
  })

  describe('auth:validate', () => {
    it('returns valid: false when no refresh token exists', async () => {
      const response = await sendMessage({ action: 'auth:validate' }, { id: EXTENSION_ID })
      expect(response.success).toBe(true)

      const data = response.data as { valid: boolean }
      expect(data.valid).toBe(false)
    })

    it('returns valid: true when refresh succeeds', async () => {
      mockStorage['ind_refresh_token'] = 'indr_valid_token'
      mockStorage['ind_server_url'] = 'https://test.useindelible.com'

      fetchMock.mockResolvedValueOnce(makeRefreshResponse())

      const response = await sendMessage({ action: 'auth:validate' }, { id: EXTENSION_ID })
      expect(response.success).toBe(true)

      const data = response.data as { valid: boolean }
      expect(data.valid).toBe(true)
    })

    it('returns valid: false and clears token when refresh fails', async () => {
      mockStorage['ind_refresh_token'] = 'indr_expired_token'
      mockStorage['ind_server_url'] = 'https://test.useindelible.com'

      fetchMock.mockResolvedValueOnce(new Response('Unauthorized', { status: 401 }))

      const response = await sendMessage({ action: 'auth:validate' }, { id: EXTENSION_ID })
      expect(response.success).toBe(true)

      const data = response.data as { valid: boolean }
      expect(data.valid).toBe(false)
      expect(mockStorage['ind_refresh_token']).toBeUndefined()
    })
  })

  describe('toolbar:highlight-selection', () => {
    it('does not write a highlight for a configured-origin selection returned after admission', async () => {
      const tab = {
        id: 42,
        url: 'https://example.com/article',
        title: 'Example article',
      }
      let highlightWriteAttempted = false
      let checkedUrl: string | null = null

      mockStorage['ind_server_url'] = 'http://localhost:38473'
      setAccessTokenMemory('jwt_token', FUTURE_EXPIRY)
      ;(browser.tabs.query as ReturnType<typeof vi.fn>).mockResolvedValueOnce([tab])
      ;(browser.tabs.sendMessage as ReturnType<typeof vi.fn>).mockImplementation(
        async (_tabId: number, message: { action: string }) => {
          if (message.action === 'indelible:ping') return { success: true }
          if (message.action === 'selection:capture') {
            return {
              action: 'selection:result',
              payload: {
                text: 'Library',
                sourceLocator: {
                  type: 'web_page_dom_range',
                  url: 'https://new-indelible.example.com/library',
                  location: 'body > main:0,body > main:7',
                  text_content: 'Library',
                },
              },
            }
          }
          return undefined
        },
      )
      fetchMock.mockImplementation(async (input: Request | string | URL) => {
        const request = input instanceof Request ? input : new Request(input)
        if (request.url.includes('/api/v1/extension/check-url')) {
          checkedUrl = new URL(request.url).searchParams.get('url')
          mockStorage['ind_server_url'] = 'https://new-indelible.example.com'
          return new Response(
            JSON.stringify({
              exists: true,
              library_entry_id: 'lib_018f2f1a-1111-7222-8333-111111111111',
            }),
            { status: 200, headers: { 'Content-Type': 'application/json' } },
          )
        }
        if (request.url.includes('/assets/readable_html')) {
          return new Response('', { status: 404 })
        }
        if (request.method === 'POST' && request.url.includes('/highlights')) {
          highlightWriteAttempted = true
          return new Response('unexpected write', { status: 500 })
        }
        return new Response('unexpected request', { status: 500 })
      })

      await sendMessage({ action: 'toolbar:highlight-selection' }, { id: EXTENSION_ID })

      expect(checkedUrl).toBe('https://new-indelible.example.com/library')
      expect(highlightWriteAttempted).toBe(false)
    })

    it('uses flattened document asset responses to build reader locators', async () => {
      const libraryEntryId = 'lib_018f2f1a-1111-7222-8333-111111111111'
      const tab = {
        id: 42,
        url: 'https://example.com/article',
        title: 'Reader article',
      }
      let highlightBody: unknown

      mockStorage['ind_server_url'] = 'https://test.useindelible.com'
      setAccessTokenMemory('jwt_token', FUTURE_EXPIRY)
      ;(browser.tabs.query as ReturnType<typeof vi.fn>).mockResolvedValueOnce([tab])
      ;(browser.tabs.sendMessage as ReturnType<typeof vi.fn>).mockImplementation(
        async (_tabId: number, message: { action: string }) => {
          if (message.action === 'selection:capture') {
            return {
              action: 'selection:result',
              payload: {
                text: 'selected phrase',
                sourceLocator: {
                  type: 'web_page_dom_range',
                  url: tab.url,
                  location: 'body > article',
                  text_content: 'selected phrase',
                },
              },
            }
          }
          return undefined
        },
      )

      const jsonHeaders = { 'Content-Type': 'application/json' }

      fetchMock.mockImplementation(async (input: Request | string | URL, init?: RequestInit) => {
        const request = input instanceof Request ? input : undefined
        const href = request ? request.url : String(input)
        const method = request?.method ?? init?.method

        if (href.includes('/api/v1/extension/check-url')) {
          return new Response(
            JSON.stringify({
              exists: true,
              library_entry_id: libraryEntryId,
              document_id: 'doc_018f2f1a-1111-7222-8333-111111111111',
              title: tab.title,
              saved_at: '2026-06-03T00:00:00Z',
              triage_state: 'inbox',
            }),
            { status: 200, headers: jsonHeaders },
          )
        }

        if (href.endsWith(`/api/v1/extension/entries/${libraryEntryId}/assets/readable_html`)) {
          return new Response(
            JSON.stringify({
              download_url:
                'https://test.useindelible.com/api/v1/assets/documents/doc_018f2f1a-1111-7222-8333-222222222222/readable_html',
            }),
            { status: 200, headers: jsonHeaders },
          )
        }

        if (
          href.endsWith(
            '/api/v1/assets/documents/doc_018f2f1a-1111-7222-8333-222222222222/readable_html',
          )
        ) {
          return new Response('<article>Before selected phrase after.</article>', { status: 200 })
        }

        if (href.endsWith(`/api/v1/extension/entries/${libraryEntryId}/highlights`)) {
          if (method === 'POST') {
            highlightBody = JSON.parse(await (request ?? new Request(href, init)).text())
            return new Response(
              JSON.stringify({
                id: 'hl_018f2f1a-1111-7222-8333-222222222222',
                document_id: 'doc_018f2f1a-1111-7222-8333-222222222222',
                color: 'yellow',
                text_content: 'selected phrase',
                locator: (highlightBody as { locator?: unknown }).locator,
                source_locator: (highlightBody as { source_locator?: unknown }).source_locator,
                created_at: '2026-06-03T00:00:00Z',
                updated_at: '2026-06-03T00:00:00Z',
              }),
              { status: 201, headers: jsonHeaders },
            )
          }
          return new Response(JSON.stringify({ highlights: [], count: 0 }), {
            status: 200,
            headers: jsonHeaders,
          })
        }

        if (href.endsWith(`/api/v1/extension/entries/${libraryEntryId}`)) {
          return new Response(
            JSON.stringify({
              library_entry_id: libraryEntryId,
              document_id: 'doc_018f2f1a-1111-7222-8333-111111111111',
              title: tab.title,
              url: tab.url,
              triage_state: 'inbox',
              is_favorite: false,
              saved_at: '2026-06-03T00:00:00Z',
              reader_url: `https://test.useindelible.com/read/${libraryEntryId}`,
              tags: [],
              note: null,
            }),
            { status: 200, headers: jsonHeaders },
          )
        }

        return new Response('unexpected request', { status: 500 })
      })

      const response = await sendMessage(
        { action: 'toolbar:highlight-selection' },
        { id: EXTENSION_ID },
      )

      expect(response.success).toBe(true)
      expect(highlightBody).toMatchObject({
        color: 'yellow',
        text_content: 'selected phrase',
        locator: {
          type: 'html',
          start_offset: 7,
          end_offset: 22,
        },
      })
      expect(fetchMock).not.toHaveBeenCalledWith(
        expect.stringContaining(`/api/v1/assets/${libraryEntryId}/readable_html`),
        expect.anything(),
      )
    })
  })
})
