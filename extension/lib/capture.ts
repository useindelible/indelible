// Discriminated union for all messages in the capture flow.
// Using a discriminated union eliminates the need for runtime casts elsewhere.

import type { SourceLocatorPayload } from '../../shared/highlight-source'

export type CaptureMessage =
  | { action: 'capture:run' }
  | { action: 'selection:capture' }
  | { action: 'selection:result'; payload: SelectionPayload }
  | { action: 'capture:progress'; step: CaptureStep }
  | { action: 'capture:result'; payload: CapturePayload }
  | { action: 'capture:error'; message: string; payload?: CapturePayload }

export type CaptureStep = 'thumbnail' | 'extracting' | 'singlefile' | 'uploading'

export interface CapturePayload {
  url: string
  canonicalUrl?: string
  title: string
  readerHtml: string
  htmlBase64: string
  leadImageUrl?: string
  thumbnailBase64?: string
  excerpt?: string
  author?: string
  language?: string
  wordCount?: number
  readingTimeMinutes?: number
  publishedAt?: string
  itemType?: string
  videoDurationSeconds?: number
}

export type { SourceLocatorPayload } from '../../shared/highlight-source'

export interface SelectionPayload {
  text: string
  sourceLocator: SourceLocatorPayload
}

export function isCaptureMessage(value: unknown): value is CaptureMessage {
  return (
    typeof value === 'object' &&
    value !== null &&
    'action' in value &&
    typeof (value as Record<string, unknown>)['action'] === 'string' &&
    (String((value as Record<string, unknown>)['action']).startsWith('capture:') ||
      String((value as Record<string, unknown>)['action']).startsWith('selection:'))
  )
}
