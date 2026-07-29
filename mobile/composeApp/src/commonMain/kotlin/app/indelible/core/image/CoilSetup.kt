package app.indelible.core.image

import app.indelible.core.network.AuthenticatedApiTransport
import coil3.ImageLoader
import coil3.PlatformContext
import coil3.network.ktor3.KtorNetworkFetcherFactory
import coil3.request.crossfade

/**
 * Builds the app-wide Coil image loader.
 *
 * [AuthAssetInterceptor] gives backend-served image URLs the origin rewrite and
 * Bearer token the API uses, so protected assets (thumbnails, lead images, avatars)
 * load instead of silently 401-ing or pointing at an unreachable origin.
 *
 * Registering [KtorNetworkFetcherFactory] explicitly is required on Kotlin/Native
 * (iOS): unlike JVM/Android, Coil does not auto-discover network fetchers there via
 * ServiceLoader, so without it every http(s) image URL fails to load.
 */
fun newImageLoader(
    context: PlatformContext,
    transport: AuthenticatedApiTransport,
): ImageLoader =
    ImageLoader
        .Builder(context)
        .components {
            add(AuthAssetInterceptor(transport))
            add(KtorNetworkFetcherFactory())
        }.crossfade(true)
        .build()
