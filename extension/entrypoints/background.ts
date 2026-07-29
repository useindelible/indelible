import { connect, disconnect, getAuthState, validateToken } from '@/lib/auth'
import {
  refreshAccessToken,
  authenticatedFetch,
  checkExtensionUrl,
  createExtensionHighlight,
  getEntryAsset,
  getExtensionSavedEntry,
  listExtensionHighlights,
  patchExtensionSavedEntry,
  reprocessDocument,
  saveExtensionFullArchive,
  saveExtensionReaderArchive,
  syncEntryTags,
  upsertEntryNote,
  type LocatorPayload,
} from '@/lib/api'
import { getServerUrl } from '@/lib/storage'
import { buildFullArchiveBody, buildReaderSaveFallbackBody } from '@/lib/archive'
import {
  isCaptureMessage,
  type CaptureMessage,
  type CapturePayload,
  type SelectionPayload,
} from '@/lib/capture'
import { canExtensionSaveUrl } from '@/lib/content-type'

const SAVE_PAGE_MENU_ID = 'indelible-save-page'
const SAVE_SELECTION_MENU_ID = 'indelible-save-selection'
const SAVED_BADGE_MS = 6000
const FULL_ARCHIVE_CONTENT_SCRIPT = '/content-scripts/full-archive.js'

const savingAnimations = new Map<number, ReturnType<typeof setInterval>>()
const savedTimers = new Map<number, ReturnType<typeof setTimeout>>()
const captureInProgress = new Map<number, boolean>()
const toolbarSuppressed = new Set<number>()

function reportBestEffortFailure(operation: string, error: unknown): void {
  console.debug(`[Indelible] ${operation} failed`, error)
}

export default defineBackground(() => {
  browser.runtime.onInstalled.addListener(({ reason }) => {
    if (reason === 'install') {
      console.log('Indelible extension installed')
    }
    installContextMenus()
  })

  installContextMenus()

  refreshAccessToken().catch((error) => reportBestEffortFailure('startup token refresh', error))

  browser.contextMenus.onClicked.addListener((info, tab) => {
    if (info.menuItemId === SAVE_PAGE_MENU_ID) {
      void handleCaptureStart(tab).catch((error) =>
        reportBestEffortFailure('context-menu page capture', error),
      )
      return
    }
    if (info.menuItemId === SAVE_SELECTION_MENU_ID) {
      void handleSelectionHighlight(tab).catch((error) =>
        reportBestEffortFailure('context-menu selection capture', error),
      )
    }
  })

  browser.commands.onCommand.addListener((command) => {
    if (command === 'save-current-page') {
      void handleCaptureStart().catch((error) =>
        reportBestEffortFailure('keyboard page capture', error),
      )
    }
  })

  browser.action.onClicked.addListener((tab) => {
    void handleActionClick(tab).catch(async (err) => {
      if (tab.id) {
        await setActionError(tab.id, err instanceof Error ? err.message : String(err))
      }
    })
  })

  browser.runtime.onMessage.addListener(
    (message: unknown, sender: Browser.runtime.MessageSender, sendResponse) => {
      if (!isTrustedSender(sender)) {
        sendResponse({ success: false, error: 'Untrusted sender' })
        return true
      }

      // Progress notifications from content script keep the toolbar and action badge in sync.
      if (
        typeof message === 'object' &&
        message !== null &&
        (message as Record<string, unknown>)['action'] === 'capture:progress'
      ) {
        if (sender.tab?.id && isCaptureMessage(message) && message.action === 'capture:progress') {
          const tabId = sender.tab.id
          setActionSaving(tabId, message.step)
          void getServerUrl().then((serverUrl) =>
            renderToolbar(tabId, {
              view: 'saving',
              serverUrl,
              step: message.step,
            }),
          )
        }
        sendResponse({ success: true })
        return true
      }

      // A handler that throws (e.g. a throwOnError API call) must still answer, or the message
      // port stays open and the caller's write silently drops instead of surfacing an error.
      handleMessage(message, sender)
        .then(sendResponse)
        .catch((error) =>
          sendResponse({
            success: false,
            error: error instanceof Error ? error.message : String(error),
          }),
        )
      return true
    },
  )
})

