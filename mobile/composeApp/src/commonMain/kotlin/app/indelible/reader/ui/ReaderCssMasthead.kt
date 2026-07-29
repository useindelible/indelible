package app.indelible.reader.ui

/**
 * Left aligned, so the title shares the body's measure and left margin. The previous
 * centred stack put a centred title over left-aligned prose, which is what made it
 * read as a template.
 *
 * The summary is an explicit labelled disclosure. It used to hide behind a bare
 * circular chevron floating on a rule, which gave no clue what it would open.
 */
internal val READER_MASTHEAD_CSS =
    """
.masthead { padding: 0; }
.mh-source { display: inline-flex; align-items: center; gap: 9px; }
.mh-mark {
  width: 24px; height: 24px; border-radius: 7px; background: var(--chip-bg);
  display: inline-flex; align-items: center; justify-content: center;
  font-family: var(--sans); font-size: 12px; font-weight: 600; color: var(--reader-ink);
  box-shadow: inset 0 0 0 1px var(--hairline);
}
.mh-name {
  font-family: var(--mono); font-size: 10.5px; font-weight: 600;
  letter-spacing: 0.18em; text-transform: uppercase; color: var(--text-secondary);
}
.mh-title {
  font-family: var(--serif); font-size: 33px; font-weight: 600;
  letter-spacing: -0.028em; line-height: 1.1; color: var(--reader-ink);
  margin-top: 16px; text-align: left; text-wrap: balance;
}
.mh-meta {
  margin-top: 15px; display: flex; align-items: center; gap: 8px; flex-wrap: wrap;
  font-family: var(--mono); font-size: 10.5px; font-weight: 500;
  letter-spacing: 0.05em; color: var(--text-tertiary);
}
.mh-meta .sep { opacity: 0.45; }
.mh-rule { height: 1px; background: var(--hairline); margin: 24px 0 0; }
.sum-toggle {
  width: 100%; display: flex; align-items: center; gap: 10px; padding: 14px 0;
  background: none; border: none; cursor: pointer; color: var(--text-secondary);
  text-align: left; -webkit-tap-highlight-color: transparent;
}
.sum-toggle .lab {
  flex: 1; font-family: var(--mono); font-size: 10px; font-weight: 600;
  letter-spacing: 0.16em; text-transform: uppercase;
}
.sum-toggle .spark { width: 15px; height: 15px; color: var(--accent); flex-shrink: 0; }
.sum-toggle .chev {
  width: 15px; height: 15px; flex-shrink: 0;
  transition: transform .28s cubic-bezier(.32,.72,0,1);
}
.sum-toggle.open { color: var(--accent); }
.sum-toggle.open .chev { transform: rotate(180deg); }
.sum { display: grid; grid-template-rows: 0fr; transition: grid-template-rows .42s cubic-bezier(.32,.72,0,1); }
.sum.open { grid-template-rows: 1fr; }
.sum-in { overflow: hidden; min-height: 0; }
.sum-card {
  padding: 0 0 18px; opacity: 0; transform: translateY(-6px);
  transition: opacity .28s ease 40ms, transform .42s cubic-bezier(.32,.72,0,1) 40ms;
}
.sum.open .sum-card { opacity: 1; transform: none; }
.sum-text {
  font-family: var(--sans); font-size: 14.5px; line-height: 1.6;
  letter-spacing: -0.005em; color: var(--text-secondary);
}
.sum-points { list-style: none; margin: 14px 0 0; padding: 0; display: flex; flex-direction: column; gap: 9px; }
.sum-points li { display: flex; gap: 10px; font-size: 13px; line-height: 1.5; color: var(--text-tertiary); }
.sum-points li::before {
  content: ''; flex-shrink: 0; width: 4px; height: 4px; border-radius: 999px;
  background: var(--accent); margin-top: 8px;
}
.sum-foot { margin-top: 16px; }
.sum-ask {
  display: inline-flex; align-items: center; gap: 7px; padding: 6px 0;
  background: none; border: none; cursor: pointer;
  font-family: var(--mono); font-size: 10px; font-weight: 600;
  letter-spacing: 0.14em; text-transform: uppercase; color: var(--accent);
}
.sum-ask:hover, .sum-ask:active { opacity: 0.72; }
    """.trimIndent()
