package app.indelible.share

actual fun isValidUrl(text: String): Boolean =
    try {
        val url = java.net.URL(text)
        url.protocol in listOf("http", "https")
    } catch (_: Exception) {
        false
    }

internal actual fun isNetworkException(throwable: Throwable): Boolean =
    throwable is java.net.UnknownHostException ||
        throwable is java.net.SocketTimeoutException ||
        throwable is java.net.ConnectException

internal actual fun currentEpochMillis(): Long = System.currentTimeMillis()
