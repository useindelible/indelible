---
title: Save with the browser extension
sidebar:
  order: 2
---

The Indelible extension saves the page you're on, full content and any text you
select as a highlight, straight to your library. It works in Chrome, Firefox, and
Edge.

## Install

The extension isn't in the web stores yet. For now, build it from source and load it
unpacked:

```bash
cd extension
npm install
npm run build          # Chrome/Edge: output in .output/chrome-mv3
npm run build:firefox  # Firefox: output in .output/firefox-mv3
```

- **Chrome / Edge**: open `chrome://extensions`, enable Developer mode, choose
  "Load unpacked", and select the build output folder.
- **Firefox**: open `about:debugging` → This Firefox → "Load Temporary Add-on" and
  select the built manifest.

## Connect it to your instance

1. Click the Indelible icon in the toolbar. It opens your instance's authorization
   page (`/extension/auth`).
2. Sign in and approve. The extension uses a PKCE OAuth flow and stores its token
   locally; your password never touches the extension.

If you self-host, point the extension at your own instance URL in its settings
before connecting.

## Save a page

- Click the toolbar icon, or press **Alt+Shift+S**, to save the current page.
- Right-click → **Save to Indelible** does the same from the context menu.
- Select text first, then right-click → **Save highlight to Indelible** to capture
  that passage as a highlight.

After saving, an inline toolbar lets you add tags and notes without leaving the page.
