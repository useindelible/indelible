import { describe, expect, it } from 'vitest'

import {
  SERVER_UNREACHABLE_MESSAGE,
  isServerUnreachableError,
  resolveReachabilityView,
  serverUnavailableMessage,
} from '../lib/server-reachability'

describe('server reachability', () => {
  it('recognises the canonical transport and availability failures', () => {
    expect(isServerUnreachableError(SERVER_UNREACHABLE_MESSAGE)).toBe(true)
    expect(isServerUnreachableError(serverUnavailableMessage(502))).toBe(true)
  })

  it('leaves unrelated failures alone', () => {
    expect(isServerUnreachableError('Upload failed')).toBe(false)
    expect(isServerUnreachableError('Token is invalid or expired')).toBe(false)
  })

  it('promotes an unreachable save failure to the recoverable toolbar view', () => {
    expect(
      resolveReachabilityView({
        view: 'error',
        serverUrl: 'http://localhost:38481',
        message: SERVER_UNREACHABLE_MESSAGE,
      }),
    ).toEqual({
      view: 'unreachable',
      serverUrl: 'http://localhost:38481',
      message: SERVER_UNREACHABLE_MESSAGE,
    })
  })

  it('keeps unrelated errors and non-error views unchanged', () => {
    const uploadFailure = { view: 'error', message: 'Upload failed' }
    expect(resolveReachabilityView(uploadFailure)).toBe(uploadFailure)

    const saving = { view: 'saving', message: SERVER_UNREACHABLE_MESSAGE }
    expect(resolveReachabilityView(saving)).toBe(saving)
  })
})
