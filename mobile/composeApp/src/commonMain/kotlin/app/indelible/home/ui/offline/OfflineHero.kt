package app.indelible.home.ui.offline

import androidx.compose.animation.AnimatedContent
import androidx.compose.animation.core.tween
import androidx.compose.animation.fadeIn
import androidx.compose.animation.fadeOut
import androidx.compose.animation.togetherWith
import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.FlowRow
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxHeight
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.State
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.draw.drawWithContent
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.graphics.Brush
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.text.SpanStyle
import androidx.compose.ui.text.buildAnnotatedString
import androidx.compose.ui.text.font.FontStyle
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.withStyle
import androidx.compose.ui.unit.dp
import app.indelible.home.viewmodel.OfflineShelfState
import app.indelible.home.viewmodel.ShelfNet
import app.indelible.ui.theme.IndelibleIcons
import app.indelible.ui.theme.IndelibleShape
import app.indelible.ui.theme.IndelibleSpacing
import app.indelible.ui.theme.IndelibleTheme
import app.indelible.ui.theme.MonochromeColors
import app.indelible.ui.theme.OfflineIcons
import app.indelible.ui.theme.WindowScene
import indelible.composeapp.generated.resources.Res
import indelible.composeapp.generated.resources.home_offline_body
import indelible.composeapp.generated.resources.home_offline_chip_back
import indelible.composeapp.generated.resources.home_offline_chip_offline
import indelible.composeapp.generated.resources.home_offline_chip_synced
import indelible.composeapp.generated.resources.home_offline_empty_body
import indelible.composeapp.generated.resources.home_offline_empty_subtitle
import indelible.composeapp.generated.resources.home_offline_items_on_board
import indelible.composeapp.generated.resources.home_offline_on_phone
import indelible.composeapp.generated.resources.home_offline_sending_body
import indelible.composeapp.generated.resources.home_offline_sending_progress
import indelible.composeapp.generated.resources.home_offline_sending_subtitle
import indelible.composeapp.generated.resources.home_offline_sending_title
import indelible.composeapp.generated.resources.home_offline_subtitle
import indelible.composeapp.generated.resources.home_offline_synced_body
import indelible.composeapp.generated.resources.home_offline_synced_pill
import indelible.composeapp.generated.resources.home_offline_synced_title
import indelible.composeapp.generated.resources.home_offline_title
import indelible.composeapp.generated.resources.home_offline_to_send
import org.jetbrains.compose.resources.pluralStringResource
import org.jetbrains.compose.resources.stringResource

private enum class HeroVariant { OFFLINE, EMPTY, SENDING, SYNCED }

/** The window-seat card at the top of the offline Home; the sky and the words follow [shelf]'s network state. */
@Composable
internal fun OfflineHero(
    shelf: OfflineShelfState,
    modifier: Modifier = Modifier,
) {
    val mono = IndelibleTheme.monochrome
    val variant =
        when {
            shelf.net == ShelfNet.SENDING -> HeroVariant.SENDING
            shelf.net == ShelfNet.SYNCED -> HeroVariant.SYNCED
            shelf.items.isEmpty() -> HeroVariant.EMPTY
            else -> HeroVariant.OFFLINE
        }
    val clock = rememberSceneClock(variant to shelf.sent)
    Column(
        modifier
            .polished(mono, IndelibleShape.xxl, mono.raised)
            .glint(clock)
            .padding(start = 7.dp, end = 7.dp, top = 7.dp, bottom = 18.dp),
    ) {
        Box(Modifier.fillMaxWidth().height(SCENE_HEIGHT).clip(IndelibleShape.xl)) {
            WindowSceneView(variant.sky(), clock, Modifier.fillMaxWidth().fillMaxHeight())
            SceneChip(variant, Modifier.align(Alignment.TopStart).padding(IndelibleSpacing.step12))
            SceneCount(
                shelf.items.size,
                Modifier
                    .align(Alignment.BottomStart)
                    .padding(start = IndelibleSpacing.step14, bottom = IndelibleSpacing.step12),
            )
        }
        AnimatedContent(
            variant,
            transitionSpec = { fadeIn(tween(SWAP_MS)) togetherWith fadeOut(tween(SWAP_MS)) },
        ) { shown ->
            Column(Modifier.padding(start = 13.dp, end = 13.dp, top = 18.dp)) {
                HeroBody(shown, shelf, mono, clock)
            }
        }
    }
}

