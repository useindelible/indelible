import { beforeEach, describe, expect, it, vi } from 'vitest'

const getMessage = vi.fn()

vi.stubGlobal('browser', {
  i18n: {
    getMessage,
    getUILanguage: () => 'fr',
  },
})

import en from '../public/_locales/en/messages.json'
import { relativeTime, resolveFallback, savingStepLabel, t, tPlural } from '../lib/i18n'

describe('extension i18n', () => {
  beforeEach(() => {
    getMessage.mockReset()
    getMessage.mockReturnValue('')
  })

  it('prefers the browser catalog and falls back to English', () => {
    getMessage.mockReturnValueOnce('Enregistrer dans Indelible')

    expect(t('menu_save_page')).toBe('Enregistrer dans Indelible')
    expect(t('menu_save_highlight')).toBe('Save highlight to Indelible')
  })

  it('resolves named and positional fallback substitutions', () => {
    expect(resolveFallback(en.notify_saving, ['Fetching'])).toBe('Indelible is saving: Fetching')
    expect(resolveFallback({ message: 'Price: $$5' }, [])).toBe('Price: $5')
  })

  it('selects singular and plural messages', () => {
    expect(tPlural('toolbar_highlights', 1)).toBe('1 highlight')
    expect(tPlural('toolbar_highlights', 3)).toBe('3 highlights')
  })

  it('translates finite saving states before interpolation', () => {
    getMessage.mockImplementation((key: string) =>
      key === 'notify_step_extracting' ? 'Extraction du contenu' : '',
    )

    expect(savingStepLabel('extracting')).toBe('Extraction du contenu')
  })

  it('formats relative time using the browser UI language', () => {
    const now = Date.parse('2026-08-23T12:00:00Z')

    expect(relativeTime('2026-08-23T11:57:00Z', now)).toBe('il y a 3 minutes')
  })
})
