import { readdir, readFile } from 'node:fs/promises'
import { join } from 'node:path'
import { getNodeValue, parseTree, printParseErrorCode } from 'jsonc-parser'

const localesDir = join(process.cwd(), 'public', '_locales')
const localeNames = (await readdir(localesDir, { withFileTypes: true }))
  .filter((entry) => entry.isDirectory())
  .map((entry) => entry.name)
  .sort()

const catalogs = new Map()
const errors = []

for (const locale of localeNames) {
  const path = join(localesDir, locale, 'messages.json')
  try {
    const text = await readFile(path, 'utf8')
    const parseErrors = []
    const root = parseTree(text, parseErrors, { allowTrailingComma: false })
    for (const error of parseErrors) {
      errors.push(`${locale}: could not parse messages.json: ${printParseErrorCode(error.error)}`)
    }
    if (!root || parseErrors.length > 0) continue
    findDuplicateKeys(locale, root)
    catalogs.set(locale, getNodeValue(root))
  } catch (error) {
    errors.push(`${locale}: could not parse messages.json: ${error.message}`)
  }
}

const source = catalogs.get('en')
if (!isRecord(source)) {
  errors.push('en: source catalog must be a JSON object')
}

for (const [locale, catalog] of catalogs) {
  if (!isRecord(catalog)) {
    errors.push(`${locale}: catalog must be a JSON object`)
    continue
  }

  const keys = Object.keys(catalog)
  if (keys.join('\n') !== [...keys].sort().join('\n')) {
    errors.push(`${locale}: message keys must be sorted`)
  }

  for (const [key, entry] of Object.entries(catalog)) {
    validateEntry(locale, key, entry)
  }

  if (!isRecord(source) || locale === 'en') continue

  for (const key of keys) {
    if (!(key in source)) errors.push(`${locale}.${key}: key does not exist in en`)
  }

  if (locale === 'fr') {
    for (const key of Object.keys(source)) {
      if (!(key in catalog)) errors.push(`fr.${key}: missing reference translation`)
    }
  }

  for (const key of keys.filter((candidate) => candidate in source)) {
    comparePlaceholders(locale, key, source[key], catalog[key])
  }
}

if (errors.length > 0) {
  console.error(`Extension i18n check failed (${errors.length}):`)
  for (const error of errors) console.error(`- ${error}`)
  process.exitCode = 1
} else {
  console.log(
    `Extension i18n check passed (${localeNames.length} locales, ${Object.keys(source).length} source messages)`,
  )
}

function validateEntry(locale, key, entry) {
  if (!isRecord(entry)) {
    errors.push(`${locale}.${key}: entry must be an object`)
    return
  }

  const allowedFields = new Set(['message', 'description', 'placeholders'])
  for (const field of Object.keys(entry)) {
    if (!allowedFields.has(field)) errors.push(`${locale}.${key}: unknown field ${field}`)
  }

  if (typeof entry.message !== 'string' || entry.message.trim() === '') {
    errors.push(`${locale}.${key}: message must be a non-empty string`)
  }
  if (entry.description !== undefined && typeof entry.description !== 'string') {
    errors.push(`${locale}.${key}: description must be a string`)
  }
  if (entry.placeholders === undefined) return
  if (!isRecord(entry.placeholders)) {
    errors.push(`${locale}.${key}: placeholders must be an object`)
    return
  }

  for (const [name, placeholder] of Object.entries(entry.placeholders)) {
    if (!isRecord(placeholder) || typeof placeholder.content !== 'string' || !placeholder.content) {
      errors.push(`${locale}.${key}.placeholders.${name}: content must be a non-empty string`)
    }
  }
}

function comparePlaceholders(locale, key, sourceEntry, translatedEntry) {
  if (!isRecord(sourceEntry) || !isRecord(translatedEntry)) return

  const sourceNames = placeholderNames(sourceEntry)
  const translatedNames = placeholderNames(translatedEntry)
  if (sourceNames.join('\n') !== translatedNames.join('\n')) {
    errors.push(`${locale}.${key}: placeholder names differ from en`)
  }

  if (tokenSignature(sourceEntry.message) !== tokenSignature(translatedEntry.message)) {
    errors.push(`${locale}.${key}: message placeholder tokens differ from en`)
  }

  for (const name of sourceNames) {
    const sourcePlaceholder = sourceEntry.placeholders?.[name]
    const translatedPlaceholder = translatedEntry.placeholders?.[name]
    if (
      isRecord(sourcePlaceholder) &&
      isRecord(translatedPlaceholder) &&
      tokenSignature(sourcePlaceholder.content) !== tokenSignature(translatedPlaceholder.content)
    ) {
      errors.push(`${locale}.${key}.placeholders.${name}: substitution tokens differ from en`)
    }
  }
}

function placeholderNames(entry) {
  return isRecord(entry.placeholders) ? Object.keys(entry.placeholders).sort() : []
}

function tokenSignature(value) {
  if (typeof value !== 'string') return ''
  const tokens = [
    ...(value.match(/\$[A-Za-z][A-Za-z0-9_@]*\$/g) ?? []),
    ...(value.match(/\$\d+/g) ?? []),
  ]
  return tokens.sort().join('|')
}

function isRecord(value) {
  return typeof value === 'object' && value !== null && !Array.isArray(value)
}

function findDuplicateKeys(locale, node, path = []) {
  if (node.type === 'object') {
    const seen = new Set()
    for (const property of node.children ?? []) {
      const keyNode = property.children?.[0]
      const valueNode = property.children?.[1]
      const key = keyNode?.value
      if (typeof key !== 'string' || !valueNode) continue

      if (seen.has(key)) {
        const location = path.length === 0 ? 'message key' : `object key at ${path.join('.')}`
        errors.push(`${locale}: duplicate ${location}: ${key}`)
      }
      seen.add(key)
      findDuplicateKeys(locale, valueNode, [...path, key])
    }
    return
  }

  for (const child of node.children ?? []) findDuplicateKeys(locale, child, path)
}
