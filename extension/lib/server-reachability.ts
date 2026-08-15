export const SERVER_UNREACHABLE_MESSAGE = 'Indelible server is unreachable'

const SERVER_UNAVAILABLE_PREFIX = 'Indelible server is unavailable'

export function serverUnavailableMessage(status: number): string {
  return `${SERVER_UNAVAILABLE_PREFIX} (HTTP ${status})`
}

export function isServerUnreachableError(message: string): boolean {
  return message === SERVER_UNREACHABLE_MESSAGE || message.startsWith(SERVER_UNAVAILABLE_PREFIX)
}

/**
 * Transport failures surface as browser-specific `TypeError`s ("Failed to fetch", "NetworkError…").
 * Normalising them here keeps reachability detection off brittle per-browser strings.
 */
export async function fetchOrThrowUnreachable(
  input: Request | string,
  init?: RequestInit,
): Promise<Response> {
  try {
    return await fetch(input as RequestInfo, init)
  } catch (error) {
    if (error instanceof Error && isServerUnreachableError(error.message)) throw error
    throw new Error(SERVER_UNREACHABLE_MESSAGE)
  }
}

/**
 * A save that failed because the server was down leaves the session valid, so it belongs in the
 * unreachable view — the only one offering the server address and sign-out — rather than the
 * generic error view, which is a dead end.
 */
export function resolveReachabilityView(state: Record<string, unknown>): Record<string, unknown> {
  if (state.view !== 'error') return state
  const message = typeof state.message === 'string' ? state.message : ''
  if (!isServerUnreachableError(message)) return state
  return { ...state, view: 'unreachable' }
}