private fun HeroVariant.sky(): SkyState =
    when (this) {
        HeroVariant.OFFLINE -> SkyState.DUSK
        HeroVariant.EMPTY -> SkyState.NIGHT
        HeroVariant.SENDING -> SkyState.DAWN
        HeroVariant.SYNCED -> SkyState.DAY
    }

@Composable
private fun HeroBody(
    variant: HeroVariant,
    shelf: OfflineShelfState,
    mono: MonochromeColors,
    clock: State<Float>,
) {
    when (variant) {
        HeroVariant.OFFLINE -> {
            HeroTitle(
                stringResource(Res.string.home_offline_title),
                stringResource(Res.string.home_offline_subtitle),
                mono,
            )
            HeroText(stringResource(Res.string.home_offline_body), mono)
            FlowRow(
                horizontalArrangement = Arrangement.spacedBy(IndelibleSpacing.step8),
                verticalArrangement = Arrangement.spacedBy(IndelibleSpacing.step8),
                modifier = Modifier.padding(top = IndelibleSpacing.step16),
            ) {
                val count = shelf.items.size
                val onPhone = pluralStringResource(Res.plurals.home_offline_on_phone, count, count)
                Pill(onPhone, OfflineIcons.Check, PillTone.OK, mono)
                if (shelf.queued > 0) {
                    val toSend = pluralStringResource(Res.plurals.home_offline_to_send, shelf.queued, shelf.queued)
                    Pill(toSend, IndelibleIcons.Clock, PillTone.WAIT, mono)
                }
            }
        }
        HeroVariant.EMPTY -> {
            HeroTitle(
                stringResource(Res.string.home_offline_title),
                stringResource(Res.string.home_offline_empty_subtitle),
                mono,
            )
            HeroText(stringResource(Res.string.home_offline_empty_body), mono)
        }
        HeroVariant.SENDING -> {
            HeroTitle(
                stringResource(Res.string.home_offline_sending_title),
                stringResource(Res.string.home_offline_sending_subtitle),
                mono,
            )
            HeroText(stringResource(Res.string.home_offline_sending_body), mono)
            val sending = (shelf.sent + 1).coerceAtMost(shelf.sendTotal)
            Row(Modifier.padding(top = IndelibleSpacing.step16)) {
                val progress = stringResource(Res.string.home_offline_sending_progress, sending, shelf.sendTotal)
                Pill(progress, OfflineIcons.ArrowUp, PillTone.LIVE, mono)
            }
            SyncBar(if (shelf.sendTotal == 0) 0f else shelf.sent.toFloat() / shelf.sendTotal, done = false, mono, clock)
        }
        HeroVariant.SYNCED -> {
            HeroTitle(stringResource(Res.string.home_offline_synced_title), null, mono)
            HeroText(pluralStringResource(Res.plurals.home_offline_synced_body, shelf.sendTotal, shelf.sendTotal), mono)
            Row(Modifier.padding(top = IndelibleSpacing.step16)) {
                Pill(stringResource(Res.string.home_offline_synced_pill), OfflineIcons.Check, PillTone.OK, mono)
            }
            SyncBar(1f, done = true, mono, clock)
        }
    }
}

@Composable
private fun HeroTitle(
    title: String,
    subtitle: String?,
    mono: MonochromeColors,
) {
    val text =
        buildAnnotatedString {
            append(title)
            if (subtitle != null) {
                append(" ")
                val quiet = SpanStyle(fontStyle = FontStyle.Italic, fontWeight = FontWeight.Normal, color = mono.ink2)
                withStyle(quiet) { append(subtitle) }
            }
        }
    Text(text, style = serif(MaterialTheme.typography.headlineMedium), color = mono.ink)
}

