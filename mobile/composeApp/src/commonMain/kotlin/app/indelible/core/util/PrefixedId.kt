package app.indelible.core.util

import kotlinx.datetime.Clock
import kotlin.random.Random

private const val VERSION_BYTE_INDEX = 6
private const val VARIANT_BYTE_INDEX = 8
private const val UUID_BYTE_COUNT = 16
private const val TIMESTAMP_BYTE_COUNT = 6
private const val BITS_PER_BYTE = 8
private const val NIBBLE_BITS = 4
private const val BYTE_MASK = 0xFF
private const val VERSION_NIBBLE_MASK = 0x0F
private const val LOW_NIBBLE_MASK = 0x0F
private const val VERSION_NIBBLE_VALUE = 0x70
private const val VARIANT_NIBBLE_MASK = 0x3F
private const val VARIANT_NIBBLE_VALUE = 0x80
private const val UUID_GROUP_1_END = 8
private const val UUID_GROUP_2_END = 12
private const val UUID_GROUP_3_END = 16
private const val UUID_GROUP_4_END = 20
private const val UUID_GROUP_5_END = 32
private val HEX_DIGITS = "0123456789abcdef".toCharArray()

/**
 * RFC 9562 UUIDv7: a 48-bit big-endian millisecond timestamp, the version/variant nibbles, and
 * random fill for the rest. The timestamp prefix keeps generated ids sortable by creation time.
 */
fun uuidV7(): String {
    val millis = Clock.System.now().toEpochMilliseconds()
    val bytes = ByteArray(UUID_BYTE_COUNT)
    for (i in 0 until TIMESTAMP_BYTE_COUNT) {
        bytes[i] = (millis shr ((TIMESTAMP_BYTE_COUNT - 1 - i) * BITS_PER_BYTE)).toByte()
    }
    Random.nextBytes(bytes, TIMESTAMP_BYTE_COUNT, UUID_BYTE_COUNT)
    bytes[VERSION_BYTE_INDEX] =
        ((bytes[VERSION_BYTE_INDEX].toInt() and VERSION_NIBBLE_MASK) or VERSION_NIBBLE_VALUE).toByte()
    bytes[VARIANT_BYTE_INDEX] =
        ((bytes[VARIANT_BYTE_INDEX].toInt() and VARIANT_NIBBLE_MASK) or VARIANT_NIBBLE_VALUE).toByte()
    return bytes.toUuidString()
}

fun readingEventId(): String = "rev_${uuidV7()}"

fun highlightClientId(): String = "hlt_${uuidV7()}"

fun clientId(): String = "cli_${uuidV7()}"

private fun ByteArray.toUuidString(): String {
    val hex = CharArray(size * 2)
    for (i in indices) {
        val value = this[i].toInt() and BYTE_MASK
        hex[i * 2] = HEX_DIGITS[value ushr NIBBLE_BITS]
        hex[i * 2 + 1] = HEX_DIGITS[value and LOW_NIBBLE_MASK]
    }
    val digits = hex.concatToString()
    return buildString {
        append(digits, 0, UUID_GROUP_1_END)
        append('-')
        append(digits, UUID_GROUP_1_END, UUID_GROUP_2_END)
        append('-')
        append(digits, UUID_GROUP_2_END, UUID_GROUP_3_END)
        append('-')
        append(digits, UUID_GROUP_3_END, UUID_GROUP_4_END)
        append('-')
        append(digits, UUID_GROUP_4_END, UUID_GROUP_5_END)
    }
}
