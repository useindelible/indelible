package app.indelible.share

import kotlinx.cinterop.ExperimentalForeignApi
import kotlinx.cinterop.alloc
import kotlinx.cinterop.memScoped
import kotlinx.cinterop.ptr
import platform.Foundation.NSURL
import platform.posix.gettimeofday
import platform.posix.timeval

actual fun isValidUrl(text: String): Boolean {
    val url = NSURL.URLWithString(text) ?: return false
    return url.scheme in listOf("http", "https")
}

internal actual fun isNetworkException(throwable: Throwable): Boolean =
    throwable.message?.contains("NSURLErrorDomain") == true ||
        throwable.message?.contains("Could not connect") == true ||
        throwable.message?.contains("The network connection was lost") == true

private const val MS_PER_SECOND = 1000L
private const val US_PER_MS = 1000L

@OptIn(ExperimentalForeignApi::class)
internal actual fun currentEpochMillis(): Long =
    memScoped {
        val tv = alloc<timeval>()
        gettimeofday(tv.ptr, null)
        tv.tv_sec * MS_PER_SECOND + tv.tv_usec.toLong() / US_PER_MS
    }
