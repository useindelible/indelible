const STORAGE_KEYS = {
  refreshToken: 'ind_refresh_token',
  serverUrl: 'ind_server_url',
  connectedAt: 'ind_connected_at',
} as const

const DEFAULT_SERVER_URL = 'https://useindelible.com'

export function normalizeServerUrl(url: string): string {
  const trimmed = url.trim()
  if (!trimmed) {
    throw new Error('Server URL is required')
  }

  let parsed: URL
  try {
    parsed = new URL(trimmed)
  } catch {
    throw new Error('Server URL must be a valid http or https URL')
  }

  if (parsed.protocol !== 'http:' && parsed.protocol !== 'https:') {
    throw new Error('Server URL must use http or https')
  }

  if (parsed.protocol === 'http:' && !isLocalhost(parsed.hostname)) {
    throw new Error('Server URL must use HTTPS unless it points to localhost')
  }

  return parsed.origin
}

function isLocalhost(hostname: string): boolean {
  return hostname === 'localhost' || hostname === '127.0.0.1' || hostname === '[::1]'
}

export interface StoredAuthData {
  refreshToken: string | null
  serverUrl: string
  connectedAt: string | null
}

export async function getRefreshToken(): Promise<string | null> {
  const result = await browser.storage.local.get(STORAGE_KEYS.refreshToken)
  const value = result[STORAGE_KEYS.refreshToken]
  return typeof value === 'string' ? value : null
}

export async function setRefreshToken(token: string): Promise<void> {
  await browser.storage.local.set({
    [STORAGE_KEYS.refreshToken]: token,
    [STORAGE_KEYS.connectedAt]: new Date().toISOString(),
  })
}

export async function clearRefreshToken(): Promise<void> {
  await browser.storage.local.remove([STORAGE_KEYS.refreshToken, STORAGE_KEYS.connectedAt])
}

export async function getServerUrl(): Promise<string> {
  const result = await browser.storage.local.get(STORAGE_KEYS.serverUrl)
  const value = result[STORAGE_KEYS.serverUrl]
  return typeof value === 'string' && value.length > 0 ? value : DEFAULT_SERVER_URL
}

export async function setServerUrl(url: string): Promise<void> {
  await browser.storage.local.set({ [STORAGE_KEYS.serverUrl]: normalizeServerUrl(url) })
}

export async function getConnectedAt(): Promise<string | null> {
  const result = await browser.storage.local.get(STORAGE_KEYS.connectedAt)
  const value = result[STORAGE_KEYS.connectedAt]
  return typeof value === 'string' ? value : null
}

export async function getStoredAuthData(): Promise<StoredAuthData> {
  const result = await browser.storage.local.get([
    STORAGE_KEYS.refreshToken,
    STORAGE_KEYS.serverUrl,
    STORAGE_KEYS.connectedAt,
  ])

  const tokenVal = result[STORAGE_KEYS.refreshToken]
  const urlVal = result[STORAGE_KEYS.serverUrl]
  const connectedVal = result[STORAGE_KEYS.connectedAt]

  return {
    refreshToken: typeof tokenVal === 'string' ? tokenVal : null,
    serverUrl: typeof urlVal === 'string' && urlVal.length > 0 ? urlVal : DEFAULT_SERVER_URL,
    connectedAt: typeof connectedVal === 'string' ? connectedVal : null,
  }
}
