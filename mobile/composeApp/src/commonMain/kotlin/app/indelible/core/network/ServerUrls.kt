package app.indelible.core.network

import app.indelible.core.config.ServerBuildConfig
import app.indelible.core.storage.TokenStorage

/**
 * Resolution order for the active server: what the user connected to, then the
 * URL baked into the build (Cloud flavor), then the local-dev fallback.
 */
suspend fun TokenStorage.resolvedServerUrl(): String =
    getServerUrl()
        ?: ServerBuildConfig.SERVER_URL_DEFAULT.trim().ifEmpty { null }
        ?: AuthenticatedApiTransport.DEFAULT_SERVER_URL
