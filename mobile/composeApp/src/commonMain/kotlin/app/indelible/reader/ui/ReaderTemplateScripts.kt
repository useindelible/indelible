package app.indelible.reader.ui

/**
 * JavaScript bridge injected into the reader WebView: scroll/selection/highlight
 * listeners plus the `window.*` helpers the native side calls (typography updates,
 * summary toggle, sentence speaking, highlight application).
 * Kept beside [ReaderHtmlTemplate] so the template builder stays within the
 * file-size cap as more bridge helpers land in later reader phases.
 */
internal val READER_BRIDGE_JS: String =
    """
(function() {
  // Android WebView lays this document out with a zero-height CSS viewport: 100%,
  // 100vh and 100dvh all resolve to 0, even on an element parented to <html>, while
  // window.innerHeight reports the real height. Without this the fixed frame
  // collapses, .rscroll has nothing to fill, and body's overflow:hidden propagates
  // to the viewport so nothing scrolls either. The stylesheet keeps height:100% for
  // engines that resolve it correctly; this makes the frame definite everywhere.
  function sizeFrame() {
    var px = window.innerHeight + 'px';
    document.documentElement.style.height = px;
    document.body.style.height = px;
  }
  sizeFrame();
  window.addEventListener('resize', sizeFrame);
  window.addEventListener('orientationchange', sizeFrame);
  if (window.visualViewport) {
    window.visualViewport.addEventListener('resize', sizeFrame);
  }

  var scroller = document.querySelector('.rscroll') || document.scrollingElement;
  var painting = false;

  function paintScroll() {
    painting = false;
    var y = scroller.scrollTop;
    var max = scroller.scrollHeight - scroller.clientHeight;
    var p = max > 0 ? y / max : 0;
    document.body.style.setProperty('--y', y.toFixed(1));
    document.body.style.setProperty('--p', p.toFixed(4));
    var percent = Math.min(100, Math.max(0, p * 100));
    try { NativeBridge.onScroll(percent, y); } catch(e) {}
  }

  // Coalesced to one paint and one bridge call per frame. A WebView emits many
  // scroll events per frame, and each one re-arms a native chrome retire timer.
  scroller.addEventListener('scroll', function() {
    if (!painting) { painting = true; requestAnimationFrame(paintScroll); }
  }, { passive: true });

  // The field travels its own height, which follows the frame width, so it is
  // measured from the rendered drawing rather than hardcoded per drawing.
  // getBoundingClientRect, not offsetHeight: the drawing is an <svg>, and SVG
  // elements are SVGElement rather than HTMLElement, so they have no offsetHeight.
  // Reading it yields undefined, which silently leaves the travel unset and pins
  // the drawing over the prose instead of letting it leave the frame.
  function measureAuraTravel() {
    var art = document.querySelector('.aura-art');
    if (!art) return;
    var h = art.getBoundingClientRect().height || art.clientHeight;
    if (h > 0) {
      document.body.style.setProperty('--aura-travel', h + 'px');
    }
  }
  window.addEventListener('load', measureAuraTravel);
  window.addEventListener('resize', measureAuraTravel);
  measureAuraTravel();
  paintScroll();

  document.addEventListener('selectionchange', function() {
    var sel = window.getSelection();
    if (!sel || sel.isCollapsed || !sel.toString().trim()) {
      try { NativeBridge.onSelectionCleared(); } catch(e) {}
      return;
    }
    var text = sel.toString();
    var range = sel.getRangeAt(0);
    var articleBody = document.getElementById('article-body');
    if (!articleBody) return;
    var preRange = document.createRange();
    preRange.setStart(articleBody, 0);
    preRange.setEnd(range.startContainer, range.startOffset);
    var startOffset = preRange.toString().length;
    // endOffset must share applyHighlights' raw text-node basis (Range.toString,
    // not Selection.toString): the latter collapses inter-tag whitespace and
    // would land short, truncating the highlight end.
    var endRange = document.createRange();
    endRange.setStart(articleBody, 0);
    endRange.setEnd(range.endContainer, range.endOffset);
    var endOffset = endRange.toString().length;
    var rect = range.getBoundingClientRect();
    var rectJson = JSON.stringify({x: rect.x, y: rect.y, width: rect.width, height: rect.height});
    try { NativeBridge.onTextSelected(text, startOffset, endOffset, rectJson); } catch(e) {}
  });

  // Masthead controls are bound here rather than with onclick attributes in the
  // markup: the document's script-src carries a nonce and no 'unsafe-inline', so an
  // inline handler attribute is never compiled and the control is silently inert.
  // Delegated from the document so it survives the body being re-rendered.
  document.addEventListener('click', function(e) {
    var t = e.target;
    if (!t || !t.closest) return;
    if (t.closest('.sum-toggle')) {
      window.toggleReaderSummary();
      return;
    }
    if (t.closest('.sum-ask')) {
      window.readerSummaryAction('ask');
    }
  });

  // Reveal on a tap on the page itself. Deliberately 'click', not touchstart or
  // touchend: click is only synthesised after a tap that was not a scroll, a drag,
  // or a long-press selection, so the chrome cannot flash at the start of a swipe.
  document.addEventListener('click', function(e) {
    var t = e.target;
    // These carry their own response; revealing on top of them would fight the
    // surface they raise.
    if (t && t.closest &&
        t.closest('mark[data-highlight-id], .sum-toggle, .sum-ask, a, .t-seg')) {
      return;
    }
    var sel = window.getSelection();
    if (sel && !sel.isCollapsed) return;
    try { NativeBridge.onReaderTap(); } catch(err) {}
  });

  document.addEventListener('click', function(e) {
    var mark = e.target.closest('mark[data-highlight-id]');
    if (!mark) return;
    var hid = mark.getAttribute('data-highlight-id');
    var rect = mark.getBoundingClientRect();
    var rectJson = JSON.stringify({x: rect.x, y: rect.y, width: rect.width, height: rect.height});
    try { NativeBridge.onHighlightTapped(hid, rectJson); } catch(e) {}
  });

  window.scrollToPercent = function(percent) {
    var max = scroller.scrollHeight - scroller.clientHeight;
    if (max > 0) {
      scroller.scrollTop = (percent / 100) * max;
      paintScroll();
    }
  };

  window.scrollToAnchor = function(id, fallbackIndex) {
    var root = document.getElementById('article-body') || document.body;
    var el = null;
    if (id) {
      try { el = root.querySelector('#' + CSS.escape(id)); } catch (e) { el = null; }
    }
    if (!el) {
      var hs = root.querySelectorAll('h1,h2,h3,h4,h5,h6');
      if (fallbackIndex >= 0 && fallbackIndex < hs.length) el = hs[fallbackIndex];
    }
    if (!el) return;
    var top = el.getBoundingClientRect().top - scroller.getBoundingClientRect().top + scroller.scrollTop;
    scroller.scrollTop = Math.max(0, top - 12);
    paintScroll();
  };

  window.updateTypography = function(cssText, colorScheme) {
    var style = document.getElementById('dynamic-typography');
    if (!style) {
      style = document.createElement('style');
      style.id = 'dynamic-typography';
      document.head.appendChild(style);
    }
    style.textContent = cssText;
    if (colorScheme) {
      var meta = document.querySelector('meta[name="color-scheme"]');
      if (!meta) {
        meta = document.createElement('meta');
        meta.name = 'color-scheme';
        document.head.appendChild(meta);
      }
      meta.content = colorScheme;
    }
  };

  // Mirrors the native chrome state into the document so the veils can retire with
  // the system bars they exist to protect the prose from.
  window.setReaderImmersive = function(on) {
    document.body.classList.toggle('immersive', !!on);
  };

  window.toggleReaderSummary = function() {
    var sum = document.querySelector('.sum');
    var handle = document.querySelector('.sum-toggle');
    if (sum) sum.classList.toggle('open');
    if (handle) {
      handle.classList.toggle('open');
      handle.setAttribute('aria-expanded', handle.classList.contains('open') ? 'true' : 'false');
    }
  };

  window.readerSummaryAction = function(action) {
    if (action === 'hide') {
      var sum = document.querySelector('.sum');
      var handle = document.querySelector('.sum-toggle');
      if (sum) sum.classList.remove('open');
      if (handle) {
        handle.classList.remove('open');
        handle.setAttribute('aria-expanded', 'false');
      }
    }
    try { NativeBridge.onSummaryAction(action); } catch(e) {}
  };

  // Best-effort sentence wrapping for the Listen player's spoken-sentence
  // highlight. Runs once, lazily (on first setSpeaking), and skips paragraphs
  // that already hold highlights/links/media so it never clobbers them.
  window.segmentSentences = function() {
    var body = document.getElementById('article-body');
    if (!body || body.getAttribute('data-segmented') === '1') return;
    var paras = body.querySelectorAll('p');
    var idx = 0;
    for (var p = 0; p < paras.length; p++) {
      var para = paras[p];
      if (para.querySelector('mark, img, a, .say')) continue;
      var text = para.textContent;
      if (!text || !text.trim()) continue;
      var parts = text.match(/[^.!?]+[.!?]*\s*/g);
      if (!parts) continue;
      para.textContent = '';
      for (var s = 0; s < parts.length; s++) {
        var span = document.createElement('span');
        span.className = 'say';
        span.setAttribute('data-s', idx);
        span.textContent = parts[s];
        para.appendChild(span);
        idx++;
      }
    }
    body.setAttribute('data-segmented', '1');
  };

  window.setSpeaking = function(index) {
    if (index < 0) {
      document.querySelectorAll('.say.speaking').forEach(function(s) {
        s.classList.remove('speaking');
      });
      return;
    }
    window.segmentSentences();
    document.querySelectorAll('.say.speaking').forEach(function(s) {
      s.classList.remove('speaking');
    });
    var active = document.querySelector('.say[data-s="' + index + '"]');
    if (active) active.classList.add('speaking');
  };

  var TAG_COLORS = {
    yellow: '#c89b00', blue: '#0a84ff', green: '#34c759',
    pink: '#ff2d55', purple: '#af52de'
  };

  window.applyHighlights = function(highlights) {
    var existing = document.querySelectorAll('mark[data-highlight-id]');
    existing.forEach(function(m) {
      var parent = m.parentNode;
      while (m.firstChild) parent.insertBefore(m.firstChild, m);
      parent.removeChild(m);
    });
    document.querySelectorAll('.hl-tag-indicator').forEach(function(el) { el.remove(); });
    var body = document.getElementById('article-body');
    if (!body || !highlights || highlights.length === 0) return;
    highlights.sort(function(a, b) { return b.start - a.start; });
    var tw = document.createTreeWalker(body, NodeFilter.SHOW_TEXT, null);
    var nodes = [];
    var node;
    while ((node = tw.nextNode())) nodes.push(node);
    var charIndex = 0;
    var nodeMap = [];
    for (var i = 0; i < nodes.length; i++) {
      var n = nodes[i];
      nodeMap.push({node: n, start: charIndex, end: charIndex + n.length});
      charIndex += n.length;
    }
    for (var h = 0; h < highlights.length; h++) {
      var hl = highlights[h];
      wrapRange(body, nodeMap, hl.start, hl.end, hl.id, hl.color);
    }
    for (var h = 0; h < highlights.length; h++) {
      var hl = highlights[h];
      if (!hl.tags || hl.tags.length === 0) continue;
      var marks = body.querySelectorAll('mark[data-highlight-id="' + hl.id + '"]');
      if (marks.length === 0) continue;
      var lastMark = marks[marks.length - 1];
      var indicator = document.createElement('span');
      indicator.className = 'hl-tag-indicator';
      var dot = document.createElement('span');
      dot.className = 'hl-tag-dot';
      dot.style.background = TAG_COLORS[hl.color] || '#888';
      var label = document.createElement('span');
      label.className = 'hl-tag-label';
      label.textContent = hl.tags[0] + (hl.tags.length > 1 ? ' +' + (hl.tags.length - 1) : '');
      indicator.appendChild(dot);
      indicator.appendChild(label);
      lastMark.parentNode.insertBefore(indicator, lastMark.nextSibling);
    }
  };

  function wrapRange(body, nodeMap, start, end, id, color) {
    for (var i = 0; i < nodeMap.length; i++) {
      var nm = nodeMap[i];
      if (nm.end <= start || nm.start >= end) continue;
      var node = nm.node;
      var relStart = Math.max(0, start - nm.start);
      var relEnd = Math.min(node.length, end - nm.start);
      if (relStart === 0 && relEnd === node.length) {
        var mark = document.createElement('mark');
        mark.setAttribute('data-highlight-id', id);
        mark.className = 'hl-' + color;
        node.parentNode.insertBefore(mark, node);
        mark.appendChild(node);
      } else {
        var range = document.createRange();
        range.setStart(node, relStart);
        range.setEnd(node, relEnd);
        var mark = document.createElement('mark');
        mark.setAttribute('data-highlight-id', id);
        mark.className = 'hl-' + color;
        range.surroundContents(mark);
      }
    }
  }
})();
    """.trimIndent()

/**
 * Transcript timestamps. YouTube reader bodies carry a `.t-seg[data-t]` span per
 * transcript segment; the web reader reveals the timestamp on hover, which does not
 * exist on touch, so mobile reveals it on tap and keeps one open at a time.
 */
internal val TRANSCRIPT_TAP_JS: String =
    """
(function() {
  document.addEventListener('click', function(e) {
    var seg = e.target && e.target.closest ? e.target.closest('.t-seg') : null;
    var open = document.querySelector('.t-seg.t-open');
    if (open && open !== seg) open.classList.remove('t-open');
    if (seg) seg.classList.toggle('t-open');
  }, true);
})();
    """.trimIndent()
