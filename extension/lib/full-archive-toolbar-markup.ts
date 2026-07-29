import type { ToolbarState } from './full-archive-toolbar'
import { escapeAttr, escapeHtml, safeHttpUrl } from './html'

const BRAND_MARK = `<svg class="brand-mark" viewBox="0 0 200 200" aria-hidden="true"><rect width="200" height="200" rx="42" fill="#0071E3"/><circle cx="100" cy="100" r="80" stroke="#fff" stroke-width="1.875" opacity=".32"/><g fill="#fff" opacity=".62"><circle cx="100" cy="32.5" r="4.25"/><circle cx="147.5" cy="52.5" r="4.25"/><circle cx="167.5" cy="100" r="4.25"/><circle cx="147.5" cy="147.5" r="4.25"/><circle cx="100" cy="167.5" r="4.25"/><circle cx="52.5" cy="147.5" r="4.25"/><circle cx="32.5" cy="100" r="4.25"/><circle cx="52.5" cy="52.5" r="4.25"/></g><g fill="#fff"><rect x="92.5" y="35" width="15" height="50" rx="7.5"/><rect x="92.5" y="115" width="15" height="50" rx="7.5"/><rect x="35" y="92.5" width="50" height="15" rx="7.5"/><rect x="115" y="92.5" width="50" height="15" rx="7.5"/></g><circle cx="100" cy="100" r="23.75" fill="#fff"/><circle cx="100" cy="100" r="8.75" fill="#0071E3"/></svg>`
const IC_TAG = `<svg class="ic" viewBox="0 0 16 16"><path d="M2.4 2.4h4.6a1 1 0 0 1 .7.3l4.3 4.3a1 1 0 0 1 0 1.4l-3.4 3.4a1 1 0 0 1-1.4 0L3 8.1a1 1 0 0 1-.6-.7V2.4z"/><circle cx="4.6" cy="4.6" r="0.9" fill="currentColor" stroke="none"/><path d="M12.7 10.5v3.6M10.9 12.3h3.6"/></svg>`
const IC_NOTE = `<svg class="ic" viewBox="0 0 16 16"><path d="M2.2 5a2 2 0 0 1 2-2h5a2 2 0 0 1 2 2v2.5a2 2 0 0 1-2 2H5.4L2.8 12v-2.5h-.6z"/><path d="M13 10v3.6M11.2 11.8h3.6"/></svg>`
const IC_STAR = `<svg class="ic" viewBox="0 0 16 16"><path d="M8 2l1.85 3.95 4.35.46-3.25 2.97.93 4.27L8 11.55 4.12 13.65l.93-4.27L1.8 6.41l4.35-.46z"/></svg>`
const IC_CLOCK = `<svg class="ic" viewBox="0 0 16 16"><circle cx="8" cy="8" r="5.4"/><path d="M8 5v3.2l2.1 1.3"/></svg>`
const IC_CHEVRON_DOWN = `<svg class="ic" viewBox="0 0 16 16"><path d="M4 6l4 4 4-4"/></svg>`
const IC_CHEVRON_UP = `<svg class="ic" viewBox="0 0 16 16"><path d="M4 10l4-4 4 4"/></svg>`
const IC_CLOSE = `<svg class="ic" viewBox="0 0 16 16"><path d="M4.2 4.2l7.6 7.6M11.8 4.2l-7.6 7.6"/></svg>`
const IC_ARROW_RIGHT = `<svg class="ic" viewBox="0 0 16 16"><path d="M6 4l4 4-4 4"/></svg>`
const IC_REFRESH = `<svg class="ic" viewBox="0 0 16 16"><path d="M12.5 5.5A5 5 0 1 0 13 9"/><path d="M12.5 2v3.5H9"/></svg>`
const TI_INBOX = `<svg class="ti-ic" viewBox="0 0 16 16"><path d="M2.5 9.5h3l1.5 2.5h2l1.5-2.5h3v3a1 1 0 01-1 1H3.5a1 1 0 01-1-1v-3z"/><path d="M5 9.5V7a1 1 0 011-1h4a1 1 0 011 1v2.5"/></svg>`
const TI_CLOCK = `<svg class="ti-ic" viewBox="0 0 16 16"><circle cx="8" cy="8" r="5.4"/><path d="M8 5v3.2l2.1 1.3"/></svg>`
const TI_ARCHIVE = `<svg class="ti-ic" viewBox="0 0 16 16"><rect x="2.5" y="4.5" width="11" height="2" rx=".5"/><path d="M3.5 6.5v6a1 1 0 001 1h7a1 1 0 001-1v-6"/><path d="M6 10h4"/></svg>`
const TI_CHECK = `<svg class="ti-check" viewBox="0 0 14 14"><path d="M2 7l3.5 3.5 6.5-7"/></svg>`

export function triageLabel(state: string | undefined): string {
  if (state === 'inbox') return 'Inbox'
  if (state === 'archive') return 'Archive'
  return 'Later'
}

