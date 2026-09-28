package app.indelible.reader.ui

internal const val READER_BOOK_CSS = ".rscroll.hold { visibility: hidden; }"

/**
 * Book reading on top of [READER_BRIDGE_JS]: complete samples (`offset` is the first visible
 * character of `#article-body`, counted like highlight offsets), restore by character, and in-book links.
 */
internal val READER_BOOK_JS: String =
    """
(function() {
  var scroller = document.querySelector('.rscroll') || document.scrollingElement;
  var body = document.getElementById('article-body');
  var settleTimer = null;

  function reveal() { scroller.classList.remove('hold'); }
  if (scroller.classList.contains('hold')) setTimeout(reveal, 1500);

  function textNodes() {
    return document.createTreeWalker(body, NodeFilter.SHOW_TEXT, {
      acceptNode: function(node) {
        return node.parentNode.closest('.hl-tag-indicator') ? NodeFilter.FILTER_REJECT : NodeFilter.FILTER_ACCEPT;
      }
    });
  }

  // The caret probe can miss between paragraphs; the text node crossing the top edge answers then,
  // at the share of it that lies above the edge.
  function firstTextBelow(top) {
    var walker = textNodes();
    var count = 0;
    var node;
    while ((node = walker.nextNode())) {
      var range = document.createRange();
      range.selectNodeContents(node);
      var box = range.getBoundingClientRect();
      if (box.height > 0 && box.bottom > top) {
        var above = Math.max(0, Math.min(1, (top - box.top) / box.height));
        return count + Math.round(node.length * above);
      }
      count += node.length;
    }
    return count;
  }

  function firstVisibleOffset() {
    if (!body) return 0;
    var top = scroller.getBoundingClientRect().top + 1;
    var box = body.getBoundingClientRect();
    if (box.top >= top) return 0;
    var caret = document.caretRangeFromPoint
      ? document.caretRangeFromPoint(box.left + box.width / 2, top)
      : null;
    if (caret && body.contains(caret.startContainer)) {
      return window.readerTextOffset(caret.startContainer, caret.startOffset);
    }
    return firstTextBelow(top);
  }

  function sample(complete) {
    var max = scroller.scrollHeight - scroller.clientHeight;
    return {
      percent: max > 0 ? Math.min(100, Math.max(0, scroller.scrollTop / max * 100)) : 0,
      y: scroller.scrollTop,
      offset: complete ? firstVisibleOffset() : -1,
      atEnd: scroller.scrollTop + scroller.clientHeight >= scroller.scrollHeight - 2
    };
  }

  function post(s) {
    try { NativeBridge.onScroll(s.percent, s.y, s.offset, s.atEnd); } catch (e) {}
  }

  scroller.addEventListener('scroll', function() {
    clearTimeout(settleTimer);
    settleTimer = setTimeout(function() { post(sample(true)); }, 250);
  }, { passive: true });

  window.captureSample = function() {
    clearTimeout(settleTimer);
    post(sample(true));
  };

  function settled(then) {
    var done = false;
    function finish() { if (!done) { done = true; then(); } }
    var waits = [];
    if (document.fonts && document.fonts.ready) waits.push(document.fonts.ready);
    (body ? Array.prototype.slice.call(body.querySelectorAll('img')) : []).forEach(function(img) {
      if (img.decode) waits.push(img.decode().catch(function() {}));
    });
    Promise.all(waits).then(finish, finish);
    setTimeout(finish, 1200);
  }

  function positioned() {
    reveal();
    try { NativeBridge.onPositioned(); } catch (e) {}
  }

  // Collapsed whitespace has no box, so the nearest glyph in the node answers: forward, then back.
  function boxOf(node, at) {
    var text = node.data;
    var i = at;
    while (i < text.length && !/\S/.test(text.charAt(i))) i++;
    if (i >= text.length) {
      i = Math.min(at, text.length) - 1;
      while (i >= 0 && !/\S/.test(text.charAt(i))) i--;
    }
    if (i < 0) return null;
    var range = document.createRange();
    range.setStart(node, i);
    range.setEnd(node, i + 1);
    return range.getClientRects()[0] || null;
  }

  // Unrendered whitespace between blocks is skipped, and past the last character the chapter's end
  // answers, so a stale offset still opens the chapter.
  function boxAt(offset) {
    var walker = textNodes();
    var seen = 0;
    var last = null;
    var node;
    while ((node = walker.nextNode())) {
      if (seen + node.length > offset) {
        var box = boxOf(node, Math.max(0, offset - seen));
        if (box) return box;
      }
      if (/\S/.test(node.data)) last = node;
      seen += node.length;
    }
    return last ? boxOf(last, last.length) : null;
  }

  function scrollToChar(offset) {
    var box = body && offset > 0 ? boxAt(offset) : null;
    scroller.scrollTop = box ? Math.max(0, scroller.scrollTop + box.top - scroller.getBoundingClientRect().top) : 0;
    positioned();
  }

  window.scrollToCharOffset = function(offset) {
    if (offset > 0) settled(function() { scrollToChar(offset); }); else scrollToChar(0);
  };

  var scrollToAnchor = window.scrollToAnchor;
  window.scrollToAnchor = function(id, fallbackIndex) {
    function run() {
      scrollToAnchor(id, fallbackIndex);
      positioned();
    }
    if (scroller.classList.contains('hold')) settled(run); else run();
  };

  document.addEventListener('click', function(e) {
    var link = e.target && e.target.closest ? e.target.closest('#article-body a[href]') : null;
    if (!link) return;
    var href = link.getAttribute('href');
    if (/^(https?|mailto):/i.test(href)) return;
    e.preventDefault();
    try { NativeBridge.onBookLink(href); } catch (err) {}
  });
})();
    """.trimIndent()
