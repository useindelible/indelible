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
  returnTabId?: number
  expiresAt: number
}

const PENDING_PKCE_KEY = 'ind_pending_pkce'
const PENDING_PKCE_TTL_MS = 10 * 60 * 1000
let authFlowInProgress = false

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

export async function connect(
  serverUrl: string,
  returnTabId?: number,
): Promise<{ returnTabId?: number }> {
  const normalizedUrl = normalizeServerUrl(serverUrl)
  if (authFlowInProgress) {
    throw new Error('Authorization is already in progress')
  }
  authFlowInProgress = true
  let ownsPendingAuthorization = false

  try {
    if (await getPendingPkce()) {
      throw new Error('Authorization is already in progress')
    }

    await setServerUrl(normalizedUrl)

    const verifier = generateCodeVerifier()
    const challenge = await generateCodeChallenge(verifier)
    const state = generateState()
    const redirectUri = browser.identity.getRedirectURL('indelible')

    const params = new URLSearchParams({
      code_challenge: challenge,
      state,
      redirect_uri: redirectUri,
    })

    await setPendingPkce({
      verifier,
      state,
      redirectUri,
      returnTabId,
      expiresAt: Date.now() + PENDING_PKCE_TTL_MS,
    })
    ownsPendingAuthorization = true

    const authUrl = `${normalizedUrl}/api/v1/auth/extension/start?${params.toString()}`
    const responseUrl = await launchAuthFlowWithTimeout(authUrl)
    if (!responseUrl) {
      throw new Error('Authorization was cancelled. Please try again.')
    }
    return await handleAuthResponse(responseUrl)
  } catch (error) {
    if (ownsPendingAuthorization) await clearPendingPkce()
    if (error instanceof Error && isActionableAuthError(error.message)) {
      throw error
    }
    throw new Error('Authorization could not be completed. Please try again.')
  } finally {
    authFlowInProgress = false
  }
}

async function launchAuthFlowWithTimeout(authUrl: string): Promise<string | undefined> {
  let timeoutId: ReturnType<typeof setTimeout> | undefined
  try {
    return await Promise.race([
      browser.identity.launchWebAuthFlow({ url: authUrl, interactive: true }),
      new Promise<never>((_, reject) => {
        timeoutId = setTimeout(
          () => reject(new Error('Authorization timed out. Please try again.')),
          PENDING_PKCE_TTL_MS,
        )
      }),
    ])
  } finally {
    if (timeoutId !== undefined) clearTimeout(timeoutId)
  }
}

function isActionableAuthError(message: string): boolean {
  return (
    message === 'Authorization is already in progress' ||
    message.startsWith('Authorization was cancelled') ||
    message.startsWith('Authorization timed out') ||
    message === 'Invalid authorization response' ||
    message === 'State mismatch' ||
    message === 'No pending authorization'
  )
}

async function handleAuthResponse(responseUrl: string): Promise<{ returnTabId?: number }> {
  const pendingPkce = await getPendingPkce()
  if (!pendingPkce) {
    throw new Error('No pending authorization')
  }

  let response: URL
  let expected: URL
  try {
    response = new URL(responseUrl)
    expected = new URL(pendingPkce.redirectUri)
  } catch {
    throw new Error('Invalid authorization response')
  }

  if (response.origin !== expected.origin || response.pathname !== expected.pathname) {
    throw new Error('Invalid authorization response')
  }

  const code = response.searchParams.get('code')
  const state = response.searchParams.get('state')
  if (!code || !state) {
    throw new Error('Invalid authorization response')
  }

  return handleAuthCallback(code, state)
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
  const returnTabId = pendingPkce.returnTabId
  await clearPendingPkce()

  const tokens = await exchangeCode(code, verifier, redirectUri)
  await setRefreshToken(tokens.refresh_token)
  setAccessTokenMemory(tokens.access_token, tokens.expires_at)

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
