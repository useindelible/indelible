package app.indelible.auth.oauth

import kotlinx.datetime.Clock

expect fun secureRandomBytes(size: Int): ByteArray

expect fun sha256(bytes: ByteArray): ByteArray

private const val CODE_VERIFIER_BYTES = 32
private const val APP_STATE_BYTES = 24
private const val PENDING_FLOW_TTL_SECONDS = 600L

fun generateCodeVerifier(): String = base64UrlNoPadding(secureRandomBytes(CODE_VERIFIER_BYTES))

fun generateAppState(): String = base64UrlNoPadding(secureRandomBytes(APP_STATE_BYTES))

fun codeChallenge(verifier: String): String = base64UrlNoPadding(sha256(verifier.encodeToByteArray()))

fun pendingFlowExpiry(): Long = Clock.System.now().epochSeconds + PENDING_FLOW_TTL_SECONDS

fun isExpired(flow: PendingOAuthFlow): Boolean = flow.expiresAtEpochSeconds <= Clock.System.now().epochSeconds

private val base64UrlAlphabet = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_"

// Canonical RFC 4648 base64url-without-padding encoder; the masks (0xff/0x03/0x0f/0x3f) and
// shifts are the standard base64 6-bit grouping, clearer as the well-known idiom than as names.
@Suppress("MagicNumber")
fun base64UrlNoPadding(bytes: ByteArray): String {
    val output = StringBuilder((bytes.size * 4 + 2) / 3)
    var index = 0
    while (index < bytes.size) {
        val b0 = bytes[index++].toInt() and 0xff
        val b1 = if (index < bytes.size) bytes[index++].toInt() and 0xff else -1
        val b2 = if (index < bytes.size) bytes[index++].toInt() and 0xff else -1

        output.append(base64UrlAlphabet[b0 shr 2])
        if (b1 == -1) {
            output.append(base64UrlAlphabet[(b0 and 0x03) shl 4])
        } else {
            output.append(base64UrlAlphabet[((b0 and 0x03) shl 4) or (b1 shr 4)])
            if (b2 == -1) {
                output.append(base64UrlAlphabet[(b1 and 0x0f) shl 2])
            } else {
                output.append(base64UrlAlphabet[((b1 and 0x0f) shl 2) or (b2 shr 6)])
                output.append(base64UrlAlphabet[b2 and 0x3f])
            }
        }
    }
    return output.toString()
}
