import type { ToolbarState } from './full-archive-toolbar'

const DEFAULT_SERVER_URL = 'https://useindelible.com'

interface MessageResponse {
  success?: boolean
  error?: string
  data?: unknown
}

type Rerender = (state: ToolbarState) => void

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === 'object' && value !== null
}

function send(message: Record<string, unknown>): Promise<MessageResponse | undefined> {
  return browser.runtime.sendMessage(message) as Promise<MessageResponse | undefined>
}

function readableError(value: string | undefined): string | undefined {
  const message = value?.replace(/^Error:\s*/, '').trim()
  return message ? message : undefined
}

/**
 * Wires every toolbar control that talks to the configured server: connecting, retrying an
 * unreachable server against a possibly-edited address, and signing out. Sign-out has to keep
 * working while the server is down, which is why it never waits on a successful revocation.
 */
export function bindConnectionEvents(
  root: ShadowRoot,
  state: ToolbarState,
  rerender: Rerender,
): void {
  function readServerUrl(): string {
    const input = root.querySelector<HTMLInputElement>('[data-role="server-url"]')
    return input?.value.trim() || state.serverUrl || DEFAULT_SERVER_URL
  }

  function busy(action: string, label?: string): void {
    const button = root.querySelector<HTMLButtonElement>(`[data-action="${action}"]`)
    if (!button) return
    button.disabled = true
    if (label) button.textContent = label
  }

  async function connect(): Promise<void> {
    const serverUrl = readServerUrl()
    rerender({ view: 'connecting', serverUrl })

    try {
      const response = await send({ action: 'toolbar:connect', serverUrl })
      if (response?.success) return

      rerender({
        view: 'auth-error',
        serverUrl,
        message: readableError(response?.error) ?? 'Authorization could not be started.',
      })
    } catch (error) {
      rerender({
        view: 'auth-error',
        serverUrl,
        message:
          error instanceof Error
            ? error.message
            : 'Authorization could not be started. Please try again.',
      })
    }
  }

  async function retry(): Promise<void> {
    const serverUrl = readServerUrl()
    busy('retry', 'Retrying')

    try {
      if (serverUrl !== state.serverUrl) {
        const saved = await send({ action: 'toolbar:set-server-url', serverUrl })
        if (!saved?.success) {
          rerender({
            view: 'unreachable',
            serverUrl: state.serverUrl,
            message: readableError(saved?.error) ?? 'That server address could not be saved.',
          })
          return
        }
      }

      const response = await send({ action: 'auth:status' })
      const auth = isRecord(response?.data) ? response.data : undefined
      const status = typeof auth?.status === 'string' ? auth.status : 'error'

      if (status === 'connected') {
        void send({ action: 'toolbar:save' })
        return
      }

      if (status === 'disconnected') {
        rerender({ view: 'disconnected', serverUrl })
        return
      }

      rerender({
        view: 'unreachable',
        serverUrl,
        message: typeof auth?.message === 'string' ? auth.message : undefined,
      })
    } catch (error) {
      rerender({
        view: 'unreachable',
        serverUrl,
        message: error instanceof Error ? error.message : undefined,
      })
    }
  }

  async function signOut(): Promise<void> {
    const serverUrl = readServerUrl()
    busy('sign-out')

    try {
      const response = await send({ action: 'auth:logout' })
      if (!response?.success) {
        rerender({
          view: 'unreachable',
          serverUrl,
          message: readableError(response?.error) ?? 'Sign out could not be completed.',
        })
        return
      }
      rerender({ view: 'disconnected', serverUrl })
    } catch (error) {
      rerender({
        view: 'unreachable',
        serverUrl,
        message: error instanceof Error ? error.message : 'Sign out could not be completed.',
      })
    }
  }

  root.querySelector('[data-action="connect"]')?.addEventListener('click', () => {
    void connect()
  })

  root.querySelector('[data-action="retry"]')?.addEventListener('click', () => {
    void retry()
  })

  root.querySelector('[data-action="sign-out"]')?.addEventListener('click', () => {
    void signOut()
  })

  root.querySelector('[data-action="refresh"]')?.addEventListener('click', () => {
    void send({ action: 'toolbar:save' })
  })

  root
    .querySelector<HTMLInputElement>('[data-role="server-url"]')
    ?.addEventListener('keydown', (e) => {
      if (e.key !== 'Enter') return
      if (state.view === 'unreachable') {
        void retry()
        return
      }
      void connect()
    })
}
