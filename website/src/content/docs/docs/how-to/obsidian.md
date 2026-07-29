---
title: Sync to Obsidian
sidebar:
  order: 6
---

The Indelible Obsidian plugin pulls your highlights and saved documents into your
vault as markdown. It's a one-way sync: Indelible writes into the vault and your
edits there are never sent back.

## Install the plugin

The plugin isn't in the Obsidian community registry yet, so install it manually:

1. Copy the built plugin files into `<your-vault>/.obsidian/plugins/indelible/`
   (or add the repo through the BRAT community plugin).
2. In Obsidian, open **Settings → Community plugins**, enable **Indelible**.

## Configure

In the plugin's settings:

1. **API token**: create an API token for your account in Indelible, then select it
   in the plugin (it's read from Obsidian's secret storage).
2. **API base URL**: your Indelible instance (self-hosters point this at their own
   URL).
3. **Base folder**: where synced notes are written (default `Indelible/`).
4. **Sync interval**: manual, hourly, every 12 hours, daily, or weekly, plus an optional
   sync when the vault opens.

## Sync

Run **Sync your data now** from the command palette, or let the scheduled sync run.
The plugin appends new highlights and documents into your chosen folder; with
append-only mode your own edits to those notes are preserved. Customize the markdown
template from **Settings → Integrations → Obsidian** in the Indelible web app.

No operator configuration is needed. The plugin authenticates with your personal API
token, so it works against any instance.
