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

import {
  getRefreshToken,
  setRefreshToken,
  clearRefreshToken,
  getServerUrl,
  setServerUrl,
  getConnectedAt,
  getStoredAuthData,
  normalizeServerUrl,
} from '../lib/storage'

function clearMockStorage(): void {
  for (const key of Object.keys(mockStorage)) {
    // eslint-disable-next-line @typescript-eslint/no-dynamic-delete
    delete mockStorage[key]
  }
}

describe('storage', () => {
  beforeEach(() => {
    clearMockStorage()
    vi.clearAllMocks()
  })

  describe('getRefreshToken', () => {
    it('returns null when no token is stored', async () => {
      const token = await getRefreshToken()
      expect(token).toBeNull()
    })

    it('returns the stored token', async () => {
      mockStorage['ind_refresh_token'] = 'indr_test_token_123'
      const token = await getRefreshToken()
      expect(token).toBe('indr_test_token_123')
    })

    it('returns null when stored value is not a string', async () => {
      mockStorage['ind_refresh_token'] = 42
      const token = await getRefreshToken()
      expect(token).toBeNull()
    })
  })

  describe('setRefreshToken', () => {
    it('stores the token and connected timestamp', async () => {
      await setRefreshToken('indr_new_token')
      expect(mockStorage['ind_refresh_token']).toBe('indr_new_token')
      expect(typeof mockStorage['ind_connected_at']).toBe('string')
    })

    it('sets a valid ISO timestamp for connectedAt', async () => {
      await setRefreshToken('indr_new_token')
      const timestamp = mockStorage['ind_connected_at'] as string
      const date = new Date(timestamp)
      expect(date.getTime()).not.toBeNaN()
    })
  })

  describe('clearRefreshToken', () => {
    it('removes the token and connectedAt', async () => {
      mockStorage['ind_refresh_token'] = 'indr_some_token'
      mockStorage['ind_connected_at'] = '2026-01-01T00:00:00.000Z'

      await clearRefreshToken()

      expect(mockStorage['ind_refresh_token']).toBeUndefined()
      expect(mockStorage['ind_connected_at']).toBeUndefined()
    })
  })

  describe('getServerUrl', () => {
    it('returns the default URL when none is stored', async () => {
      const url = await getServerUrl()
      expect(url).toBe('https://useindelible.com')
    })

    it('returns the stored server URL', async () => {
      mockStorage['ind_server_url'] = 'https://self-hosted.example.com'
      const url = await getServerUrl()
      expect(url).toBe('https://self-hosted.example.com')
    })

    it('returns default URL when stored value is empty string', async () => {
      mockStorage['ind_server_url'] = ''
      const url = await getServerUrl()
      expect(url).toBe('https://useindelible.com')
    })
  })

  describe('setServerUrl', () => {
    it('stores the server URL', async () => {
      await setServerUrl('https://my-instance.com')
      expect(mockStorage['ind_server_url']).toBe('https://my-instance.com')
    })

    it('normalizes the server URL to its origin', async () => {
      await setServerUrl('https://my-instance.com/path?foo=bar#hash')
      expect(mockStorage['ind_server_url']).toBe('https://my-instance.com')
    })

    it('rejects invalid server URLs', async () => {
      await expect(setServerUrl('not-a-url')).rejects.toThrow(
        'Server URL must be a valid http or https URL',
      )
    })
  })

  describe('normalizeServerUrl', () => {
    it('returns the normalized origin', () => {
      expect(normalizeServerUrl(' https://my-instance.com/foo ')).toBe('https://my-instance.com')
    })

    it('allows localhost http URLs for local development', () => {
      expect(normalizeServerUrl('http://localhost:5173/path')).toBe('http://localhost:5173')
      expect(normalizeServerUrl('http://127.0.0.1:8080/path')).toBe('http://127.0.0.1:8080')
      expect(normalizeServerUrl('http://[::1]:3000/path')).toBe('http://[::1]:3000')
    })

    it('rejects non-local http URLs', () => {
      expect(() => normalizeServerUrl('http://my-instance.com')).toThrow(
        'Server URL must use HTTPS unless it points to localhost',
      )
    })

    it('rejects non-http urls', () => {
      expect(() => normalizeServerUrl('ftp://my-instance.com')).toThrow(
        'Server URL must use http or https',
      )
    })
  })

  describe('getConnectedAt', () => {
    it('returns null when no timestamp is stored', async () => {
      const connectedAt = await getConnectedAt()
      expect(connectedAt).toBeNull()
    })

    it('returns the stored timestamp', async () => {
      mockStorage['ind_connected_at'] = '2026-03-01T12:00:00.000Z'
      const connectedAt = await getConnectedAt()
      expect(connectedAt).toBe('2026-03-01T12:00:00.000Z')
    })
  })

  describe('getStoredAuthData', () => {
    it('returns defaults when nothing is stored', async () => {
      const data = await getStoredAuthData()
      expect(data).toEqual({
        refreshToken: null,
        serverUrl: 'https://useindelible.com',
        connectedAt: null,
      })
    })

    it('returns all stored values', async () => {
      mockStorage['ind_refresh_token'] = 'indr_stored_token'
      mockStorage['ind_server_url'] = 'https://custom.example.com'
      mockStorage['ind_connected_at'] = '2026-03-15T10:30:00.000Z'

      const data = await getStoredAuthData()
      expect(data).toEqual({
        refreshToken: 'indr_stored_token',
        serverUrl: 'https://custom.example.com',
        connectedAt: '2026-03-15T10:30:00.000Z',
      })
    })

    it('returns defaults for missing fields while preserving existing ones', async () => {
      mockStorage['ind_refresh_token'] = 'indr_partial_token'

      const data = await getStoredAuthData()
      expect(data).toEqual({
        refreshToken: 'indr_partial_token',
        serverUrl: 'https://useindelible.com',
        connectedAt: null,
      })
    })
  })
})
