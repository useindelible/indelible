package app.indelible.collections.ui

import app.indelible.ui.theme.paletteBucketIndex

// Maps a collection's colour name (or id fallback) to an index into
// IndelibleTheme.colors.collectionBanners, sharing the palette-slot logic with tag dots.
internal fun collectionBannerIndex(
    color: String?,
    id: String,
): Int = paletteBucketIndex(color, id)
