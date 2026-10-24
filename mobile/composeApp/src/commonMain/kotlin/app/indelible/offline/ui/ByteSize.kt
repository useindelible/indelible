package app.indelible.offline.ui

import androidx.compose.runtime.Composable
import app.indelible.core.i18n.LocaleFormatters
import indelible.composeapp.generated.resources.Res
import indelible.composeapp.generated.resources.offline_size_gb
import indelible.composeapp.generated.resources.offline_size_kb
import indelible.composeapp.generated.resources.offline_size_mb
import org.jetbrains.compose.resources.stringResource

private const val KIB = 1024L
private const val MIB = KIB * 1024
private const val GIB = MIB * 1024

enum class SizeUnit { KB, MB, GB }

data class DisplaySize(
    val value: Double,
    val unit: SizeUnit,
)

/** The largest unit that holds at least one whole unit; kilobytes round up so a small file never reads as zero. */
fun displaySize(bytes: Long): DisplaySize =
    when {
        bytes >= GIB -> DisplaySize(bytes.toDouble() / GIB, SizeUnit.GB)
        bytes >= MIB -> DisplaySize(bytes.toDouble() / MIB, SizeUnit.MB)
        else -> DisplaySize(((bytes + KIB - 1) / KIB).toDouble(), SizeUnit.KB)
    }

@Composable
fun byteSize(bytes: Long): String {
    val size = displaySize(bytes)
    val value = LocaleFormatters.decimal(size.value)
    return when (size.unit) {
        SizeUnit.KB -> stringResource(Res.string.offline_size_kb, value)
        SizeUnit.MB -> stringResource(Res.string.offline_size_mb, value)
        SizeUnit.GB -> stringResource(Res.string.offline_size_gb, value)
    }
}