function installContextMenus(): void {
  if (!browser.contextMenus) return
  browser.contextMenus
    .removeAll()
    .then(() => {
      browser.contextMenus.create({
        id: SAVE_PAGE_MENU_ID,
        title: 'Save to Indelible',
        contexts: ['page'],
      })
      browser.contextMenus.create({
        id: SAVE_SELECTION_MENU_ID,
        title: 'Save highlight to Indelible',
        contexts: ['selection'],
      })
    })
    .catch((error) => reportBestEffortFailure('context-menu refresh', error))
}

function isTrustedSender(sender: Browser.runtime.MessageSender): boolean {
  if (sender.id !== browser.runtime.id) return false
  return true
}

interface ExtensionMessage {
  action: string
  [key: string]: unknown
}

function isExtensionMessage(value: unknown): value is ExtensionMessage {
  return typeof value === 'object' && value !== null && 'action' in value
}

async function handleMessage(
  message: unknown,
  sender: Browser.runtime.MessageSender,
): Promise<{ success: boolean; error?: string; data?: unknown }> {
  if (!isExtensionMessage(message)) {
    return { success: false, error: 'Invalid message format' }
  }

  switch (message.action) {
    case 'auth:status':
      return handleAuthStatus()

    case 'auth:start':
      return handleAuthStart(message)

    case 'auth:logout':
      return handleAuthLogout()

    case 'auth:validate':
      return handleAuthValidate()

    case 'toolbar:connect':
      return handleAuthStart(
        {
          action: 'auth:start',
          serverUrl:
            typeof message.serverUrl === 'string' ? message.serverUrl : await getServerUrl(),
        },
        sender.tab?.id,
      )

    case 'toolbar:save':
      return handleToolbarSave()

    case 'toolbar:highlight-selection':
      return handleToolbarHighlightSelection(sender.tab)

    case 'toolbar:patch-item':
      return handleToolbarPatchItem(message)

    case 'toolbar:set-tags':
      return handleToolbarSetTags(message)

    case 'toolbar:set-note':
      return handleToolbarSetNote(message)

    case 'toolbar:reprocess-document':
      return handleToolbarReprocessDocument(message)

    default:
      return { success: false, error: `Unknown action: ${message.action}` }
  }
}

async function handleToolbarReprocessDocument(
  message: ExtensionMessage,
): Promise<{ success: boolean; data?: unknown; error?: string }> {
  const documentId = typeof message.documentId === 'string' ? message.documentId : ''
  if (!documentId) return { success: false, error: 'Missing document id' }
  const result = await reprocessDocument(documentId)
  return { success: true, data: result }
}

async function handleAuthStatus(): Promise<{
  success: boolean
  data?: unknown
  error?: string
}> {
  try {
    const state = await getAuthState()
    return { success: true, data: state }
  } catch (err) {
    return { success: false, error: String(err) }
  }
}

async function handleAuthStart(
  message: ExtensionMessage,
  returnTabId?: number,
): Promise<{ success: boolean; error?: string }> {
  try {
    const serverUrl = typeof message.serverUrl === 'string' ? message.serverUrl : ''
    if (!serverUrl) {
      return { success: false, error: 'Server URL is required' }
    }
    const result = await connect(serverUrl, returnTabId)
    if (result.returnTabId) {
      await resumeToolbarAfterAuth(result.returnTabId)
    }
    return { success: true }
  } catch (err) {
    return { success: false, error: String(err) }
  }
}

async function resumeToolbarAfterAuth(tabId: number): Promise<void> {
  let tab: Browser.tabs.Tab | undefined
  try {
    tab = await browser.tabs.get(tabId)
  } catch {
    return
  }
  if (!tab?.id) return

  const auth = await getAuthState()
  if (auth.status !== 'connected') {
    await renderToolbar(tab.id, {
      view: 'disconnected',
      serverUrl: auth.serverUrl || (await getServerUrl()),
    })
    return
  }

  if (!tab.url || !canExtensionSaveUrl(tab.url)) {
    await setActionIdle(tab.id)
    await renderToolbar(tab.id, {
      view: 'unsupported',
      serverUrl: auth.serverUrl,
      url: tab.url ?? '',
    })
    return
  }

  const checked = await checkExtensionUrl(tab.url)
  if (checked.exists && checked.library_entry_id) {
    await loadToolbarPanel(tab.id, auth.serverUrl, checked.library_entry_id)
    return
  }

  await handleCaptureStart(tab)
}

async function handleAuthLogout(): Promise<{ success: boolean; error?: string }> {
  try {
    await disconnect()
    return { success: true }
  } catch (err) {
    return { success: false, error: String(err) }
  }
}

