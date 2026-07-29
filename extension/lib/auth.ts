import {
  getRefreshToken,
  setRefreshToken,
  clearRefreshToken,
  getServerUrl,
  setServerUrl,
  getConnectedAt,
  normalizeServerUrl,
} from './storage'
import {
  getExtensionStatus,
  exchangeCode,
  refreshAccessToken,
  setAccessTokenMemory,
  clearAccessTokenMemory,
  revokeRefreshToken,
} from './api'

export interface AuthState {
  status: 'disconnected' | 'connected' | 'error'
  serverUrl: string
  connectedAt?: string
  user?: {
    id: string
    email: string
    displayName: string
  }
  message?: string
}

interface PendingPkce {
  verifier: string
  state: string
  redirectUri: string
  tabId?: number
  returnTabId?: number
  expiresAt: number
}

const PENDING_PKCE_KEY = 'ind_pending_pkce'
const PENDING_PKCE_TTL_MS = 10 * 60 * 1000

export async function getPendingPkce(): Promise<PendingPkce | null> {
  const result = await browser.storage.local.get(PENDING_PKCE_KEY)
  const value = result[PENDING_PKCE_KEY]
  if (value === undefined) return null
  if (!isPendingPkce(value)) {
    await clearPendingPkce()
    return null
  }
  if (value.expiresAt <= Date.now()) {
    await clearPendingPkce()
    return null
  }
  return value
}

export async function clearPendingPkce(): Promise<void> {
  await browser.storage.local.remove(PENDING_PKCE_KEY)
}

async function setPendingPkce(pendingPkce: PendingPkce): Promise<void> {
  await browser.storage.local.set({ [PENDING_PKCE_KEY]: pendingPkce })
}

function isPendingPkce(value: unknown): value is PendingPkce {
  if (typeof value !== 'object' || value === null) return false
  const record = value as Record<string, unknown>
  return (
    typeof record.verifier === 'string' &&
    typeof record.state === 'string' &&
    typeof record.redirectUri === 'string' &&
    typeof record.expiresAt === 'number' &&
    (record.tabId === undefined || typeof record.tabId === 'number') &&
    (record.returnTabId === undefined || typeof record.returnTabId === 'number')
  )
}

export async function getAuthState(): Promise<AuthState> {
  const [token, serverUrl, connectedAt] = await Promise.all([
    getRefreshToken(),
    getServerUrl(),
    getConnectedAt(),
  ])

  if (!token) {
    return { status: 'disconnected', serverUrl }
  }

  try {
    const extensionStatus = await getExtensionStatus()
    if (!extensionStatus.connected) {
      await clearRefreshToken()
      clearAccessTokenMemory()
      return { status: 'disconnected', serverUrl }
    }

    return {
      status: 'connected',
      serverUrl,
      connectedAt: connectedAt ?? undefined,
      user: extensionStatus.user,
    }
  } catch (err) {
    const remainingToken = await getRefreshToken()
    if (!remainingToken) {
      return { status: 'disconnected', serverUrl }
    }

    return {
      status: 'error',
      serverUrl,
      message: err instanceof Error ? err.message : 'Unable to validate extension status',
    }
  }
}

function generateCodeVerifier(): string {
  const array = new Uint8Array(32)
  crypto.getRandomValues(array)
  return base64UrlEncode(array)
}

async function generateCodeChallenge(verifier: string): Promise<string> {
  const encoder = new TextEncoder()
  const data = encoder.encode(verifier)
  const digest = await crypto.subtle.digest('SHA-256', data)
  return base64UrlEncode(new Uint8Array(digest))
}

function base64UrlEncode(bytes: Uint8Array): string {
  let binary = ''
  for (const byte of bytes) {
    binary += String.fromCharCode(byte)
  }
  return btoa(binary).replace(/\+/g, '-').replace(/\//g, '_').replace(/=+$/, '')
}

function generateState(): string {
  const array = new Uint8Array(16)
  crypto.getRandomValues(array)
  return base64UrlEncode(array)
}

export async function connect(serverUrl: string, returnTabId?: number): Promise<void> {
  const normalizedUrl = normalizeServerUrl(serverUrl)
  await setServerUrl(normalizedUrl)
  await clearPendingPkce()

  const verifier = generateCodeVerifier()
  const challenge = await generateCodeChallenge(verifier)
  const state = generateState()

  // redirect_uri points back to the server's callback path which the
  // extension watches via tabs.onUpdated
  const redirectUri = `${normalizedUrl}/extension/auth/callback`

  const params = new URLSearchParams({
    code_challenge: challenge,
    state,
    redirect_uri: redirectUri,
  })

  const authUrl = `${normalizedUrl}/extension/auth?${params.toString()}`
  const tab = await browser.tabs.create({ url: authUrl })

  await setPendingPkce({
    verifier,
    state,
    redirectUri,
    tabId: tab.id,
    returnTabId,
    expiresAt: Date.now() + PENDING_PKCE_TTL_MS,
  })
}

export async function handleAuthCallback(
  code: string,
  state: string,
): Promise<{ returnTabId?: number }> {
  const pendingPkce = await getPendingPkce()
  if (!pendingPkce) {
    throw new Error('No pending authorization')
  }

  if (state !== pendingPkce.state) {
    await clearPendingPkce()
    throw new Error('State mismatch')
  }

  const verifier = pendingPkce.verifier
  const redirectUri = pendingPkce.redirectUri
  const tabId = pendingPkce.tabId
  const returnTabId = pendingPkce.returnTabId
  await clearPendingPkce()

  const tokens = await exchangeCode(code, verifier, redirectUri)
  setAccessTokenMemory(tokens.access_token, tokens.expires_at)
  await setRefreshToken(tokens.refresh_token)

  if (tabId) {
    try {
      await browser.tabs.remove(tabId)
    } catch {
      // tab may already be closed
    }
  }

  return { returnTabId }
}

export async function disconnect(): Promise<void> {
  await revokeRefreshToken()
  clearAccessTokenMemory()
  await clearRefreshToken()
  await clearPendingPkce()
}

export async function validateToken(): Promise<boolean> {
  const token = await getRefreshToken()
  if (!token) return false

  try {
    return await refreshAccessToken()
  } catch {
    return false
  }
}
