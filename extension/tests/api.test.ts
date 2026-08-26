import { describe, it, expect, beforeEach, vi } from 'vitest'

const mockStorage: Record<string, unknown> = {}

vi.stubGlobal('browser', {
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
        for (const key of keys) {
          // eslint-disable-next-line @typescript-eslint/no-dynamic-delete
          delete mockStorage[key]
        }
      }),
    },
  },
})

const fetchMock = vi.fn()
vi.stubGlobal('fetch', fetchMock)

import {
  authenticatedFetch,
  getExtensionStatus,
  getExtensionSavedEntry,
  checkExtensionUrl,
  patchExtensionSavedEntry,
  upsertEntryNote,
  syncEntryTags,
  getPipelineStatus,
  reprocessDocument,
  refreshAccessToken,
  setAccessTokenMemory,
  clearAccessTokenMemory,
} from '../lib/api'

function clearMockStorage(): void {
  for (const key of Object.keys(mockStorage)) {
    // eslint-disable-next-line @typescript-eslint/no-dynamic-delete
    delete mockStorage[key]
  }
}

const FUTURE_EXPIRY = Math.floor(Date.now() / 1000) + 3600

describe('api', () => {
  beforeEach(() => {
    clearMockStorage()
    vi.clearAllMocks()
    clearAccessTokenMemory()
    mockStorage['ind_server_url'] = 'https://test.useindelible.com'
  })

  describe('authenticatedFetch', () => {
    it('retains the refresh token and reports an unavailable server on refresh 503', async () => {
      mockStorage['ind_refresh_token'] = 'indr_valid_token'
      fetchMock.mockResolvedValueOnce(new Response('Unavailable', { status: 503 }))

      await expect(refreshAccessToken()).rejects.toThrow('server is unavailable')
      expect(mockStorage['ind_refresh_token']).toBe('indr_valid_token')
    })

    it('clears a rejected refresh token when the backend returns its current 400 contract', async () => {
      mockStorage['ind_refresh_token'] = 'indr_expired'
      fetchMock.mockResolvedValueOnce(new Response('Invalid token', { status: 400 }))

      expect(await refreshAccessToken()).toBe(false)
      expect(mockStorage['ind_refresh_token']).toBeUndefined()
    })

    it('includes Authorization header with Bearer token', async () => {
      setAccessTokenMemory('jwt_access_token', FUTURE_EXPIRY)
      fetchMock.mockResolvedValueOnce(new Response('{}', { status: 200 }))

      await authenticatedFetch('/api/v1/test')

      expect(fetchMock).toHaveBeenCalledOnce()
      const [url, init] = fetchMock.mock.calls[0] as [string, RequestInit]
      expect(url).toBe('https://test.useindelible.com/api/v1/test')
      const headers = init.headers as Headers
      expect(headers.get('Authorization')).toBe('Bearer jwt_access_token')
    })

    it('includes Accept: application/json header', async () => {
      setAccessTokenMemory('jwt_access_token', FUTURE_EXPIRY)
      fetchMock.mockResolvedValueOnce(new Response('{}', { status: 200 }))

      await authenticatedFetch('/api/v1/test')

      const [, init] = fetchMock.mock.calls[0] as [string, RequestInit]
      const headers = init.headers as Headers
      expect(headers.get('Accept')).toBe('application/json')
    })

    it('throws when no valid token is available and refresh fails', async () => {
      // No in-memory token, no refresh token in storage
      await expect(authenticatedFetch('/api/v1/test')).rejects.toThrow('No valid authentication')
      expect(fetchMock).not.toHaveBeenCalled()
    })

    it('refreshes access token when in-memory token is absent', async () => {
      mockStorage['ind_refresh_token'] = 'indr_valid_token'
      // refresh endpoint
      fetchMock.mockResolvedValueOnce(
        new Response(
          JSON.stringify({
            access_token: 'jwt_new',
            refresh_token: 'indr_new',
            expires_at: FUTURE_EXPIRY,
            token_type: 'Bearer',
          }),
          { status: 200 },
        ),
      )
      // actual API call
      fetchMock.mockResolvedValueOnce(new Response('{}', { status: 200 }))

      await authenticatedFetch('/api/v1/test')

      expect(fetchMock).toHaveBeenCalledTimes(2)
      const [url, init] = fetchMock.mock.calls[1] as [string, RequestInit]
      expect(url).toBe('https://test.useindelible.com/api/v1/test')
      const headers = init.headers as Headers
      expect(headers.get('Authorization')).toBe('Bearer jwt_new')
    })

    it('retries after successful 401 token refresh', async () => {
      setAccessTokenMemory('jwt_expired', FUTURE_EXPIRY)
      mockStorage['ind_refresh_token'] = 'indr_valid'

      fetchMock.mockResolvedValueOnce(new Response('Unauthorized', { status: 401 }))
      fetchMock.mockResolvedValueOnce(
        new Response(
          JSON.stringify({
            access_token: 'jwt_new',
            refresh_token: 'indr_new',
            expires_at: FUTURE_EXPIRY,
            token_type: 'Bearer',
          }),
          { status: 200 },
        ),
      )
      fetchMock.mockResolvedValueOnce(new Response('{"ok":true}', { status: 200 }))

      const response = await authenticatedFetch('/api/v1/test')
      expect(response.status).toBe(200)
    })

    it('clears refresh token and throws on 401 when refresh also fails', async () => {
      setAccessTokenMemory('jwt_expired', FUTURE_EXPIRY)
      mockStorage['ind_refresh_token'] = 'indr_also_expired'

      fetchMock.mockResolvedValueOnce(new Response('Unauthorized', { status: 401 }))
      fetchMock.mockResolvedValueOnce(new Response('Unauthorized', { status: 401 }))

      await expect(authenticatedFetch('/api/v1/test')).rejects.toThrow(
        'Token is invalid or expired',
      )

      expect(mockStorage['ind_refresh_token']).toBeUndefined()
    })

    it('strips trailing slashes from server URL', async () => {
      setAccessTokenMemory('jwt_token', FUTURE_EXPIRY)
      mockStorage['ind_server_url'] = 'https://test.useindelible.com/'
      fetchMock.mockResolvedValueOnce(new Response('{}', { status: 200 }))

      await authenticatedFetch('/api/v1/test')

      const [url] = fetchMock.mock.calls[0] as [string, RequestInit]
      expect(url).toBe('https://test.useindelible.com/api/v1/test')
    })

    it('uses default server URL when none is configured', async () => {
      setAccessTokenMemory('jwt_token', FUTURE_EXPIRY)
      delete mockStorage['ind_server_url']
      fetchMock.mockResolvedValueOnce(new Response('{}', { status: 200 }))

      await authenticatedFetch('/api/v1/test')

      const [url] = fetchMock.mock.calls[0] as [string, RequestInit]
      expect(url).toBe('https://useindelible.com/api/v1/test')
    })

    it('passes through additional request options', async () => {
      setAccessTokenMemory('jwt_token', FUTURE_EXPIRY)
      fetchMock.mockResolvedValueOnce(new Response('{}', { status: 200 }))

      await authenticatedFetch('/api/v1/test', { method: 'POST', body: '{"key":"val"}' })

      const [, init] = fetchMock.mock.calls[0] as [string, RequestInit]
      expect(init.method).toBe('POST')
      expect(init.body).toBe('{"key":"val"}')
    })
  })

  describe('getExtensionStatus', () => {
    it('fetches and returns extension status', async () => {
      setAccessTokenMemory('jwt_token', FUTURE_EXPIRY)
      const statusData = {
        authenticated: true,
        user: {
          id: 'user_123',
          email: 'test@example.com',
          display_name: 'Test User',
        },
      }
      fetchMock.mockResolvedValueOnce(
        new Response(JSON.stringify(statusData), {
          status: 200,
          headers: { 'Content-Type': 'application/json' },
        }),
      )

      const result = await getExtensionStatus()
      expect(result).toEqual({
        connected: true,
        user: {
          id: 'user_123',
          email: 'test@example.com',
          displayName: 'Test User',
        },
      })
    })

    it('throws on non-ok response', async () => {
      setAccessTokenMemory('jwt_token', FUTURE_EXPIRY)
      fetchMock.mockResolvedValueOnce(new Response('Server Error', { status: 500 }))

      await expect(getExtensionStatus()).rejects.toThrow('Server Error')
    })
  })

  describe('getExtensionSavedEntry', () => {
    it('fetches saved entry context by id', async () => {
      setAccessTokenMemory('jwt_token', FUTURE_EXPIRY)
      const contextData = {
        library_entry_id: 'lib_123',
        document_id: 'doc_123',
        title: 'Test Article',
        url: 'https://example.com/article',
        triage_state: 'inbox',
        is_favorite: false,
        saved_at: '2026-01-01T00:00:00Z',
        reader_url: 'https://app.indelible.test/reader/doc_123',
        tags: [{ id: 'tag_1', name: 'rust' }],
        note: {
          id: 'note_1',
          body: 'my note',
          created_at: '2026-01-01T00:00:00Z',
          updated_at: '2026-01-01T00:00:00Z',
        },
      }
      fetchMock.mockResolvedValueOnce(
        new Response(JSON.stringify(contextData), {
          status: 200,
          headers: { 'Content-Type': 'application/json' },
        }),
      )

      const result = await getExtensionSavedEntry('lib_123')
      expect(result).not.toBeNull()
      expect(result!.library_entry_id).toBe('lib_123')
      expect(result!.document_id).toBe('doc_123')
      expect(result!.reader_url).toBe('https://app.indelible.test/reader/doc_123')
      expect(result!.tags).toHaveLength(1)
      expect(result!.tags[0]!.name).toBe('rust')
      expect(result!.note?.body).toBe('my note')
    })

    it('returns null on 404', async () => {
      setAccessTokenMemory('jwt_token', FUTURE_EXPIRY)
      fetchMock.mockResolvedValueOnce(new Response('Not Found', { status: 404 }))

      const result = await getExtensionSavedEntry('lib_missing')
      expect(result).toBeNull()
    })

    it('throws on server error', async () => {
      setAccessTokenMemory('jwt_token', FUTURE_EXPIRY)
      fetchMock.mockResolvedValueOnce(new Response('Server Error', { status: 500 }))

      await expect(getExtensionSavedEntry('lib_123')).rejects.toThrow(
        'Get extension saved entry failed: 500',
      )
    })
  })

  describe('checkExtensionUrl', () => {
    it('reads the flattened backend response shape', async () => {
      setAccessTokenMemory('jwt_token', FUTURE_EXPIRY)
      fetchMock.mockResolvedValueOnce(
        new Response(JSON.stringify({ exists: false }), {
          status: 200,
          headers: { 'Content-Type': 'application/json' },
        }),
      )

      const result = await checkExtensionUrl('https://example.com')
      expect(result.exists).toBe(false)

      const [request] = fetchMock.mock.calls[0] as [Request]
      expect(request.url).toContain('/api/v1/extension/check-url?url=')
    })
  })

  describe('patchExtensionSavedEntry', () => {
    it('sends PATCH with triage_state and is_favorite', async () => {
      setAccessTokenMemory('jwt_token', FUTURE_EXPIRY)
      const responseData = {
        library_entry_id: 'lib_123',
        document_id: 'doc_123',
        title: 'Test',
        triage_state: 'later',
        is_favorite: true,
        saved_at: '2026-01-01T00:00:00Z',
        reader_url: 'https://app.indelible.test/reader/doc_123',
        tags: [],
      }
      fetchMock.mockResolvedValueOnce(
        new Response(JSON.stringify(responseData), {
          status: 200,
          headers: { 'Content-Type': 'application/json' },
        }),
      )

      const result = await patchExtensionSavedEntry('lib_123', {
        triage_state: 'later',
        is_favorite: true,
      })
      expect(result.triage_state).toBe('later')
      expect(result.is_favorite).toBe(true)

      const [request] = fetchMock.mock.calls[0] as [Request]
      expect(request.method).toBe('PATCH')
      const sentBody = JSON.parse(await request.text()) as Record<string, unknown>
      expect(sentBody.triage_state).toBe('later')
      expect(sentBody.is_favorite).toBe(true)
    })

    it('throws on non-ok response', async () => {
      setAccessTokenMemory('jwt_token', FUTURE_EXPIRY)
      fetchMock.mockResolvedValueOnce(new Response('Unprocessable', { status: 422 }))

      await expect(
        patchExtensionSavedEntry('lib_123', { triage_state: 'invalid' }),
      ).rejects.toThrow('Unprocessable')
    })
  })

  describe('upsertEntryNote', () => {
    it('sends PUT with note body and returns note', async () => {
      setAccessTokenMemory('jwt_token', FUTURE_EXPIRY)
      const noteData = {
        id: 'note_1',
        body: 'updated note',
        created_at: '2026-01-01T00:00:00Z',
        updated_at: '2026-01-01T00:00:00Z',
      }
      fetchMock.mockResolvedValueOnce(
        new Response(JSON.stringify(noteData), {
          status: 200,
          headers: { 'Content-Type': 'application/json' },
        }),
      )

      const result = await upsertEntryNote('lib_123', 'updated note')
      expect(result).not.toBeNull()
      expect(result!.body).toBe('updated note')

      const [request] = fetchMock.mock.calls[0] as [Request]
      expect(request.url).toContain('/api/v1/extension/entries/lib_123/note')
      expect(request.method).toBe('PUT')
    })

    it('returns null when body is empty (delete)', async () => {
      setAccessTokenMemory('jwt_token', FUTURE_EXPIRY)
      fetchMock.mockResolvedValueOnce(
        new Response('null', { status: 200, headers: { 'Content-Type': 'application/json' } }),
      )

      const result = await upsertEntryNote('lib_123', '')
      expect(result).toBeNull()
    })

    it('throws on server error', async () => {
      setAccessTokenMemory('jwt_token', FUTURE_EXPIRY)
      fetchMock.mockResolvedValueOnce(new Response('Server Error', { status: 500 }))

      await expect(upsertEntryNote('lib_123', 'note')).rejects.toThrow('Server Error')
    })
  })

  describe('syncEntryTags', () => {
    it('sends PUT with tag names and returns sorted tags', async () => {
      setAccessTokenMemory('jwt_token', FUTURE_EXPIRY)
      const tagsData = { tags: ['programming', 'rust', 'webdev'] }
      fetchMock.mockResolvedValueOnce(
        new Response(JSON.stringify(tagsData), {
          status: 200,
          headers: { 'Content-Type': 'application/json' },
        }),
      )

      const result = await syncEntryTags('lib_123', ['rust', 'webdev', 'programming'])
      expect(result.tags).toEqual(['programming', 'rust', 'webdev'])

      const [request] = fetchMock.mock.calls[0] as [Request]
      expect(request.url).toContain('/api/v1/extension/entries/lib_123/tags')
      expect(request.method).toBe('PUT')
      const sentBody = JSON.parse(await request.text()) as { tags: string[] }
      expect(sentBody.tags).toEqual(['rust', 'webdev', 'programming'])
    })

    it('handles empty tag list', async () => {
      setAccessTokenMemory('jwt_token', FUTURE_EXPIRY)
      const tagsData = { tags: [] as string[] }
      fetchMock.mockResolvedValueOnce(
        new Response(JSON.stringify(tagsData), {
          status: 200,
          headers: { 'Content-Type': 'application/json' },
        }),
      )

      const result = await syncEntryTags('lib_123', [])
      expect(result.tags).toEqual([])
    })

    it('throws on validation error', async () => {
      setAccessTokenMemory('jwt_token', FUTURE_EXPIRY)
      fetchMock.mockResolvedValueOnce(new Response('Too many tags', { status: 422 }))

      await expect(
        syncEntryTags(
          'lib_123',
          Array.from({ length: 21 }, (_, i) => `tag${i}`),
        ),
      ).rejects.toThrow('Too many tags')
    })
  })

  describe('getPipelineStatus', () => {
    it('maps saved entry context to pipeline status response', async () => {
      setAccessTokenMemory('jwt_token', FUTURE_EXPIRY)
      const contextData = {
        library_entry_id: 'lib_123',
        document_id: 'doc_123',
        title: 'Test Article',
        url: 'https://example.com',
        triage_state: 'inbox',
        is_favorite: false,
        saved_at: '2026-01-01T00:00:00Z',
        reader_url: 'https://app.indelible.test/reader/doc_123',
        tags: [],
      }
      fetchMock.mockResolvedValueOnce(
        new Response(JSON.stringify(contextData), {
          status: 200,
          headers: { 'Content-Type': 'application/json' },
        }),
      )

      const result = await getPipelineStatus('lib_123')
      expect(result).not.toBeNull()
      expect(result!.status).toBe('completed')
      expect(result!.entry?.id).toBe('lib_123')
      expect(result!.entry?.title).toBe('Test Article')
    })

    it('returns null on 404', async () => {
      setAccessTokenMemory('jwt_token', FUTURE_EXPIRY)
      fetchMock.mockResolvedValueOnce(new Response('Not Found', { status: 404 }))

      const result = await getPipelineStatus('lib_missing')
      expect(result).toBeNull()
    })
  })

  describe('reprocessDocument', () => {
    it('queues document reprocessing through the configured server', async () => {
      setAccessTokenMemory('jwt_token', FUTURE_EXPIRY)
      fetchMock.mockResolvedValueOnce(
        new Response(JSON.stringify({ queued: true, job_type: 'document.reprocess' }), {
          status: 200,
          headers: { 'Content-Type': 'application/json' },
        }),
      )

      const result = await reprocessDocument('doc_123')

      expect(result.queued).toBe(true)
      expect(result.job_type).toBe('document.reprocess')
      const [request] = fetchMock.mock.calls[0] as [Request]
      expect(request.url).toBe('https://test.useindelible.com/api/v1/documents/doc_123/reprocess')
      expect(request.method).toBe('POST')
    })

    it('preserves the generated cooldown response', async () => {
      setAccessTokenMemory('jwt_token', FUTURE_EXPIRY)
      fetchMock.mockResolvedValueOnce(
        new Response(
          JSON.stringify({
            queued: false,
            job_type: 'document.reprocess',
            retry_after_seconds: 120,
          }),
          { status: 200, headers: { 'Content-Type': 'application/json' } },
        ),
      )

      const result = await reprocessDocument('doc_123')

      expect(result.queued).toBe(false)
      expect(result.retry_after_seconds).toBe(120)
    })
  })
})

describe('refreshAccessToken', () => {
  beforeEach(() => {
    clearMockStorage()
    clearAccessTokenMemory()
    fetchMock.mockReset()
  })

  it('shares one refresh request between concurrent callers', async () => {
    mockStorage['ind_refresh_token'] = 'refresh_1'
    mockStorage['ind_server_url'] = 'https://test.useindelible.com'
    fetchMock.mockImplementation(
      async () =>
        new Response(
          JSON.stringify({
            access_token: 'jwt_2',
            refresh_token: 'refresh_2',
            expires_at: FUTURE_EXPIRY,
            token_type: 'Bearer',
          }),
          { status: 200, headers: { 'Content-Type': 'application/json' } },
        ),
    )

    const results = await Promise.all([refreshAccessToken(), refreshAccessToken()])

    expect(results).toEqual([true, true])
    expect(fetchMock).toHaveBeenCalledTimes(1)
    expect(mockStorage['ind_refresh_token']).toBe('refresh_2')
  })
})
