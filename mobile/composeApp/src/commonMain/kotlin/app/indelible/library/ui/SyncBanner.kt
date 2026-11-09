package app.indelible.library.ui

import androidx.compose.animation.AnimatedVisibility
import androidx.compose.animation.animateContentSize
import androidx.compose.animation.expandVertically
import androidx.compose.animation.fadeIn
import androidx.compose.animation.fadeOut
import androidx.compose.animation.shrinkVertically
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.material3.LinearProgressIndicator
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.semantics.LiveRegionMode
import androidx.compose.ui.semantics.liveRegion
import androidx.compose.ui.semantics.semantics
import androidx.compose.ui.tooling.preview.Preview
import app.indelible.home.viewmodel.ShelfNet
import app.indelible.home.viewmodel.SyncProgress
import app.indelible.ui.components.IndelibleButton
import app.indelible.ui.components.IndelibleButtonStyle
import app.indelible.ui.theme.AppTheme
import app.indelible.ui.theme.IndelibleShape
import app.indelible.ui.theme.IndelibleSpacing
import app.indelible.ui.theme.IndelibleTheme
import indelible.composeapp.generated.resources.Res
import indelible.composeapp.generated.resources.library_sync_banner_all_synced
import indelible.composeapp.generated.resources.library_sync_banner_attention
import indelible.composeapp.generated.resources.library_sync_banner_key
import indelible.composeapp.generated.resources.library_sync_banner_retry_now
import indelible.composeapp.generated.resources.library_sync_banner_review
import indelible.composeapp.generated.resources.library_sync_banner_sent
import indelible.composeapp.generated.resources.library_sync_banner_synced
import indelible.composeapp.generated.resources.library_sync_banner_syncing
import indelible.composeapp.generated.resources.library_sync_banner_to_go
import indelible.composeapp.generated.resources.library_sync_banner_unreachable
import indelible.composeapp.generated.resources.library_sync_banner_unreachable_body
import indelible.composeapp.generated.resources.library_sync_banner_unreachable_queued
import org.jetbrains.compose.resources.pluralStringResource
import org.jetbrains.compose.resources.stringResource

/** What the Library banner says about the catch-up; null means no banner. */
sealed interface SyncBanner {
    data class Syncing(
        val sent: Int,
        val total: Int,
    ) : SyncBanner

    data class Attention(
        val synced: Int,
        val needAttention: Int,
    ) : SyncBanner

    data object AllSynced : SyncBanner

    data class Unreachable(
        val host: String,
        val queued: Int,
    ) : SyncBanner
}

// An unanswering server comes first: its fix differs from being offline, and offline shows no banner.
fun syncBannerOf(
    progress: SyncProgress?,
    host: String?,
): SyncBanner? {
    progress ?: return null
    return when {
        progress.unreachable -> host?.let { SyncBanner.Unreachable(it, progress.queued) }
        progress.net == ShelfNet.SENDING -> SyncBanner.Syncing(progress.sent, progress.sendTotal)
        progress.net == ShelfNet.SYNCED -> SyncBanner.AllSynced
        progress.net == ShelfNet.ONLINE && progress.attention > 0 ->
            SyncBanner.Attention(progress.lastSent, progress.attention)
        else -> null
    }
}

@Composable
fun SyncBannerBar(
    banner: SyncBanner?,
    onReview: () -> Unit,
    onRetryNow: () -> Unit,
    modifier: Modifier = Modifier,
) {
    var shown by remember { mutableStateOf(banner) }
    if (banner != null) shown = banner
    AnimatedVisibility(
        visible = banner != null,
        enter = expandVertically() + fadeIn(),
        exit = shrinkVertically() + fadeOut(),
        modifier = modifier,
    ) {
        shown?.let { BannerContent(it, onReview, onRetryNow) }
    }
}

private data class Tone(
    val fg: Color,
    val bg: Color,
)

private const val WASH_ALPHA = 0.12f
private const val TRACK_ALPHA = 0.2f

@Composable
private fun toneOf(banner: SyncBanner): Tone =
    when (banner) {
        is SyncBanner.Syncing, SyncBanner.AllSynced ->
            IndelibleTheme.colors.success.let { Tone(it, it.copy(alpha = WASH_ALPHA)) }
        is SyncBanner.Attention -> IndelibleTheme.colors.warning.let { Tone(it, it.copy(alpha = WASH_ALPHA)) }
        is SyncBanner.Unreachable -> MaterialTheme.colorScheme.error.let { Tone(it, it.copy(alpha = WASH_ALPHA)) }
    }

