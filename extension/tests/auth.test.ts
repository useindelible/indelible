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
        const keyList = Array.isArray(keys) ? keys : [keys]
        for (const key of keyList) {
          // eslint-disable-next-line @typescript-eslint/no-dynamic-delete
          delete mockStorage[key]
        }
      }),
    },
  },
  tabs: {
    create: vi.fn(async () => ({ id: 1 })),
    remove: vi.fn(async () => undefined),
  },
})

const fetchMock = vi.fn()
vi.stubGlobal('fetch', fetchMock)

import {
  getAuthState,
  connect,
  disconnect,
  handleAuthCallback,
  validateToken,
  clearPendingPkce,
  getPendingPkce,
} from '../lib/auth'
import { clearAccessTokenMemory } from '../lib/api'

function clearMockStorage(): void {
  for (const key of Object.keys(mockStorage)) {
    // eslint-disable-next-line @typescript-eslint/no-dynamic-delete
    delete mockStorage[key]
  }
}

const FUTURE_EXPIRY = Math.floor(Date.now() / 1000) + 3600

function makeRefreshResponse(accessToken = 'jwt_access', refreshToken = 'indr_new') {
  return new Response(
    JSON.stringify({
      access_token: accessToken,
      refresh_token: refreshToken,
      expires_at: FUTURE_EXPIRY,
      token_type: 'Bearer',
    }),
    { status: 200 },
  )
}

function makeStatusResponse(userId = 'user_123', email = 'test@example.com') {
  return new Response(
    JSON.stringify({
      authenticated: true,
      user: { id: userId, email, display_name: 'Test User' },
    }),
    { status: 200, headers: { 'Content-Type': 'application/json' } },
  )
}