async function handleAuthValidate(): Promise<{
  success: boolean
  data?: unknown
  error?: string
}> {
  try {
    const valid = await validateToken()
    return { success: true, data: { valid } }
  } catch (err) {
    return { success: false, error: String(err) }
  }
}

async function handleActionClick(tab: Browser.tabs.Tab): Promise<void> {
  if (!tab.id) return
  const tabId = tab.id

  const auth = await getAuthState()
  if (auth.status !== 'connected') {
    await renderToolbar(tabId, {
      view: 'disconnected',
      serverUrl: auth.serverUrl || (await getServerUrl()),
    })
    return
  }

  if (!tab.url || !canExtensionSaveUrl(tab.url)) {
    await setActionIdle(tabId)
    await renderToolbar(tabId, {
      view: 'unsupported',
      serverUrl: auth.serverUrl,
      url: tab.url ?? '',
    })
    return
  }

  // Second click while saving — un-suppress toolbar so user can see progress
  if (captureInProgress.get(tabId)) {
    toolbarSuppressed.delete(tabId)
    await renderToolbar(tabId, {
      view: 'saving',
      serverUrl: auth.serverUrl,
      url: tab.url,
      step: 'Saving…',
    })
    return
  }

  const checked = await checkExtensionUrl(tab.url)
  if (checked.exists && checked.library_entry_id) {
    await loadToolbarPanel(tabId, auth.serverUrl, checked.library_entry_id)
    return
  }

  toolbarSuppressed.add(tabId)
  captureInProgress.set(tabId, true)
  try {
    await handleCaptureStart(tab)
  } finally {
    captureInProgress.set(tabId, false)
    toolbarSuppressed.delete(tabId)
  }
}

async function handleToolbarSave(): Promise<{ success: boolean; error?: string }> {
  const [tab] = await browser.tabs.query({ active: true, currentWindow: true })
  if (!tab?.id) return { success: false, error: 'No active tab found' }
  const result = await handleCaptureStart(tab)
  if (result.success && result.data?.libraryEntryId) {
    await loadToolbarPanel(tab.id, await getServerUrl(), result.data.libraryEntryId)
  }
  return result.success ? { success: true } : { success: false, error: result.error }
}

async function handleToolbarHighlightSelection(
  sourceTab?: Browser.tabs.Tab,
): Promise<{ success: boolean; error?: string }> {
  const [tab] = sourceTab?.id
    ? [sourceTab]
    : await browser.tabs.query({ active: true, currentWindow: true })
  if (!tab?.id) return { success: false, error: 'No active tab found' }
  try {
    await handleSelectionHighlight(tab)
    return { success: true }
  } catch (err) {
    const message = err instanceof Error ? err.message : 'Highlight failed'
    await setActionError(tab.id, message)
    return { success: false, error: message }
  }
}

async function handleToolbarPatchItem(
  message: ExtensionMessage,
): Promise<{ success: boolean; error?: string }> {
  const libraryEntryId = typeof message.libraryEntryId === 'string' ? message.libraryEntryId : ''
  if (!libraryEntryId) return { success: false, error: 'Missing saved entry id' }
  const patch = isRecord(message.patch) ? message.patch : {}
  const triageState = typeof patch.triage_state === 'string' ? patch.triage_state : undefined
  const isFavorite = typeof patch.is_favorite === 'boolean' ? patch.is_favorite : undefined
  if (triageState === undefined && isFavorite === undefined) {
    return { success: true }
  }

  await patchExtensionSavedEntry(libraryEntryId, {
    triage_state: triageState,
    is_favorite: isFavorite,
  })
  return { success: true }
}

async function handleToolbarSetTags(
  message: ExtensionMessage,
): Promise<{ success: boolean; error?: string }> {
  const libraryEntryId = typeof message.libraryEntryId === 'string' ? message.libraryEntryId : ''
  const tags = Array.isArray(message.tags)
    ? message.tags.filter((tag): tag is string => typeof tag === 'string')
    : []
  if (!libraryEntryId) return { success: false, error: 'Missing saved entry id' }
  await syncEntryTags(libraryEntryId, tags)
  return { success: true }
}

