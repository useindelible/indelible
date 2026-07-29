package app.indelible.core.network

import app.indelible.core.config.ServerBuildConfig
import app.indelible.core.storage.TokenStorage

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
        ?.let(::canonicalServerOrigin)
        ?.takeIf { it.isNotEmpty() }
        ?: canonicalServerOrigin(bakedDefaultUrl).ifEmpty { AuthenticatedApiTransport.DEFAULT_SERVER_URL }

internal fun canonicalServerOrigin(url: String): String = url.trim().trimEnd('/')
