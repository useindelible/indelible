import { getRefreshToken, getServerUrl, clearRefreshToken, setRefreshToken } from './storage'
import { serverRequestOptions } from './api/client'
import {
  SERVER_UNREACHABLE_MESSAGE,
  fetchOrThrowUnreachable,
  isServerUnreachableError,
  serverUnavailableMessage,
} from './server-reachability'
import type { SourceLocatorPayload } from '../../shared/highlight-source'
import {
  extensionCheckUrl,
  extensionCreateHighlight,
  extensionFullArchive,
  extensionGetEntry,
  extensionGetEntryAsset,
  extensionListHighlights,
  extensionPatchEntry,
  extensionReaderSave,
  extensionReplaceTags,
  reprocessDocument as generatedReprocessDocument,
  extensionStatus,
  extensionUpsertNote,
} from './api/generated/sdk.gen'
import type {
  DocumentAssetResponse,
  DocumentReprocessResponse,
  ExtensionSaveResponse,
  ExtensionNoteResponse,
  ExtensionReplaceTagsResponse,
  ExtensionSavedEntryResponse,
  ExtensionUrlCheckResponse,
  HighlightListResponse,
  HighlightResponse,
  FullArchiveRequest,
  ReaderSaveRequest,
} from './api/generated/types.gen'

export type {
  ExtensionNoteResponse,
  ExtensionSavedEntryResponse,
  ExtensionUrlCheckResponse,
  HighlightListResponse,
  HighlightResponse,
  HighlightWithNoteResponse,
  TagResponse,
} from './api/generated/types.gen'

export interface ExtensionStatusResponse {
  authenticated: boolean
  user?: {
    id: string
    email: string
    display_name: string
  }
}

export interface ExtensionUserInfo {
  id: string
  email: string
  displayName: string
}

export interface ExtensionStatus {
  connected: boolean
  user?: ExtensionUserInfo
}

export interface TokenResponse {
  access_token: string
  refresh_token: string
  expires_at: number
  token_type: string
}

let accessToken: string | null = null
let accessTokenExpiresAt: number | null = null

export function getAccessToken(): string | null {
  if (accessToken && accessTokenExpiresAt && Date.now() / 1000 < accessTokenExpiresAt - 60) {
    return accessToken
  }
  return null
}

export function setAccessTokenMemory(token: string, expiresAt: number): void {
  accessToken = token
  accessTokenExpiresAt = expiresAt
}

export function clearAccessTokenMemory(): void {
  accessToken = null
  accessTokenExpiresAt = null
}

export async function authenticatedFetch(path: string, init?: RequestInit): Promise<Response> {
  let token = getAccessToken()

  if (!token) {
    const refreshed = await refreshAccessToken()
    if (!refreshed) {
      throw new Error('No valid authentication')
    }
    token = getAccessToken()
  }

  if (!token) {
    throw new Error('No access token available')
  }

  const serverUrl = await getServerUrl()
  const url = `${serverUrl.replace(/\/+$/, '')}${path}`
  const headers = new Headers(init?.headers)
  headers.set('Authorization', `Bearer ${token}`)
  headers.set('Accept', 'application/json')

  const response = await fetchOrThrowUnreachable(url, { ...init, headers })

  if (response.status === 401) {
    const refreshed = await refreshAccessToken()
    if (refreshed) {
      const retryToken = getAccessToken()
      if (retryToken) {
        headers.set('Authorization', `Bearer ${retryToken}`)
        return fetchOrThrowUnreachable(url, { ...init, headers })
      }
    }
    clearAccessTokenMemory()
    await clearRefreshToken()
    throw new Error('Token is invalid or expired')
  }

  return response
}

let refreshInFlight: Promise<boolean> | null = null

export function refreshAccessToken(): Promise<boolean> {
  refreshInFlight ??= refreshAccessTokenOnce().finally(() => {
    refreshInFlight = null
  })
  return refreshInFlight
}

async function refreshAccessTokenOnce(): Promise<boolean> {
  const refreshToken = await getRefreshToken()
  if (!refreshToken) return false

  const serverUrl = await getServerUrl()
  const url = `${serverUrl.replace(/\/+$/, '')}/api/v1/auth/extension/refresh`

  try {
    const response = await fetch(url, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ refresh_token: refreshToken }),
    })

    if (response.status === 400 || response.status === 401 || response.status === 403) {
      clearAccessTokenMemory()
      await clearRefreshToken()
      return false
    }

    if (!response.ok) {
      throw new Error(serverUnavailableMessage(response.status))
    }

    const data = (await response.json()) as TokenResponse
    setAccessTokenMemory(data.access_token, data.expires_at)
    if (data.refresh_token) {
      await setRefreshToken(data.refresh_token)
    }
    return true
  } catch (error) {
    if (error instanceof Error && isServerUnreachableError(error.message)) {
      throw error
    }
    throw new Error(SERVER_UNREACHABLE_MESSAGE)
  }
}

export async function exchangeCode(
  code: string,
  codeVerifier: string,
  redirectUri: string,
): Promise<TokenResponse> {
  const serverUrl = await getServerUrl()
  const url = `${serverUrl.replace(/\/+$/, '')}/api/v1/auth/extension/token`

  const response = await fetch(url, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ code, code_verifier: codeVerifier, redirect_uri: redirectUri }),
  })

  if (!response.ok) {
    throw new Error(`Token exchange failed: ${response.status}`)
  }

  return (await response.json()) as TokenResponse
}

