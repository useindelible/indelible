import { spawnSync } from 'node:child_process'
import { fileURLToPath } from 'node:url'

import { describe, expect, it } from 'vitest'

describe('extension catalog checker', () => {
  it('rejects duplicate message keys', () => {
    const cwd = fileURLToPath(new URL('./fixtures/i18n-duplicate', import.meta.url))
    const script = fileURLToPath(new URL('../scripts/i18n-check.js', import.meta.url))
    const result = spawnSync(process.execPath, [script], { cwd, encoding: 'utf8' })

    expect(result.status).toBe(1)
    expect(result.stderr).toContain('duplicate message key: menu_save_page')
  })
})