@Composable
private fun BannerContent(
    banner: SyncBanner,
    onReview: () -> Unit,
    onRetryNow: () -> Unit,
) {
    val tone = toneOf(banner)
    Surface(color = tone.bg, contentColor = tone.fg, modifier = Modifier.fillMaxWidth()) {
        Column(
            modifier =
                Modifier
                    .animateContentSize()
                    .padding(horizontal = IndelibleSpacing.step16, vertical = IndelibleSpacing.step10),
            verticalArrangement = Arrangement.spacedBy(IndelibleSpacing.step8),
        ) {
            Row(
                horizontalArrangement = Arrangement.spacedBy(IndelibleSpacing.step8),
                verticalAlignment = Alignment.CenterVertically,
            ) {
                Box(Modifier.size(IndelibleSpacing.step8).clip(IndelibleShape.full).background(tone.fg))
                Text(
                    text = headline(banner),
                    style = MaterialTheme.typography.titleSmall,
                    modifier = Modifier.weight(1f).semantics { liveRegion = LiveRegionMode.Polite },
                )
                BannerAction(banner, onReview, onRetryNow)
            }
            when (banner) {
                is SyncBanner.Syncing -> SyncingDetail(banner, tone)
                is SyncBanner.Unreachable ->
                    Text(
                        text = unreachableBody(banner.queued),
                        style = MaterialTheme.typography.bodySmall,
                        color = MaterialTheme.colorScheme.onSurfaceVariant,
                    )
                else -> Unit
            }
        }
    }
}

@Composable
private fun unreachableBody(queued: Int): String =
    if (queued == 0) {
        stringResource(Res.string.library_sync_banner_unreachable_body)
    } else {
        pluralStringResource(Res.plurals.library_sync_banner_unreachable_queued, queued, queued)
    }

@Composable
private fun headline(banner: SyncBanner): String =
    when (banner) {
        is SyncBanner.Syncing ->
            stringResource(
                Res.string.library_sync_banner_syncing,
                (banner.sent + 1).coerceAtMost(banner.total),
                banner.total,
            )
        is SyncBanner.Attention -> attentionHeadline(banner)
        SyncBanner.AllSynced -> stringResource(Res.string.library_sync_banner_all_synced)
        is SyncBanner.Unreachable -> stringResource(Res.string.library_sync_banner_unreachable, banner.host)
    }

@Composable
private fun attentionHeadline(banner: SyncBanner.Attention): String {
    val attention =
        pluralStringResource(Res.plurals.library_sync_banner_attention, banner.needAttention, banner.needAttention)
    if (banner.synced == 0) return attention
    val synced = pluralStringResource(Res.plurals.library_sync_banner_synced, banner.synced, banner.synced)
    return stringResource(Res.string.library_sync_banner_key, synced, attention)
}

@Composable
private fun BannerAction(
    banner: SyncBanner,
    onReview: () -> Unit,
    onRetryNow: () -> Unit,
) {
    when (banner) {
        is SyncBanner.Attention ->
            IndelibleButton(
                text = stringResource(Res.string.library_sync_banner_review),
                onClick = onReview,
                style = IndelibleButtonStyle.Text,
                compact = true,
            )
        is SyncBanner.Unreachable ->
            IndelibleButton(
                text = stringResource(Res.string.library_sync_banner_retry_now),
                onClick = onRetryNow,
                style = IndelibleButtonStyle.OutlinedDestructive,
                compact = true,
            )
        else -> Unit
    }
}

@Composable
private fun SyncingDetail(
    banner: SyncBanner.Syncing,
    tone: Tone,
) {
    val toGo = (banner.total - banner.sent).coerceAtLeast(0)
    LinearProgressIndicator(
        progress = { if (banner.total == 0) 0f else banner.sent.toFloat() / banner.total },
        color = tone.fg,
        trackColor = tone.fg.copy(alpha = TRACK_ALPHA),
        modifier = Modifier.fillMaxWidth().height(IndelibleSpacing.step4),
    )
    Text(
        text =
            stringResource(
                Res.string.library_sync_banner_key,
                pluralStringResource(Res.plurals.library_sync_banner_sent, banner.sent, banner.sent),
                pluralStringResource(Res.plurals.library_sync_banner_to_go, toGo, toGo),
            ),
        style = MaterialTheme.typography.bodySmall,
        color = MaterialTheme.colorScheme.onSurfaceVariant,
    )
}

@Preview(showBackground = true)
@Composable
private fun SyncBannerPreviewLight() {
    AppTheme(darkTheme = false) {
        Column {
            SyncBannerBar(SyncBanner.Syncing(1, 3), onReview = {}, onRetryNow = {})
            SyncBannerBar(SyncBanner.Attention(2, 1), onReview = {}, onRetryNow = {})
            SyncBannerBar(SyncBanner.AllSynced, onReview = {}, onRetryNow = {})
            SyncBannerBar(SyncBanner.Unreachable("indelible.home.arpa", 4), onReview = {}, onRetryNow = {})
        }
    }
}

@Preview(showBackground = true, uiMode = 0x20)
@Composable
private fun SyncBannerPreviewDark() {
    AppTheme(darkTheme = true) {
        Column {
            SyncBannerBar(SyncBanner.Syncing(1, 3), onReview = {}, onRetryNow = {})
            SyncBannerBar(SyncBanner.Unreachable("indelible.home.arpa", 4), onReview = {}, onRetryNow = {})
        }
    }
}
