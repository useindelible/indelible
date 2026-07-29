package app.indelible.reader.ui

// Lifted verbatim from the original buildCss block so the YouTube markup contract
// is unchanged by the template decomposition.
internal val READER_YOUTUBE_CSS =
    """
/* YouTube reader documents. The worker emits .yt-* markup (channel header, description,
   transcript with per-segment .t-seg[data-t] timestamps); without these rules it renders
   as raw stacked text. Mirrors web ReaderContent.svelte, with the hover timestamp reveal
   replaced by a tap toggle. */
.article-body .yt-channel-header {
  display: flex; align-items: center; gap: 10px; margin-bottom: 24px;
}
.article-body .yt-channel-avatar {
  width: 36px; height: 36px; border-radius: 50%; flex-shrink: 0;
  background: linear-gradient(135deg, var(--accent) 0%, #5856D6 100%);
  display: flex; align-items: center; justify-content: center;
  font-family: var(--sans); font-size: 14px; font-weight: 600; color: #fff !important;
}
.article-body .yt-channel-info { display: flex; flex-direction: column; gap: 1px; }
.article-body .yt-channel-name {
  font-family: var(--sans); font-size: 14px; font-weight: 600;
  letter-spacing: -0.01em; color: var(--reader-ink) !important;
}
.article-body .yt-video-stats {
  font-family: var(--sans); font-size: 12px; display: flex; align-items: center; gap: 6px;
  color: var(--text-secondary) !important;
}
.article-body .yt-stat-dot {
  width: 3px; height: 3px; border-radius: 50%; flex-shrink: 0;
  background: var(--text-tertiary);
}
.article-body .yt-description {
  font-family: var(--font-family); line-height: 1.7; letter-spacing: -0.01em;
  padding-bottom: 28px; margin-bottom: 32px;
  border-bottom: 1px solid var(--hairline);
}
.article-body .yt-transcript h2 {
  font-family: var(--sans); font-size: 11px; font-weight: 600; letter-spacing: 0.1em;
  text-transform: uppercase; color: var(--text-tertiary) !important;
  display: flex; align-items: center; gap: 12px; margin: 0 0 24px;
}
.article-body .yt-transcript h2::after {
  content: ''; flex: 1; height: 1px; background: var(--hairline);
}
.article-body .transcript-flow p { margin-bottom: 1.2em; }
.article-body .transcript-flow p:last-child { margin-bottom: 0; }
.article-body .t-seg {
  position: relative; display: inline; border-radius: 4px;
  transition: background 200ms ease; -webkit-tap-highlight-color: transparent;
}
.article-body .t-seg::before {
  content: attr(data-t); position: absolute; left: 0; bottom: calc(100% + 4px);
  font-family: var(--sans); font-size: 11px; font-weight: 500;
  font-variant-numeric: tabular-nums;
  background: var(--accent); color: #fff; padding: 1px 6px; border-radius: 4px;
  opacity: 0; transform: translateY(3px); pointer-events: none; white-space: nowrap;
  z-index: 10; transition: opacity 150ms ease, transform 150ms ease;
}
.article-body .t-seg.t-open { background: var(--accent-soft); }
.article-body .t-seg.t-open::before { opacity: 1; transform: translateY(0); }
    """.trimIndent()
