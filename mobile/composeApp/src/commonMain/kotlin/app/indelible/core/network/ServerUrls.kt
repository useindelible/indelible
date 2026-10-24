package app.indelible.core.network

import app.indelible.core.config.ServerBuildConfig
import app.indelible.core.storage.TokenStorage
import io.ktor.http.parseUrl

/**
 * Resolution order for the active server: what the user connected to, then the
 * URL baked into the build (Cloud flavor), then the local-dev fallback.
 */
suspend fun TokenStorage.resolvedServerUrl(): String = resolveServerUrl(getServerUrl())

internal fun resolveServerUrl(
    storedUrl: String?,
    bakedDefaultUrl: String = ServerBuildConfig.SERVER_URL_DEFAULT,
): String =
    storedUrl
        ?.let(::normalizedOrigin)
        ?.takeIf { it.isNotEmpty() }
        ?: normalizedOrigin(bakedDefaultUrl).ifEmpty { AuthenticatedApiTransport.DEFAULT_SERVER_URL }

/**
 * Parses with Ktor so equivalent servers (case, default port, trailing slash) collapse to the
 * same scope key. Falls back to trim-only normalization for input Ktor cannot parse as a URL,
 * since this also feeds request-URL resolution and must never throw.
 */
internal fun normalizedOrigin(url: String): String {
    val trimmed = url.trim()
    val parsed = parseUrl(trimmed) ?: return trimmed.trimEnd('/')
    val portSuffix = if (parsed.port == parsed.protocol.defaultPort) "" else ":${parsed.port}"
    val path = parsed.encodedPath.trimEnd('/')
    return "${parsed.protocol.name}://${parsed.host.lowercase()}$portSuffix$path"
}
