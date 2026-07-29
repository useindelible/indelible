package app.indelible.core.network

/**
 * A remote image URL prepared for loading: [url] is reachable from the app (origin
 * rewritten when needed) and [bearerToken] is the token to attach, or null for
 * external/presigned URLs that authenticate themselves.
 */
data class ResolvedImageRequest(
    val url: String,
    val bearerToken: String?,
)