describe('auth', () => {
  beforeEach(async () => {
    clearMockStorage()
    vi.clearAllMocks()
    fetchMock.mockReset()
    clearAccessTokenMemory()
    await clearPendingPkce()
  })

  describe('getAuthState', () => {
    it('returns disconnected state when no refresh token is stored', async () => {
      const state = await getAuthState()
      expect(state.status).toBe('disconnected')
      expect(state.serverUrl).toBe('https://useindelible.com')
      expect(state.connectedAt).toBeUndefined()
    })

    it('returns connected state when refresh token is valid', async () => {
      mockStorage['ind_refresh_token'] = 'indr_valid_token'
      mockStorage['ind_connected_at'] = '2026-03-01T12:00:00.000Z'
      mockStorage['ind_server_url'] = 'https://custom.example.com'

      fetchMock.mockResolvedValueOnce(makeRefreshResponse())
      fetchMock.mockResolvedValueOnce(makeStatusResponse())

      const state = await getAuthState()
      expect(state.status).toBe('connected')
      expect(state.serverUrl).toBe('https://custom.example.com')
      expect(state.connectedAt).toBe('2026-03-01T12:00:00.000Z')
      expect(state.user).toEqual({
        id: 'user_123',
        email: 'test@example.com',
        displayName: 'Test User',
      })
    })

    it('returns disconnected state and clears token when refresh fails', async () => {
      mockStorage['ind_refresh_token'] = 'indr_expired'

      fetchMock.mockResolvedValueOnce(new Response('Unauthorized', { status: 401 }))

      const state = await getAuthState()
      expect(state.status).toBe('disconnected')
      expect(mockStorage['ind_refresh_token']).toBeUndefined()
    })

    it('returns disconnected state with default URL when server URL is not set', async () => {
      mockStorage['ind_refresh_token'] = 'indr_valid_token'

      fetchMock.mockResolvedValueOnce(makeRefreshResponse())
      fetchMock.mockResolvedValueOnce(makeStatusResponse())

      const state = await getAuthState()
      expect(state.status).toBe('connected')
      expect(state.serverUrl).toBe('https://useindelible.com')
    })
  })

  describe('connect', () => {
    it('stores the server URL and opens auth tab with PKCE params', async () => {
      await connect('https://my-indelible.com')

      expect(mockStorage['ind_server_url']).toBe('https://my-indelible.com')
      expect(mockStorage['ind_pending_pkce']).toMatchObject({
        redirectUri: 'https://my-indelible.com/extension/auth/callback',
        tabId: 1,
      })
      expect(browser.tabs.create).toHaveBeenCalledOnce()

      const [{ url }] = (browser.tabs.create as ReturnType<typeof vi.fn>).mock.calls[0] as [
        { url: string },
      ]
      const parsed = new URL(url)
      expect(parsed.origin).toBe('https://my-indelible.com')
      expect(parsed.pathname).toBe('/extension/auth')
      expect(parsed.searchParams.get('code_challenge')).toBeTruthy()
      expect(parsed.searchParams.get('state')).toBeTruthy()
      expect(parsed.searchParams.get('redirect_uri')).toBe(
        'https://my-indelible.com/extension/auth/callback',
      )
    })

    it('strips trailing slashes from the server URL', async () => {
      await connect('https://my-indelible.com/')

      expect(mockStorage['ind_server_url']).toBe('https://my-indelible.com')
      const [{ url }] = (browser.tabs.create as ReturnType<typeof vi.fn>).mock.calls[0] as [
        { url: string },
      ]
      expect(url).toContain('https://my-indelible.com/extension/auth')
    })

    it('opens tab with correct auth URL for default server', async () => {
      await connect('https://useindelible.com')

      const [{ url }] = (browser.tabs.create as ReturnType<typeof vi.fn>).mock.calls[0] as [
        { url: string },
      ]
      expect(url).toContain('https://useindelible.com/extension/auth')
    })

    it('rejects invalid server URLs', async () => {
      await expect(connect('ftp://bad.example.com')).rejects.toThrow(
        'Server URL must use http or https',
      )
    })

    it('rejects non-local http server URLs', async () => {
      await expect(connect('http://bad.example.com')).rejects.toThrow(
        'Server URL must use HTTPS unless it points to localhost',
      )
    })
  })

  describe('disconnect', () => {
    it('clears the stored tokens', async () => {
      mockStorage['ind_refresh_token'] = 'indr_some_token'
      mockStorage['ind_connected_at'] = '2026-03-01T00:00:00.000Z'

      fetchMock.mockResolvedValueOnce(new Response('', { status: 200 }))

      await disconnect()

      expect(mockStorage['ind_refresh_token']).toBeUndefined()
      expect(mockStorage['ind_connected_at']).toBeUndefined()
    })

    it('preserves the server URL after disconnect', async () => {
      mockStorage['ind_refresh_token'] = 'indr_some_token'
      mockStorage['ind_server_url'] = 'https://custom.example.com'

      fetchMock.mockResolvedValueOnce(new Response('', { status: 200 }))

      await disconnect()

      expect(mockStorage['ind_server_url']).toBe('https://custom.example.com')
    })
  })

  describe('handleAuthCallback', () => {
    it('exchanges authorization code and stores tokens', async () => {
      await connect('https://useindelible.com')

      const [{ url }] = (browser.tabs.create as ReturnType<typeof vi.fn>).mock.calls[0] as [
        { url: string },
      ]
      const state = new URL(url).searchParams.get('state')!

      fetchMock.mockResolvedValueOnce(makeRefreshResponse('jwt_new', 'indr_stored'))

      await handleAuthCallback('auth_code_123', state)

      expect(mockStorage['ind_refresh_token']).toBe('indr_stored')
    })

    it('throws when there is no pending authorization', async () => {
      await expect(handleAuthCallback('code', 'state')).rejects.toThrow('No pending authorization')
    })

    it('throws on state parameter mismatch', async () => {
      await connect('https://useindelible.com')

      await expect(handleAuthCallback('code', 'wrong_state')).rejects.toThrow('State mismatch')
      expect(await getPendingPkce()).toBeNull()
    })

    it('exchanges authorization code from stored pending authorization', async () => {
      mockStorage['ind_pending_pkce'] = {
        verifier: 'verifier',
        state: 'stored_state',
        redirectUri: 'https://useindelible.com/extension/auth/callback',
        tabId: 1,
        expiresAt: Date.now() + 60_000,
      }
      fetchMock.mockResolvedValueOnce(makeRefreshResponse('jwt_stored', 'indr_stored_reload'))

      await handleAuthCallback('auth_code_123', 'stored_state')

      expect(mockStorage['ind_refresh_token']).toBe('indr_stored_reload')
      expect(mockStorage['ind_pending_pkce']).toBeUndefined()
    })

    it('clears expired pending authorization before callback exchange', async () => {
      mockStorage['ind_pending_pkce'] = {
        verifier: 'verifier',
        state: 'state',
        redirectUri: 'https://useindelible.com/extension/auth/callback',
        tabId: 1,
        expiresAt: Date.now() - 1,
      }

      await expect(handleAuthCallback('code', 'state')).rejects.toThrow('No pending authorization')
      expect(mockStorage['ind_pending_pkce']).toBeUndefined()
      expect(fetchMock).not.toHaveBeenCalled()
    })
  })

  describe('validateToken', () => {
    it('returns false when no refresh token is stored', async () => {
      const valid = await validateToken()
      expect(valid).toBe(false)
      expect(fetchMock).not.toHaveBeenCalled()
    })

    it('returns true when refresh succeeds', async () => {
      mockStorage['ind_refresh_token'] = 'indr_valid_token'
      mockStorage['ind_server_url'] = 'https://test.useindelible.com'

      fetchMock.mockResolvedValueOnce(makeRefreshResponse())

      const valid = await validateToken()
      expect(valid).toBe(true)
    })

    it('returns false when refresh fails with 401', async () => {
      mockStorage['ind_refresh_token'] = 'indr_expired_token'
      mockStorage['ind_server_url'] = 'https://test.useindelible.com'

      fetchMock.mockResolvedValueOnce(new Response('Unauthorized', { status: 401 }))

      const valid = await validateToken()
      expect(valid).toBe(false)
    })

    it('returns false on network error', async () => {
      mockStorage['ind_refresh_token'] = 'indr_unreachable'
      mockStorage['ind_server_url'] = 'https://test.useindelible.com'

      fetchMock.mockRejectedValueOnce(new Error('Network error'))

      const valid = await validateToken()
      expect(valid).toBe(false)
    })

    it('calls the refresh endpoint', async () => {
      mockStorage['ind_refresh_token'] = 'indr_token_to_validate'
      mockStorage['ind_server_url'] = 'https://test.useindelible.com'

      fetchMock.mockResolvedValueOnce(makeRefreshResponse())

      await validateToken()

      expect(fetchMock).toHaveBeenCalledOnce()
      const [url] = fetchMock.mock.calls[0] as [string, RequestInit]
      expect(url).toBe('https://test.useindelible.com/api/v1/auth/extension/refresh')
    })
  })
})