async function handleToolbarSetNote(
  message: ExtensionMessage,
): Promise<{ success: boolean; error?: string }> {
  const libraryEntryId = typeof message.libraryEntryId === 'string' ? message.libraryEntryId : ''
  const body = typeof message.body === 'string' ? message.body : ''
  if (!libraryEntryId) return { success: false, error: 'Missing saved entry id' }
  await upsertEntryNote(libraryEntryId, body)
  return { success: true }
}

async function loadToolbarPanel(
  tabId: number,
  serverUrl: string,
  libraryEntryId: string,
): Promise<void> {
  const [ctx, highlightList] = await Promise.all([
    getExtensionSavedEntry(libraryEntryId),
    listExtensionHighlights(libraryEntryId),
  ])
  if (!ctx) {
    await renderToolbar(tabId, {
      view: 'error',
      serverUrl,
      message: 'Saved entry was not found',
    })
    return
  }
  await renderToolbar(tabId, {
    view: 'saved',
    serverUrl,
    readerUrl: ctx.reader_url,
    entry: ctx,
    tags: ctx.tags.map((tag) => tag.name),
    note: ctx.note?.body ?? '',
    highlights: highlightList.highlights,
  })
}

async function renderToolbar(tabId: number, state: Record<string, unknown>): Promise<void> {
  if (toolbarSuppressed.has(tabId)) return
  try {
    await ensureFullArchiveContentScript(tabId)
    await browser.tabs.sendMessage(tabId, { action: 'toolbar:render', state })
  } catch {
    // Pages like the Chrome Web Store and browser internals may not accept content-script messages.
  }
}

async function ensureFullArchiveContentScript(tabId: number): Promise<void> {
  try {
    const response = await browser.tabs.sendMessage(tabId, { action: 'indelible:ping' })
    if (isRecord(response) && response.success === true) return
  } catch {}

  await injectFullArchiveContentScript(tabId)
}

async function injectFullArchiveContentScript(tabId: number): Promise<void> {
  if (!browser.scripting?.executeScript) {
    throw new Error('MV3 scripting API is unavailable')
  }

  await browser.scripting.executeScript({
    target: { tabId },
    files: [FULL_ARCHIVE_CONTENT_SCRIPT],
  })
}

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === 'object' && value !== null
}

