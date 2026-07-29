export type ExtensionItemType = 'article' | 'pdf' | 'tweet' | 'video'

export interface ClassifiedUrl {
  itemType: ExtensionItemType
  platform?: 'arxiv' | 'youtube' | 'vimeo' | 'twitch' | 'twitter'
}

export function classifyExtensionUrl(rawUrl: string): ClassifiedUrl {
  let url: URL
  try {
    url = new URL(rawUrl)
  } catch {
    return { itemType: 'article' }
  }

  const host = url.hostname.replace(/^www\./, '').toLowerCase()
  const path = url.pathname.toLowerCase()

  if (path.endsWith('.pdf')) return { itemType: 'pdf' }
  if (host === 'arxiv.org' && (path.startsWith('/pdf/') || path.endsWith('.pdf'))) {
    return { itemType: 'pdf', platform: 'arxiv' }
  }

  if ((host === 'twitter.com' || host === 'x.com') && /^\/[^/]+\/status\/\d+/.test(url.pathname)) {
    return { itemType: 'tweet', platform: 'twitter' }
  }

  if (
    host === 'youtube.com' ||
    host === 'm.youtube.com' ||
    host === 'youtu.be' ||
    (host === 'vimeo.com' && /^\/\d+/.test(url.pathname)) ||
    host === 'player.vimeo.com' ||
    host === 'twitch.tv' ||
    host.endsWith('.twitch.tv')
  ) {
    const platform =
      host.includes('youtube') || host === 'youtu.be'
        ? 'youtube'
        : host.includes('vimeo')
          ? 'vimeo'
          : 'twitch'
    return { itemType: 'video', platform }
  }

  return { itemType: 'article' }
}

export function canExtensionSaveUrl(rawUrl: string): boolean {
  try {
    const url = new URL(rawUrl)
    return url.protocol === 'http:' || url.protocol === 'https:'
  } catch {
    return false
  }
}
