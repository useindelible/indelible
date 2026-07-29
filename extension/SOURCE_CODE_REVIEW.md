# Indelible Extension Source Code Review

This extension is built with WXT and ships Manifest V3 artifacts for Chrome, Edge, and Firefox.

## Build Commands

Install dependencies from the repository root:

```bash
npm --prefix extension install
```

Build unpacked store artifacts:

```bash
npm --prefix extension run build
npm --prefix extension run build:edge
npm --prefix extension run build:firefox
```

Create submission zips:

```bash
npm --prefix extension run zip
npm --prefix extension run zip:edge
npm --prefix extension run zip:firefox
```

Run the full store verification:

```bash
npm --prefix extension run verify:stores
```

Expected output directories:

```text
extension/.output/chrome-mv3
extension/.output/edge-mv3
extension/.output/firefox-mv3
```

Expected zip files:

```text
extension/.output/ind-extension-0.1.0-chrome.zip
extension/.output/ind-extension-0.1.0-edge.zip
extension/.output/ind-extension-0.1.0-firefox.zip
extension/.output/ind-extension-0.1.0-sources.zip
```

## Dependency Notes

- `defuddle` extracts reader-friendly article content locally in the page context.
- `single-file-core` produces the archived page HTML.
- SingleFile hook scripts are copied from `node_modules/single-file-core` into `public/single-file/` by `scripts/copy-singlefile.js`.

## Remote Code

The extension does not execute remote hosted JavaScript. All extension code and SingleFile helper scripts are bundled into the submitted artifact.

## Permissions Summary

- `activeTab` grants access to the current tab only after a user gesture.
- `scripting` injects the bundled full-archive content script after that user gesture.
- `contextMenus` provides Save to Indelible actions.
- `tabs` is used for active-tab metadata and the OAuth callback tab.
- `storage` stores the refresh token, configured server origin, connection timestamp, and short-lived PKCE state.

The extension does not declare `host_permissions`, `optional_host_permissions`, or manifest-registered `content_scripts`.