export function triageIcon(state: string | undefined): string {
  if (state === 'inbox')
    return `<svg class="ic" viewBox="0 0 16 16"><path d="M2.5 9.5h3l1.5 2.5h2l1.5-2.5h3v3a1 1 0 01-1 1H3.5a1 1 0 01-1-1v-3z"/><path d="M5 9.5V7a1 1 0 011-1h4a1 1 0 011 1v2.5"/></svg>`
  if (state === 'archive')
    return `<svg class="ic" viewBox="0 0 16 16"><rect x="2.5" y="4.5" width="11" height="2" rx=".5"/><path d="M3.5 6.5v6a1 1 0 001 1h7a1 1 0 001-1v-6"/><path d="M6 10h4"/></svg>`
  return IC_CLOCK
}

function formatRelativeTime(iso: string | undefined): string {
  if (!iso) return ''
  const ms = Date.now() - new Date(iso).getTime()
  const sec = Math.floor(ms / 1000)
  if (sec < 60) return 'just now'
  const min = Math.floor(sec / 60)
  if (min < 60) return `${min}m ago`
  const hr = Math.floor(min / 60)
  if (hr < 24) return `${hr}h ago`
  return `${Math.floor(hr / 24)}d ago`
}

export function toolbarMarkup(state: ToolbarState): string {
  const entry = state.entry
  const highlightCount = state.highlights?.length ?? 0
  const readerUrl = safeHttpUrl(state.readerUrl)
  const triage = entry?.triage_state ?? 'later'
  const savedAt = entry?.saved_at ? formatRelativeTime(entry.saved_at) : ''
  const domainMeta = [domainFromUrl(entry?.url), savedAt ? `saved ${savedAt}` : '']
    .filter(Boolean)
    .join(' · ')

  const tagChipsHtml = (state.tags ?? [])
    .map(
      (t) =>
        `<span class="tag-chip" data-tag="${escapeAttr(t)}">${escapeHtml(t)}<button class="tag-remove" title="Remove">×</button></span>`,
    )
    .join('')

  const tagPanel = `
    <div class="trs-panel tag-panel" style="display:none">
      <div class="panel-header">
        <span class="panel-title">Tags</span>
        <button class="ic-btn js-panel-close" title="Close">${IC_CLOSE}</button>
      </div>
      <div class="tag-chips">${tagChipsHtml}</div>
      <div class="tag-input-wrap">
        <input type="text" class="tag-input" placeholder="Add tag…">
        <button class="btn-primary js-add-tag">Add</button>
      </div>
    </div>`

  const notePanel = `
    <div class="trs-panel note-panel" style="display:none">
      <div class="panel-header">
        <span class="panel-title">Note</span>
        <button class="ic-btn js-panel-close" title="Close">${IC_CLOSE}</button>
      </div>
      <textarea class="note-textarea" placeholder="Add a note about this article…">${escapeHtml(state.note ?? '')}</textarea>
      <div class="note-actions">
        <button class="btn-ghost js-cancel-note">Cancel</button>
        <button class="btn-primary js-save-note">Save note</button>
      </div>
    </div>`

  const triageMenu = `
    <div class="triage-menu" style="display:none">
      <button class="triage-item${triage === 'inbox' ? ' active' : ''}" data-value="inbox">${TI_INBOX}<span>Inbox</span>${TI_CHECK}<span class="spacer"></span></button>
      <button class="triage-item${triage === 'later' ? ' active' : ''}" data-value="later">${TI_CLOCK}<span>Later</span>${TI_CHECK}<span class="spacer"></span></button>
      <button class="triage-item${triage === 'archive' ? ' active' : ''}" data-value="archive">${TI_ARCHIVE}<span>Archive</span>${TI_CHECK}<span class="spacer"></span></button>
    </div>`

  let barContent: string

  switch (state.view) {
    case 'saved':
      barContent = `
        <div class="left-lockup">
          ${BRAND_MARK}
          <a class="btn-text" href="${readerUrl}" target="_blank" rel="noopener noreferrer">Open reader${IC_ARROW_RIGHT}</a>
          ${highlightCount > 0 ? `<span class="count-pill">${highlightCount}</span>` : ''}
        </div>
        <div class="item">
          <span class="t">${escapeHtml(entry?.title ?? 'Saved page')}</span>
          ${domainMeta ? `<span class="dot">·</span><span class="s">${escapeHtml(domainMeta)}</span>` : ''}
        </div>
        <div class="group">
          <div class="toggle-group">
            <span class="switch js-toggle off"><span class="knob"></span></span>
            <span>Auto-highlight</span>
          </div>
          <span class="vr"></span>
          <button class="ic-btn js-tag-btn" title="Tags">${IC_TAG}</button>
          <button class="ic-btn js-note-btn" title="Note">${IC_NOTE}</button>
          <button class="ic-btn js-star-btn${entry?.is_favorite ? ' starred' : ''}" title="Star">${IC_STAR}</button>
          ${entry?.document_id ? `<button class="ic-btn js-reprocess-btn" title="Retry processing" aria-label="Retry processing">${IC_REFRESH}</button><span class="status js-reprocess-status" aria-live="polite"></span>` : ''}
          <span class="vr"></span>
          <button class="dropdown js-triage">
            <span class="triage-ic">${triageIcon(triage)}</span>
            <span class="triage-label">${triageLabel(triage)}</span>
            ${IC_CHEVRON_DOWN}
          </button>
          <button class="ic-btn js-minimize" title="Minimize">${IC_CHEVRON_UP}</button>
        </div>`
      return `<div class="bar" role="region" aria-label="Indelible controls">${barContent}</div>${tagPanel}${notePanel}${triageMenu}`

    case 'saving':
      barContent = `
        <div class="left-lockup">
          ${BRAND_MARK}
          <span class="item-name">Saving this page</span>
        </div>
        <div class="item">
          <span class="s">Extracting readable content and creating the archive</span>
        </div>
        <div class="group">
          <span class="status saving"><span class="spinner"></span>Saving</span>
          <button class="ic-btn js-dismiss" title="Dismiss">${IC_CLOSE}</button>
        </div>`
      break

    case 'checking':
      barContent = `
        <div class="left-lockup">
          ${BRAND_MARK}
          <span class="item-name">Checking this page</span>
        </div>
        <div class="item">
          <span class="s">Looking for an existing saved item after your click</span>
        </div>
        <div class="group">
          <span class="status checking"><span class="spinner"></span>Checking</span>
          <button class="ic-btn js-dismiss" title="Dismiss">${IC_CLOSE}</button>
        </div>`
      break

    case 'disconnected':
      barContent = `
        <div class="left-lockup">
          ${BRAND_MARK}
          <span class="item-name">Connect Indelible</span>
        </div>
        <div class="item">
          <span class="s">Authenticate your workspace before saving this page</span>
        </div>
        <div class="group">
          <input class="url-input" data-role="server-url" type="text" value="${escapeAttr(state.serverUrl ?? 'https://useindelible.com')}" placeholder="https://useindelible.com" spellcheck="false" autocomplete="off">
          <button class="btn-primary" data-action="connect">Connect</button>
          <button class="ic-btn js-dismiss" title="Dismiss">${IC_CLOSE}</button>
        </div>`
      break

    case 'connecting':
      barContent = `
        <div class="left-lockup">
          ${BRAND_MARK}
          <span class="item-name">Connecting Indelible</span>
        </div>
        <div class="item">
          <span class="s">Opening secure browser authorization</span>
        </div>
        <div class="group">
          <span class="status checking"><span class="spinner"></span>Connecting</span>
          <button class="ic-btn js-dismiss" title="Dismiss">${IC_CLOSE}</button>
        </div>`
      break

    case 'auth-error':
      barContent = `
        <div class="left-lockup">
          ${BRAND_MARK}
          <span class="item-name">Couldn’t connect</span>
        </div>
        <div class="item">
          <span class="s">${escapeHtml(state.message ?? 'Authorization could not be started. Please try again.')}</span>
        </div>
        <div class="group">
          <button class="btn-primary" data-action="connect">Try again</button>
          <button class="ic-btn js-dismiss" title="Dismiss">${IC_CLOSE}</button>
        </div>`
      break

    case 'already-saved':
      barContent = `
        <div class="left-lockup">
          ${BRAND_MARK}
          <span class="item-name">Already in your library</span>
        </div>
        <div class="item">
          <span class="s">This page was saved before — refresh to update or create a new entry</span>
        </div>
        <div class="group">
          <button class="btn-ghost" data-action="refresh">Refresh</button>
          <button class="ic-btn js-dismiss" title="Dismiss">${IC_CLOSE}</button>
        </div>`
      break

    case 'unsupported':
      barContent = `
        <div class="left-lockup">
          ${BRAND_MARK}
          <span class="item-name">Can't save this page</span>
        </div>
        <div class="item">
          <span class="s">This type of page cannot be archived</span>
        </div>
        <div class="group">
          <button class="ic-btn js-dismiss" title="Dismiss">${IC_CLOSE}</button>
        </div>`
      break

    default: // error
      barContent = `
        <div class="left-lockup">
          ${BRAND_MARK}
          <span class="item-name">Something went wrong</span>
        </div>
        <div class="item">
          <span class="s">${escapeHtml(state.message ?? 'An error occurred while saving')}</span>
        </div>
        <div class="group">
          <button class="ic-btn js-dismiss" title="Dismiss">${IC_CLOSE}</button>
        </div>`
      break
  }

  return `<div class="bar" role="region" aria-label="Indelible controls">${barContent}</div>`
}

function domainFromUrl(url: string | undefined): string | undefined {
  if (!url) return undefined
  try {
    return new URL(url).hostname.replace(/^www\./, '')
  } catch {
    return undefined
  }
}