export async function revokeRefreshToken(): Promise<void> {
  const refreshToken = await getRefreshToken()
  if (!refreshToken) return

  const serverUrl = await getServerUrl()
  const url = `${serverUrl.replace(/\/+$/, '')}/api/v1/auth/extension/revoke`

  try {
    await fetch(url, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ refresh_token: refreshToken }),
    })
  } catch {
    // best-effort revocation
  }
}

export interface PipelineStatusResponse {
  status:
    | 'queued'
    | 'fetching'
    | 'extracting'
    | 'archiving'
    | 'ai_processing'
    | 'indexing'
    | 'completed'
    | 'failed'
  entry?: {
    id: string
    title: string
    url: string
  }
  error?: string
}

export type ReplaceTagsResponse = ExtensionReplaceTagsResponse

export interface LocatorPayload {
  type: 'html'
  start_offset: number
  end_offset: number
}

export type { SourceLocatorPayload } from '../../shared/highlight-source'

export async function getPipelineStatus(
  libraryEntryId: string,
): Promise<PipelineStatusResponse | null> {
  const entry = await getExtensionSavedEntry(libraryEntryId)
  if (!entry) return null

  return {
    status: 'completed',
    entry: {
      id: entry.library_entry_id,
      title: entry.title,
      url: entry.url ?? '',
    },
  }
}

export async function getExtensionSavedEntry(
  libraryEntryId: string,
): Promise<ExtensionSavedEntryResponse | null> {
  const { data, error, response } = await extensionGetEntry({
    ...(await serverRequestOptions()),
    path: { library_entry_id: libraryEntryId },
  })

  if (response?.status === 404) return null
  if (error !== undefined || data === undefined) {
    throw new Error(`Get extension saved entry failed: ${response?.status ?? 'network error'}`)
  }

  return data
}

export async function checkExtensionUrl(url: string): Promise<ExtensionUrlCheckResponse> {
  const { data } = await extensionCheckUrl({
    ...(await serverRequestOptions()),
    query: { url },
    throwOnError: true,
  })

  return data
}

export async function listExtensionHighlights(
  libraryEntryId: string,
): Promise<HighlightListResponse> {
  const { data } = await extensionListHighlights({
    ...(await serverRequestOptions()),
    path: { library_entry_id: libraryEntryId },
    throwOnError: true,
  })

  return data
}

export async function createExtensionHighlight(
  libraryEntryId: string,
  body: {
    color: string
    text_content: string
    locator?: LocatorPayload
    source_locator?: SourceLocatorPayload
  },
): Promise<HighlightResponse> {
  const { data } = await extensionCreateHighlight({
    ...(await serverRequestOptions()),
    path: { library_entry_id: libraryEntryId },
    body: {
      color: body.color,
      text_content: body.text_content,
      locator: body.locator,
      source_locator: body.source_locator,
    },
    throwOnError: true,
  })

  return data
}

export async function patchExtensionSavedEntry(
  libraryEntryId: string,
  patch: { triage_state?: string; is_favorite?: boolean },
): Promise<ExtensionSavedEntryResponse> {
  const { data } = await extensionPatchEntry({
    ...(await serverRequestOptions()),
    path: { library_entry_id: libraryEntryId },
    body: { triage_state: patch.triage_state, is_favorite: patch.is_favorite },
    throwOnError: true,
  })

  return data
}

export async function upsertEntryNote(
  libraryEntryId: string,
  body: string,
): Promise<ExtensionNoteResponse | null> {
  const { data } = await extensionUpsertNote({
    ...(await serverRequestOptions()),
    path: { library_entry_id: libraryEntryId },
    body: { body },
    throwOnError: true,
  })

  return data
}

export async function syncEntryTags(
  libraryEntryId: string,
  tags: string[],
): Promise<ReplaceTagsResponse> {
  const { data } = await extensionReplaceTags({
    ...(await serverRequestOptions()),
    path: { library_entry_id: libraryEntryId },
    body: { tags },
    throwOnError: true,
  })

  return data
}

export async function getEntryAsset(
  libraryEntryId: string,
  assetKind: string,
): Promise<DocumentAssetResponse | null> {
  const { data, error } = await extensionGetEntryAsset({
    ...(await serverRequestOptions()),
    path: { library_entry_id: libraryEntryId, asset_kind: assetKind },
  })

  if (error !== undefined || data === undefined) return null
  return data
}

export async function reprocessDocument(documentId: string): Promise<DocumentReprocessResponse> {
  const { data } = await generatedReprocessDocument({
    ...(await serverRequestOptions()),
    path: { document_id: documentId },
    throwOnError: true,
  })

  return data
}

export async function saveExtensionFullArchive(
  body: FullArchiveRequest,
): Promise<ExtensionSaveResponse> {
  const { data } = await extensionFullArchive({
    ...(await serverRequestOptions()),
    body,
    throwOnError: true,
  })
  return data
}

export async function saveExtensionReaderArchive(
  body: ReaderSaveRequest,
): Promise<ExtensionSaveResponse> {
  const { data } = await extensionReaderSave({
    ...(await serverRequestOptions()),
    body,
    throwOnError: true,
  })
  return data
}

export async function getExtensionStatus(): Promise<ExtensionStatus> {
  const { data } = await extensionStatus({
    ...(await serverRequestOptions()),
    throwOnError: true,
  })

  return {
    connected: data.authenticated,
    user: data.user
      ? {
          id: data.user.id,
          email: data.user.email,
          displayName: data.user.display_name,
        }
      : undefined,
  }
}
