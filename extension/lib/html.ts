export function escapeHtml(value: string): string {
  return value.replace(/[&<>"']/g, (character) => {
    switch (character) {
      case '&':
        return '&amp;'
      case '<':
        return '&lt;'
      case '>':
        return '&gt;'
      case '"':
        return '&quot;'
      default:
        return '&#39;'
    }
  })
}

export function escapeAttr(value: string): string {
  return escapeHtml(value)
}

// Entity encoding cannot neutralize an executable URL scheme, so validate before interpolation.
export function safeHttpUrl(value: string | undefined): string {
  if (!value) return '#'
  try {
    const parsed = new URL(value)
    if (parsed.protocol !== 'http:' && parsed.protocol !== 'https:') return '#'
  } catch {
    return '#'
  }
  return escapeAttr(value)
}
