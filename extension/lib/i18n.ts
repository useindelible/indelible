import en from '../public/_locales/en/messages.json'

export type MessageKey = keyof typeof en
export type SavingStep = 'thumbnail' | 'extracting' | 'singlefile' | 'uploading'

interface CatalogEntry {
  message: string
  placeholders?: Record<string, { content: string }>
}

export function resolveFallback(entry: CatalogEntry, substitutions: string[]): string {
  const named = entry.message.replace(/\$([A-Za-z0-9_@]+)\$/g, (match, name: string) => {
    return entry.placeholders?.[name.toLowerCase()]?.content ?? match
  })

  return named
    .replace(/\$\$/g, '\u0000')
    .replace(/\$(\d)/g, (_, index: string) => substitutions[Number(index) - 1] ?? '')
    .replace(/\u0000/g, '$')
}

export function t(key: MessageKey, substitutions?: string | string[]): string {
  const values =
    substitutions === undefined
      ? []
      : Array.isArray(substitutions)
        ? substitutions
        : [substitutions]
  const browserMessage = browser.i18n?.getMessage(key, values) ?? ''

  return browserMessage || resolveFallback(en[key], values)
}

export function tPlural(base: string, count: number): string {
  const category = new Intl.PluralRules(uiLanguage()).select(count)
  const candidate = `${base}_${category}`
  const key = (candidate in en ? candidate : `${base}_other`) as MessageKey

  return t(key, String(count))
}

export function savingStepLabel(step: SavingStep): string {
  const keys: Record<SavingStep, MessageKey> = {
    thumbnail: 'notify_step_thumbnail',
    extracting: 'notify_step_extracting',
    singlefile: 'notify_step_singlefile',
    uploading: 'notify_step_uploading',
  }
  return t(keys[step])
}

export function relativeTime(iso?: string, now = Date.now()): string {
  if (!iso) return ''

  const timestamp = new Date(iso).getTime()
  if (!Number.isFinite(timestamp)) return ''

  const seconds = Math.round((timestamp - now) / 1000)
  const absoluteSeconds = Math.abs(seconds)
  const units: Array<[Intl.RelativeTimeFormatUnit, number]> = [
    ['year', 31_536_000],
    ['month', 2_592_000],
    ['week', 604_800],
    ['day', 86_400],
    ['hour', 3_600],
    ['minute', 60],
  ]

  const [unit, divisor] = units.find(([, threshold]) => absoluteSeconds >= threshold) ?? [
    'second',
    1,
  ]
  const value = Math.round(seconds / divisor)

  return new Intl.RelativeTimeFormat(uiLanguage(), { numeric: 'auto' }).format(value, unit)
}

function uiLanguage(): string {
  return (
    browser.i18n?.getUILanguage?.() ||
    (typeof navigator === 'undefined' ? 'en' : navigator.language) ||
    'en'
  )
}