@Composable
private fun HeroText(
    text: String,
    mono: MonochromeColors,
) {
    Text(
        text,
        style = MaterialTheme.typography.bodyLarge,
        color = mono.ink2,
        modifier = Modifier.padding(top = IndelibleSpacing.step10),
    )
}

@Composable
private fun SceneChip(
    variant: HeroVariant,
    modifier: Modifier,
) {
    val (icon: ImageVector, label) =
        when (variant) {
            HeroVariant.SENDING -> OfflineIcons.Wifi to stringResource(Res.string.home_offline_chip_back)
            HeroVariant.SYNCED -> OfflineIcons.Check to stringResource(Res.string.home_offline_chip_synced)
            else -> OfflineIcons.Airplane to stringResource(Res.string.home_offline_chip_offline)
        }
    Row(
        verticalAlignment = Alignment.CenterVertically,
        horizontalArrangement = Arrangement.spacedBy(7.dp),
        modifier =
            modifier
                .height(27.dp)
                .clip(IndelibleShape.full)
                .background(WindowScene.chipFill)
                .border(1.dp, WindowScene.chipEdge, IndelibleShape.full)
                .padding(start = 9.dp, end = 11.dp),
    ) {
        Icon(icon, contentDescription = null, tint = WindowScene.onScene, modifier = Modifier.size(13.dp))
        Text(label, style = mono(), color = WindowScene.onScene)
    }
}

@Composable
private fun SceneCount(
    count: Int,
    modifier: Modifier,
) {
    Column(modifier) {
        Text(count.toString(), style = serif(MaterialTheme.typography.displaySmall), color = WindowScene.onScene)
        Text(
            pluralStringResource(Res.plurals.home_offline_items_on_board, count),
            style = mono(),
            color = WindowScene.onScene.copy(alpha = 0.85f),
        )
    }
}

/** [fraction] of the queue sent; a light runs along the bar while it sends. */
@Composable
private fun SyncBar(
    fraction: Float,
    done: Boolean,
    mono: MonochromeColors,
    clock: State<Float>,
) {
    Box(
        Modifier
            .padding(top = IndelibleSpacing.step14)
            .fillMaxWidth()
            .height(5.dp)
            .clip(IndelibleShape.full)
            .background(mono.track),
    ) {
        Box(
            Modifier
                .fillMaxWidth(fraction.coerceIn(0f, 1f))
                .fillMaxHeight()
                .clip(IndelibleShape.full)
                .background(if (done) Brush.linearGradient(listOf(mono.success, mono.success)) else bone(mono))
                .drawWithContent {
                    drawContent()
                    if (!done) {
                        val x = (phase(clock.value, SHIMMER_MS) * 2 - 1) * size.width
                        val light = listOf(Color.Transparent, Color.White.copy(alpha = 0.9f), Color.Transparent)
                        drawRect(Brush.horizontalGradient(light, x, x + size.width))
                    }
                },
        )
    }
}

/** One slow light across the card, the way light crosses a lacquered surface. */
private fun Modifier.glint(clock: State<Float>): Modifier =
    drawWithContent {
        drawContent()
        val sweep = (phase(clock.value, GLINT_MS) / GLINT_SWEEP).coerceAtMost(1f)
        if (sweep < 1f) {
            val band = size.width * GLINT_BAND
            val x = -size.width * GLINT_LEAD + sweep * band * GLINT_TRAVEL
            drawRect(
                Brush.linearGradient(
                    listOf(Color.Transparent, Color.White.copy(alpha = 0.07f), Color.Transparent),
                    Offset(x, 0f),
                    Offset(x + band, size.height * GLINT_TILT),
                ),
            )
        }
    }

private val SCENE_HEIGHT = 206.dp
private const val SWAP_MS = 520
private const val SHIMMER_MS = 1_600
private const val GLINT_MS = 9_000
private const val GLINT_SWEEP = 0.22f
private const val GLINT_BAND = 0.4f
private const val GLINT_LEAD = 0.6f
private const val GLINT_TRAVEL = 4.2f
private const val GLINT_TILT = 0.2f
