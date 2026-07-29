package app.indelible.core.image

import app.indelible.core.network.AuthenticatedApiTransport
import coil3.intercept.Interceptor
import coil3.network.NetworkHeaders
import coil3.network.httpHeaders
import coil3.request.ImageResult

/**
 * Routes backend-served image URLs (`/api/v1/...`) through [AuthenticatedApiTransport] so they get
 * the same origin rewrite and Bearer token the rest of the app uses; external and
 * presigned URLs pass through untouched. Installing this on the singleton
 * ImageLoader means every AsyncImage (library rows, home, reader, avatars) loads
 * protected assets correctly while keeping Coil's memory/disk caching, instead of
 * each surface re-fetching bytes by hand.
 */
internal class AuthAssetInterceptor(
    private val transport: AuthenticatedApiTransport,
) : Interceptor {
    override suspend fun intercept(chain: Interceptor.Chain): ImageResult {
        val data = chain.request.data
        if (data !is String) return chain.proceed()

        val resolved = transport.resolveImageRequest(data)
        if (resolved.url == data && resolved.bearerToken == null) {
            return chain.proceed()
        }

        val builder = chain.request.newBuilder().data(resolved.url)
        resolved.bearerToken?.let { token ->
            builder.httpHeaders(
                NetworkHeaders.Builder().set("Authorization", "Bearer $token").build(),
            )
        }
        return chain.withRequest(builder.build()).proceed()
    }
}