async function handleCaptureStart(sourceTab?: Browser.tabs.Tab): Promise<{
  success: boolean
  data?: { libraryEntryId: string; status: string; readerUrl?: string }
  error?: string
}> {
  const [activeTab] = sourceTab?.id
    ? [sourceTab]
    : await browser.tabs.query({ active: true, currentWindow: true })
  const tab = activeTab
  if (!tab?.id || !tab.url || !tab.title) {
    return { success: false, error: 'No active tab found' }
  }
  if (!canExtensionSaveUrl(tab.url)) {
    await setActionIdle(tab.id)
    return { success: false, error: 'This page cannot be saved' }
  }

  const tabId = tab.id
  const tabUrl = tab.url
  const tabTitle = tab.title
  const serverUrl = await getServerUrl()

  let captureResult: CaptureMessage
  try {
    setActionSaving(tabId, 'extracting')
    await renderToolbar(tabId, {
      view: 'saving',
      serverUrl,
      url: tabUrl,
      step: 'extracting',
    })
    await ensureFullArchiveContentScript(tabId)
    captureResult = (await browser.tabs.sendMessage(tabId, {
      action: 'capture:run',
    } satisfies CaptureMessage)) as CaptureMessage
  } catch (err) {
    const message = err instanceof Error ? err.message : 'Content script failed'
    await setActionError(tabId, message)
    await renderToolbar(tabId, { view: 'error', serverUrl, message })
    return { success: false, error: message }
  }

  if (captureResult.action === 'capture:error') {
    if (
      (captureResult.message === 'singlefile-failed' ||
        captureResult.message === 'monolith-too-large') &&
      captureResult.payload?.readerHtml
    ) {
      try {
        const body = buildReaderSaveFallbackBody(
          tabUrl,
          captureResult.payload.canonicalUrl,
          tabTitle,
          captureResult.payload.readerHtml,
          captureResult.payload.leadImageUrl,
          captureResult.payload.excerpt,
          captureResult.payload.author,
          captureResult.payload.itemType,
        )
        const data = await saveExtensionReaderArchive(body)
        await setActionSaved(tabId)
        await loadToolbarPanel(tabId, serverUrl, data.library_entry_id)
        return {
          success: true,
          data: {
            libraryEntryId: data.library_entry_id,
            status: data.status,
            readerUrl: data.reader_url,
          },
        }
      } catch (err) {
        const message = err instanceof Error ? err.message : 'Reader save failed'
        await setActionError(tabId, message)
        await renderToolbar(tabId, { view: 'error', serverUrl, message })
        return { success: false, error: message }
      }
    }
    await setActionError(tabId, captureResult.message)
    await renderToolbar(tabId, { view: 'error', serverUrl, message: captureResult.message })
    return { success: false, error: captureResult.message }
  }

  if (captureResult.action !== 'capture:result') {
    await setActionError(tabId, 'Unexpected response from content script')
    await renderToolbar(tabId, {
      view: 'error',
      serverUrl,
      message: 'Unexpected response from content script',
    })
    return { success: false, error: 'Unexpected response from content script' }
  }

  const payload: CapturePayload = captureResult.payload

  setActionSaving(tabId, 'uploading')
  await renderToolbar(tabId, { view: 'saving', serverUrl, url: tabUrl, step: 'uploading' })

  try {
    const body = buildFullArchiveBody(
      payload.url,
      payload.canonicalUrl,
      payload.title,
      payload.readerHtml,
      payload.htmlBase64,
      payload.leadImageUrl,
      payload.excerpt,
      payload.author,
      payload.language,
      payload.publishedAt,
      payload.itemType,
    )
    const data = await saveExtensionFullArchive(body)
    await setActionSaved(tabId)
    await loadToolbarPanel(tabId, serverUrl, data.library_entry_id)
    return {
      success: true,
      data: {
        libraryEntryId: data.library_entry_id,
        status: data.status,
        readerUrl: data.reader_url,
      },
    }
  } catch (err) {
    try {
      const fallback = buildReaderSaveFallbackBody(
        payload.url,
        payload.canonicalUrl,
        payload.title,
        payload.readerHtml,
        payload.leadImageUrl,
        payload.excerpt,
        payload.author,
        payload.itemType,
      )
      const data = await saveExtensionReaderArchive(fallback)
      await setActionSaved(tabId)
      await loadToolbarPanel(tabId, serverUrl, data.library_entry_id)
      return {
        success: true,
        data: {
          libraryEntryId: data.library_entry_id,
          status: data.status,
          readerUrl: data.reader_url,
        },
      }
    } catch {
      const message = err instanceof Error ? err.message : 'Upload failed'
      await setActionError(tabId, message)
      await renderToolbar(tabId, { view: 'error', serverUrl, message })
      return { success: false, error: message }
    }
  }
}

async function handleSelectionHighlight(tab?: Browser.tabs.Tab): Promise<void> {
  const [activeTab] = tab?.id
    ? [tab]
    : await browser.tabs.query({ active: true, currentWindow: true })
  if (!activeTab?.id || !activeTab.url) return
  if (!canExtensionSaveUrl(activeTab.url)) {
    await setActionIdle(activeTab.id)
    return
  }

  setActionSaving(activeTab.id, 'extracting')
  await ensureFullArchiveContentScript(activeTab.id)

  const selectionResult = (await browser.tabs.sendMessage(activeTab.id, {
    action: 'selection:capture',
  } satisfies CaptureMessage)) as CaptureMessage

  if (selectionResult.action !== 'selection:result') {
    const message =
      selectionResult.action === 'capture:error'
        ? selectionResult.message
        : 'No selected text found'
    await setActionError(activeTab.id, message)
    return
  }

  const selection = selectionResult.payload
  let libraryEntryId = await findSavedEntryForUrl(activeTab.url)
  if (!libraryEntryId) {
    const saved = await handleCaptureStart(activeTab)
    libraryEntryId = saved.data?.libraryEntryId
  }
  if (!libraryEntryId) {
    await setActionError(activeTab.id, 'Save the page before creating a highlight')
    return
  }

  const locator = await buildReaderLocator(libraryEntryId, selection)
  await createExtensionHighlight(libraryEntryId, {
    color: 'yellow',
    text_content: selection.text,
    locator,
    source_locator: selection.sourceLocator,
  })
  await setActionSaved(activeTab.id)
  await loadToolbarPanel(activeTab.id, await getServerUrl(), libraryEntryId)
}

async function findSavedEntryForUrl(url: string): Promise<string | undefined> {
  const checked = await checkExtensionUrl(url)
  return checked.exists ? (checked.library_entry_id ?? undefined) : undefined
}

