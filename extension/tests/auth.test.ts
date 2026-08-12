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
  identity: {
    getRedirectURL: vi.fn(() => 'https://abcdefghijklmnop.chromiumapp.org/indelible'),
    launchWebAuthFlow: vi.fn(),
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
    ;(browser.identity.getRedirectURL as ReturnType<typeof vi.fn>).mockReturnValue(
      'https://abcdefghijklmnop.chromiumapp.org/indelible',
    )
    ;(browser.identity.launchWebAuthFlow as ReturnType<typeof vi.fn>).mockImplementation(
      async ({ url }: { url: string }) => {
        const state = new URL(url).searchParams.get('state')
        return `https://abcdefghijklmnop.chromiumapp.org/indelible?code=auth_code_123&state=${state}`
      },
    )
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

    it('returns an error state and retains credentials when the server is unavailable', async () => {
      mockStorage['ind_refresh_token'] = 'indr_valid_token'
      mockStorage['ind_server_url'] = 'http://localhost:38481'
      fetchMock.mockResolvedValueOnce(new Response('Unavailable', { status: 503 }))

      const state = await getAuthState()

      expect(state.status).toBe('error')
      expect(state.serverUrl).toBe('http://localhost:38481')
      expect(mockStorage['ind_refresh_token']).toBe('indr_valid_token')
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
    it('stores pending PKCE before launching browser-managed authentication', async () => {
      ;(browser.identity.launchWebAuthFlow as ReturnType<typeof vi.fn>).mockImplementationOnce(
        async ({ url }: { url: string }) => {
          expect(mockStorage['ind_pending_pkce']).toMatchObject({
            redirectUri: 'https://abcdefghijklmnop.chromiumapp.org/indelible',
          })
          const state = new URL(url).searchParams.get('state')
          return `https://abcdefghijklmnop.chromiumapp.org/indelible?code=auth_code_123&state=${state}`
        },
      )
      fetchMock.mockResolvedValueOnce(makeRefreshResponse('jwt_new', 'indr_stored'))

      await connect('https://my-indelible.com')

      expect(mockStorage['ind_server_url']).toBe('https://my-indelible.com')
      expect(mockStorage['ind_pending_pkce']).toBeUndefined()
      expect(mockStorage['ind_refresh_token']).toBe('indr_stored')
      expect(browser.identity.launchWebAuthFlow).toHaveBeenCalledOnce()

      const [{ url, interactive }] = (
        browser.identity.launchWebAuthFlow as ReturnType<typeof vi.fn>
      ).mock.calls[0] as [{ url: string; interactive: boolean }]
      const parsed = new URL(url)
      expect(parsed.origin).toBe('https://my-indelible.com')
      expect(parsed.pathname).toBe('/api/v1/auth/extension/start')
      expect(parsed.searchParams.get('code_challenge')).toBeTruthy()
      expect(parsed.searchParams.get('state')).toBeTruthy()
      expect(parsed.searchParams.get('redirect_uri')).toBe(
        'https://abcdefghijklmnop.chromiumapp.org/indelible',
      )
      expect(interactive).toBe(true)
    })

    it('strips trailing slashes from the server URL', async () => {
      fetchMock.mockResolvedValueOnce(makeRefreshResponse())
      await connect('https://my-indelible.com/')

      expect(mockStorage['ind_server_url']).toBe('https://my-indelible.com')
      const [{ url }] = (browser.identity.launchWebAuthFlow as ReturnType<typeof vi.fn>).mock
        .calls[0] as [{ url: string }]
      expect(url).toContain('https://my-indelible.com/api/v1/auth/extension/start')
    })

    it('launches the correct auth URL for the default server', async () => {
      fetchMock.mockResolvedValueOnce(makeRefreshResponse())
      await connect('https://useindelible.com')

      const [{ url }] = (browser.identity.launchWebAuthFlow as ReturnType<typeof vi.fn>).mock
        .calls[0] as [{ url: string }]
      expect(url).toContain('https://useindelible.com/api/v1/auth/extension/start')
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

    it('rejects a concurrent unexpired authorization', async () => {
      mockStorage['ind_pending_pkce'] = {
        verifier: 'verifier',
        state: 'state',
        redirectUri: 'https://abcdefghijklmnop.chromiumapp.org/indelible',
        expiresAt: Date.now() + 60_000,
      }

      await expect(connect('https://useindelible.com')).rejects.toThrow(
        'Authorization is already in progress',
      )
      expect(browser.identity.launchWebAuthFlow).not.toHaveBeenCalled()
    })

    it('rejects a simultaneous authorization before persisted state can race', async () => {
      let finishFlow: ((responseUrl: string) => void) | undefined
      ;(browser.identity.launchWebAuthFlow as ReturnType<typeof vi.fn>).mockImplementationOnce(
        () =>
          new Promise<string>((resolve) => {
            finishFlow = resolve
          }),
      )
      fetchMock.mockResolvedValueOnce(makeRefreshResponse())

      const first = connect('https://useindelible.com')
      await vi.waitFor(() => expect(browser.identity.launchWebAuthFlow).toHaveBeenCalledOnce())
      await expect(connect('https://useindelible.com')).rejects.toThrow(
        'Authorization is already in progress',
      )

      const [{ url }] = (browser.identity.launchWebAuthFlow as ReturnType<typeof vi.fn>).mock
        .calls[0] as [{ url: string }]
      const state = new URL(url).searchParams.get('state')
      finishFlow?.(
        `https://abcdefghijklmnop.chromiumapp.org/indelible?code=auth_code_123&state=${state}`,
      )
      await first
    })

    it('clears an expired authorization and starts a new one', async () => {
      mockStorage['ind_pending_pkce'] = {
        verifier: 'verifier',
        state: 'state',
        redirectUri: 'https://abcdefghijklmnop.chromiumapp.org/indelible',
        expiresAt: Date.now() - 1,
      }
      fetchMock.mockResolvedValueOnce(makeRefreshResponse())

      await connect('https://useindelible.com')

      expect(browser.identity.launchWebAuthFlow).toHaveBeenCalledOnce()
      expect(mockStorage['ind_pending_pkce']).toBeUndefined()
    })

    it('clears pending authorization when the user cancels', async () => {
      ;(browser.identity.launchWebAuthFlow as ReturnType<typeof vi.fn>).mockResolvedValueOnce(
        undefined,
      )

      await expect(connect('https://useindelible.com')).rejects.toThrow(
        'Authorization was cancelled. Please try again.',
      )
      expect(await getPendingPkce()).toBeNull()
    })

    it('clears pending authorization when the browser rejects the flow', async () => {
      ;(browser.identity.launchWebAuthFlow as ReturnType<typeof vi.fn>).mockRejectedValueOnce(
        new Error('The user did not approve access'),
      )

      await expect(connect('https://useindelible.com')).rejects.toThrow(
        'Authorization could not be completed. Please try again.',
      )
      expect(await getPendingPkce()).toBeNull()
    })

    it('rejects malformed and mismatched callback URLs without exchanging a code', async () => {
      for (const response of [
        'not a URL',
        'https://attacker.example/indelible?code=code&state=state',
        'https://abcdefghijklmnop.chromiumapp.org/indelible?state=state',
      ]) {
        ;(browser.identity.launchWebAuthFlow as ReturnType<typeof vi.fn>).mockResolvedValueOnce(
          response,
        )
        await expect(connect('https://useindelible.com')).rejects.toThrow(
          'Invalid authorization response',
        )
        expect(await getPendingPkce()).toBeNull()
      }
      expect(fetchMock).not.toHaveBeenCalled()
    })

    it('clears pending authorization on state mismatch and token exchange failure', async () => {
      ;(browser.identity.launchWebAuthFlow as ReturnType<typeof vi.fn>).mockResolvedValueOnce(
        'https://abcdefghijklmnop.chromiumapp.org/indelible?code=code&state=wrong',
      )
      await expect(connect('https://useindelible.com')).rejects.toThrow('State mismatch')
      expect(await getPendingPkce()).toBeNull()

      fetchMock.mockResolvedValueOnce(new Response('invalid code', { status: 400 }))
      await expect(connect('https://useindelible.com')).rejects.toThrow(
        'Authorization could not be completed. Please try again.',
      )
      expect(await getPendingPkce()).toBeNull()
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
      mockStorage['ind_pending_pkce'] = {
        verifier: 'verifier',
        state: 'stored_state',
        redirectUri: 'https://abcdefghijklmnop.chromiumapp.org/indelible',
        expiresAt: Date.now() + 60_000,
      }

      fetchMock.mockResolvedValueOnce(makeRefreshResponse('jwt_new', 'indr_stored'))

      await handleAuthCallback('auth_code_123', 'stored_state')

      expect(mockStorage['ind_refresh_token']).toBe('indr_stored')
    })

    it('throws when there is no pending authorization', async () => {
      await expect(handleAuthCallback('code', 'state')).rejects.toThrow('No pending authorization')
    })

    it('throws on state parameter mismatch', async () => {
      mockStorage['ind_pending_pkce'] = {
        verifier: 'verifier',
        state: 'stored_state',
        redirectUri: 'https://abcdefghijklmnop.chromiumapp.org/indelible',
        expiresAt: Date.now() + 60_000,
      }

      await expect(handleAuthCallback('code', 'wrong_state')).rejects.toThrow('State mismatch')
      expect(await getPendingPkce()).toBeNull()
    })

    it('exchanges authorization code from stored pending authorization', async () => {
      mockStorage['ind_pending_pkce'] = {
        verifier: 'verifier',
        state: 'stored_state',
        redirectUri: 'https://abcdefghijklmnop.chromiumapp.org/indelible',
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
        redirectUri: 'https://abcdefghijklmnop.chromiumapp.org/indelible',
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
