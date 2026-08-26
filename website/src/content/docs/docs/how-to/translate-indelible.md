---
title: Translate Indelible
sidebar:
  order: 8
---

Indelible uses English as its source and fallback language. French is the reference
translation and currently has complete key parity with English. The initial French copy
was authored by a non speaker using translation tools, so review from native speakers is especially welcome.

## Translate with Weblate

The hosted Weblate project is named `indelible` and has three components:

- **Web** for the Svelte JSON catalogs
- **Mobile** for Compose Multiplatform Android XML resources
- **Extension** for WebExtension `messages.json` catalogs

Choose a component and translate from English. Weblate preserves placeholders and opens
repository changes for maintainer review. If your language is not listed, request it from
the project maintainers in Weblate rather than reusing a similar locale.

The hosted project is a maintainer setup item. Until its public URL is announced, use the
pull-request workflow below.

## Translate with a pull request

Fork the repository, create a feature branch, and update the catalog for your platform:

| Platform | Catalog location | Check |
| --- | --- | --- |
| Web | `web/src/lib/i18n/locales/<locale>.json` | `cd web && pnpm i18n:check` |
| Mobile | `mobile/composeApp/src/commonMain/composeResources/values-<locale>/strings.xml` | `cd mobile && ./gradlew :composeApp:i18nCheck` |
| Extension | `extension/public/_locales/<locale>/messages.json` | `cd extension && npm run i18n:check` |

Keep placeholders unchanged and translate the complete sentence around them. Do not add
machine-generated translations without reviewing terminology and punctuation in the UI.
Mention your language and the screens you checked in the pull request.

## How language selection works

- **Web:** choose **System default**, **English**, or **Français** under Preferences →
  Reading & appearance → Language. An explicit choice follows your account across
  browsers; System default follows each browser independently.
- **Mobile:** the app follows the device language. Android 13 and later also expose
  English and French in per-app language settings.
- **Extension:** browser menus and extension status copy follow the browser UI language.

Unsupported or incomplete translations safely fall back to English. If you see a raw
message key, mismatched placeholder, clipped label, or incorrect plural, please include
the platform, locale, and screen in your issue or pull request.
