import { clearAccessTokenMemory, getAccessToken, refreshAccessToken } from '../api'
import { clearRefreshToken, getServerUrl } from '../storage'
import { fetchOrThrowUnreachable } from '../server-reachability'
import { client } from './generated/client.gen'

/**
 * Drives the generated OpenAPI client through the extension's token lifecycle: inject the in-memory
 * access token, refresh once on a 401 and retry, and clear credentials when the refresh fails. This
 * mirrors the proven `authenticatedFetch` flow so the generated save calls keep the same behavior.
 */
const authFetch: typeof fetch = async (input, init) => {
  const request = input instanceof Request ? input : new Request(input, init)
  let token = getAccessToken()
  if (!token) {
    await refreshAccessToken()
    token = getAccessToken()
  }
  if (!token) {
    throw new Error('No valid authentication')
  }
  request.headers.set('Accept', 'application/json')
  // Clone before the first fetch consumes the body, so a 401 retry can re-send the same POST body.
  const retry = request.clone()
  request.headers.set('Authorization', `Bearer ${token}`)

  const response = await fetchOrThrowUnreachable(request)
  if (response.status !== 401) {
    return response
  }

  if (await refreshAccessToken()) {
    const retryToken = getAccessToken()
    if (retryToken) {
      retry.headers.set('Authorization', `Bearer ${retryToken}`)
      return fetchOrThrowUnreachable(retry)
    }
  }
  clearAccessTokenMemory()
  await clearRefreshToken()
  return response
}

client.setConfig({ fetch: authFetch })

export { client }

/** Per-call options binding the generated client to the user's configured server origin. */
export async function serverRequestOptions(): Promise<{ baseUrl: string }> {
  const serverUrl = await getServerUrl()
  return { baseUrl: serverUrl.replace(/\/+$/, '') }
}