async function buildReaderLocator(
  libraryEntryId: string,
  selection: SelectionPayload,
): Promise<LocatorPayload | undefined> {
  const html = await fetchReadableHtml(libraryEntryId)
  if (html === undefined) return undefined

  const text = htmlToText(html)
  const match = findUniqueNormalizedMatch(text, selection.text)
  if (!match) return undefined

  return {
    type: 'html',
    start_offset: match.start,
    end_offset: match.end,
  }
}

async function fetchReadableHtml(libraryEntryId: string): Promise<string | undefined> {
  try {
    const asset = await getEntryAsset(libraryEntryId, 'readable_html')
    if (asset?.download_url) {
      // download_url targets the API asset proxy, which requires auth.
      const download = await authenticatedFetch(new URL(asset.download_url).pathname)
      if (download.ok) return await download.text()
    }
  } catch {}

  return undefined
}

function htmlToText(html: string): string {
  return html
    .replace(/<script\b[^>]*>[\s\S]*?<\/script>/gi, '')
    .replace(/<style\b[^>]*>[\s\S]*?<\/style>/gi, '')
    .replace(/<[^>]+>/g, '')
    .replace(/&nbsp;/g, ' ')
    .replace(/&amp;/g, '&')
    .replace(/&lt;/g, '<')
    .replace(/&gt;/g, '>')
    .replace(/&quot;/g, '"')
    .replace(/&#39;/g, "'")
}

function findUniqueNormalizedMatch(
  source: string,
  needle: string,
): { start: number; end: number } | undefined {
  const sourceNorm = normalizeWithMap(source)
  const needleNorm = normalizeWithMap(needle).text.trim()
  if (!needleNorm) return undefined

  const first = sourceNorm.text.indexOf(needleNorm)
  if (first === -1) return undefined
  if (sourceNorm.text.indexOf(needleNorm, first + needleNorm.length) !== -1) return undefined

  const lastNormIndex = first + needleNorm.length - 1
  return {
    start: sourceNorm.map[first] ?? 0,
    end: (sourceNorm.map[lastNormIndex] ?? source.length - 1) + 1,
  }
}

function normalizeWithMap(input: string): { text: string; map: number[] } {
  let text = ''
  const map: number[] = []
  let pendingSpace = false

  for (let i = 0; i < input.length; i += 1) {
    const char = input[i]
    if (char === undefined) continue
    if (/\s/.test(char)) {
      pendingSpace = text.length > 0
      continue
    }
    if (pendingSpace) {
      text += ' '
      map.push(i)
      pendingSpace = false
    }
    text += char
    map.push(i)
  }

  return { text, map }
}

async function setActionIdle(tabId: number): Promise<void> {
  stopActionTimers(tabId)
  await browser.action.setBadgeText({ tabId, text: '' })
  await browser.action.setTitle({ tabId, title: 'Indelible' })
}

function setActionSaving(tabId: number, step: string): void {
  stopActionTimers(tabId)
  void browser.action.setBadgeBackgroundColor({ tabId, color: '#d97706' })
  void browser.action.setTitle({ tabId, title: `Indelible is saving: ${step}` })
  void browser.action.setBadgeText({ tabId, text: ' ' })
}

async function setActionSaved(tabId: number): Promise<void> {
  stopActionTimers(tabId)
  await browser.action.setBadgeBackgroundColor({ tabId, color: '#16a34a' })
  await browser.action.setBadgeText({ tabId, text: ' ' })
  await browser.action.setTitle({ tabId, title: 'Saved to Indelible' })
  savedTimers.set(
    tabId,
    setTimeout(() => {
      void setActionIdle(tabId)
    }, SAVED_BADGE_MS),
  )
}

async function setActionError(tabId: number, message: string): Promise<void> {
  stopActionTimers(tabId)
  await browser.action.setBadgeBackgroundColor({ tabId, color: '#dc2626' })
  await browser.action.setBadgeText({ tabId, text: '!' })
  await browser.action.setTitle({ tabId, title: `Indelible save failed: ${message}` })
}

function stopActionTimers(tabId: number): void {
  const savingTimer = savingAnimations.get(tabId)
  if (savingTimer) clearInterval(savingTimer)
  savingAnimations.delete(tabId)

  const savedTimer = savedTimers.get(tabId)
  if (savedTimer) clearTimeout(savedTimer)
  savedTimers.delete(tabId)
}
