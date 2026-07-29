package app.indelible.reader.ui

// Fractal-noise tile; a data URI rather than a bundled asset so it costs no resource read.
@Suppress("MaxLineLength")
private const val GRAIN_URI =
    """url("data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' width='140' height='140'%3E%3Cfilter id='n'%3E%3CfeTurbulence type='fractalNoise' baseFrequency='0.82' numOctaves='3' stitchTiles='stitch'/%3E%3C/filter%3E%3Crect width='140' height='140' filter='url(%23n)'/%3E%3C/svg%3E")"""

/**
 * The masthead field and the layers that seat chrome over it.
 *
 * The drawing sits in a non-scrolling layer and travels out of frame at half the
 * content's speed, capped at its own height, so it leaves the way a photo leaves
 * the top of a page. Both veils and the travel are driven from `--y` (pixels
 * scrolled) and `--p` (fraction read), which the scroll listener writes on `body`.
 * A CSS scroll timeline would be more elegant and is silently inert wherever
 * scroll-driven animations are unsupported, which is why this is script-driven.
 *
 * The drawings are lit for paper. On a dark canvas they are dimmed and their
 * chroma pushed back up so they read as the same scene at dusk rather than a
 * lightbox pasted onto black. Light canvases emit no filter at all: the authored
 * values are a visual no-op there, and a live filter forces a composite pass over
 * a subtree carrying nine Gaussian blurs.
 */
internal fun buildReaderChromeCss(isDarkBg: Boolean): String {
    val artFilter =
        if (isDarkBg) {
            "  filter: saturate(1.35) brightness(0.52) contrast(1.12);\n  opacity: 0.92;"
        } else {
            "  filter: none;\n  opacity: 1;"
        }
    val chromeScrim = if (isDarkBg) "rgba(0,0,0,0.46)" else "rgba(255,255,255,0.44)"
    val grainOpacity = if (isDarkBg) "0.055" else "0.030"
    val grainBlend = if (isDarkBg) "overlay" else "multiply"
    return """
.aura {
  position: absolute; top: 0; left: 0; right: 0; z-index: 0;
  pointer-events: none; overflow: hidden; contain: paint;
  transform: translateY(calc(-1 * min(calc(var(--y) * 0.5px), var(--aura-travel, 0px))));
}
.aura-art {
  position: relative; display: block; height: auto; width: 100%;
$artFilter
}
/* The melt travels with the drawing and ends on the exact page colour, so the
   field stops without a seam wherever it happens to be. */
.aura::after {
  content: ''; position: absolute; inset: 0;
  background: linear-gradient(to bottom, transparent 28%, var(--bg-color) 100%);
}
/* Stationary, unlike the field it covers: a scrim attached to the artwork would
   slide out from under the chrome it exists to protect. */
.chrome-veil {
  position: absolute; top: 0; left: 0; right: 0; height: 110px; z-index: 1;
  pointer-events: none;
  background: linear-gradient(to bottom, $chromeScrim 0px, $chromeScrim 56px, transparent 110px);
  opacity: calc(1 - clamp(0, calc(var(--y) / var(--veil-range)), 1));
}
.grain {
  position: absolute; inset: 0; z-index: 70; pointer-events: none;
  background-image: $GRAIN_URI;
  opacity: $grainOpacity; mix-blend-mode: $grainBlend;
}
/* The scrim exists for the floating controls, so it leaves when they do. Left up on
   its own it reads as a band of page colour across the top and the article visibly
   stops short of the edge instead of running under the camera. */
.chrome-veil {
  transition: transform .28s cubic-bezier(.32,.72,0,1), opacity .28s cubic-bezier(.32,.72,0,1);
}
body.immersive .chrome-veil {
  transform: translateY(-100%);
}
    """.trimIndent()
}
