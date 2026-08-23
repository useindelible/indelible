# Web localization

English is the source and fallback locale. French is the reference translation, so
CI requires it to contain every English key. Other locales may be partial and fall
back to English.

## Add or change copy

1. Add a sorted, descriptive key to `locales/en.json`. Prefix feature copy with its
   area, such as `library_`, `reader_`, or `prefs_`; reserve `common_` for genuinely
   shared copy.
2. Add the same key and placeholders to `locales/fr.json`.
3. Render it with `$t('key')` in Svelte markup. In TypeScript modules, read the store
   with `get(t)('key')`; do not treat `t` as a plain function.
4. Run `pnpm i18n:check` and the narrowest relevant test.

Use ICU messages for values, plurals, and selects instead of assembling sentences:

```json
"library_item_count": "{count, plural, one {# item} other {# items}}"
```

Keep every interpolation name identical across locales. Translate complete messages,
not fragments whose word order only works in English.

## Add a locale

Add `locales/xx.json`. The locale loader and `SUPPORTED_LOCALES` discover catalog files
automatically. Then add the language's self-name to the language preference options in
`reading-appearance-model.ts`.

`scripts/i18n-check.mjs` controls reference-locale parity through its
`referenceLocales` option. Add a locale there only when it must have exact English key
parity; otherwise it may be partial and use the English fallback.

The checker rejects invalid JSON, duplicate or unsorted keys, empty values, invalid ICU
syntax, unknown keys, mismatched ICU arguments, and keys outside the established feature
prefixes. The allowed prefixes live in `scripts/i18n-check.mjs`; extend that list only when
introducing a real product area. It also rejects missing keys in each reference locale.

## Apply and persist

`applyLocale` changes the active catalog and updates `<html lang>` and `<html dir>`; it
does not write preference state. `applyProfileLocale` applies the server contract:

- `undefined` means the profile has not loaded, so the current locale stays unchanged.
- `null` means System default, so local storage is cleared and the browser locale wins.
- A supported locale is normalized, stored locally for startup, and applied immediately.

Persist a signed-in user's choice through `auth.updateProfile({ locale })` before calling
`applyProfileLocale`. The empty language-selector value maps to `null`, never to an empty
locale string.

## Formatting

Use the locale-aware exports from `$lib/i18n`: `$date`, `$time`, and `$number`. Use
`$lib/utils/relative-time.ts` for relative timestamps and `$lib/utils/format.ts` for
reading duration. Do not pass hardcoded locales to `Intl` or `toLocaleString`.

## Tests

`vitest-setup.ts` initializes English for the existing suite. A focused localization
test can install catalogs synchronously and switch locale without loading the app:

```ts
setupI18nSync({ en, fr }, 'fr');
expect(get(t)('common_save')).toBe('Enregistrer');
```

Prefer one representative translated assertion per UI batch. Existing English behavior
tests should remain unchanged unless the copy contract itself changed.

## RTL readiness

Locale application already sets `document.documentElement.dir` using the language tag.
A complete logical-properties and mirrored-interaction audit is still required before an
RTL locale can be declared supported. Start the audit with:

```bash
rg -n 'margin-left|padding-left|left:' src
```

## Follow-ups

- Localize the extension toolbar's remaining copy.
- Migrate the remaining mobile strings feature by feature.
- Add a Translations section to the public contributor guide after the hosted Weblate
  project is live.
- Validate backend locale values as BCP 47 tags and localize transactional emails.
- Complete the web RTL logical-properties, icon-direction, and interaction audit.
