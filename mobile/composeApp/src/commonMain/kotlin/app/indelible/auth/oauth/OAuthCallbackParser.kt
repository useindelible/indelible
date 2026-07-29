package app.indelible.auth.oauth

fun parseOAuthCallback(url: String): OAuthCallbackResult? {
    val marker = "oauth/callback"
    if (!url.startsWith("com.useindelible.app:/") || !url.contains(marker)) {
        return null
    }
    val query = url.substringAfter('?', missingDelimiterValue = "")
    val params =
        query
            .split('&')
            .filter { it.isNotBlank() }
            .mapNotNull {
                val key = it.substringBefore('=')
                val value = it.substringAfter('=', missingDelimiterValue = "")
                if (key.isBlank()) null else key to percentDecode(value)
            }.toMap()

    return OAuthCallbackResult(
        code = params["code"],
        state = params["state"],
        error = params["error"] ?: params["error_code"],
        errorDescription = params["error_description"],
    )
}

private const val HEX_RADIX = 16
private const val PERCENT_ESCAPE_LENGTH = 3 // '%' plus two hex digits

private fun percentDecode(value: String): String {
    val bytes = mutableListOf<Byte>()
    var i = 0
    while (i < value.length) {
        when (val c = value[i]) {
            '%' -> {
                if (i + 2 < value.length) {
                    bytes += value.substring(i + 1, i + PERCENT_ESCAPE_LENGTH).toInt(HEX_RADIX).toByte()
                    i += PERCENT_ESCAPE_LENGTH
                } else {
                    bytes += c.code.toByte()
                    i++
                }
            }
            '+' -> {
                bytes += ' '.code.toByte()
                i++
            }
            else -> {
                bytes += c.code.toByte()
                i++
            }
        }
    }
    return bytes.toByteArray().decodeToString()
}
