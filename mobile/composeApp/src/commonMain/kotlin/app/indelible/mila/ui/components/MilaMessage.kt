package app.indelible.mila.ui.components

import androidx.compose.animation.core.RepeatMode
import androidx.compose.animation.core.StartOffset
import androidx.compose.animation.core.animateFloat
import androidx.compose.animation.core.infiniteRepeatable
import androidx.compose.animation.core.rememberInfiniteTransition
import androidx.compose.animation.core.tween
import androidx.compose.foundation.BorderStroke
import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.ExperimentalLayoutApi
import androidx.compose.foundation.layout.FlowRow
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.alpha
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.em
import app.indelible.mila.data.ChatMessage
import app.indelible.mila.data.SourceRef
import app.indelible.ui.theme.IndelibleShape
import app.indelible.ui.theme.IndelibleSpacing
import app.indelible.ui.theme.SerifFontFamily
import app.indelible.ui.theme.geistMonoFontFamily
import com.mikepenz.markdown.m3.Markdown
import com.mikepenz.markdown.m3.markdownTypography

@Composable
internal fun MilaMessage(
    message: ChatMessage,
    onSourceRef: (String) -> Unit,
) {
    if (message.role == "user") {
        UserMessageBubble(message = message)
    } else {
        AssistantMessage(
            message = message,
            onSourceRef = onSourceRef,
        )
    }
}

@Composable
private fun UserMessageBubble(message: ChatMessage) {
    Row(
        modifier = Modifier.fillMaxWidth(),
        horizontalArrangement = Arrangement.End,
    ) {
        Surface(
            shape = IndelibleShape.chatBubbleEnd,
            color = MaterialTheme.colorScheme.primary,
            modifier = Modifier.padding(start = IndelibleSpacing.step40),
        ) {
            Text(
                text = message.content,
                style = MaterialTheme.typography.bodyMedium,
                color = MaterialTheme.colorScheme.onPrimary,
                modifier =
                    Modifier.padding(
                        horizontal = IndelibleSpacing.step12,
                        vertical = IndelibleSpacing.step8,
                    ),
            )
        }
    }
}

@OptIn(ExperimentalLayoutApi::class)
@Composable
private fun AssistantMessage(
    message: ChatMessage,
    onSourceRef: (String) -> Unit,
) {
    Column(
        modifier =
            Modifier
                .fillMaxWidth()
                .padding(end = IndelibleSpacing.step40),
    ) {
        Row(
            verticalAlignment = Alignment.CenterVertically,
            horizontalArrangement = Arrangement.spacedBy(IndelibleSpacing.step6),
        ) {
            Box(
                modifier =
                    Modifier
                        .size(IndelibleSpacing.step6)
                        .background(MaterialTheme.colorScheme.primary, CircleShape),
            )
            Text(
                text = "MILA",
                style = milaEyebrowStyle(),
                color = MaterialTheme.colorScheme.primary,
            )
        }
        Spacer(modifier = Modifier.height(IndelibleSpacing.step6))

        if (message.isStreaming) {
            Text(
                text = message.content,
                style = MaterialTheme.typography.bodyLarge.copy(fontFamily = SerifFontFamily),
                color = MaterialTheme.colorScheme.onSurface,
            )
            TypingDots()
        } else {
            Markdown(
                content = message.content,
                typography =
                    markdownTypography(
                        text = MaterialTheme.typography.bodyLarge.copy(fontFamily = SerifFontFamily),
                        paragraph = MaterialTheme.typography.bodyLarge.copy(fontFamily = SerifFontFamily),
                    ),
            )
        }

        if (!message.isStreaming && message.sourceRefs.isNotEmpty()) {
            Spacer(modifier = Modifier.height(IndelibleSpacing.step8))
            FlowRow(
                horizontalArrangement = Arrangement.spacedBy(IndelibleSpacing.step4),
                verticalArrangement = Arrangement.spacedBy(IndelibleSpacing.step4),
            ) {
                message.sourceRefs.forEach { ref ->
                    SourceChip(
                        sourceRef = ref,
                        onClick = { onSourceRef(ref.documentId) },
                    )
                }
            }
        }
    }
}

@Composable
private fun TypingDots() {
    val transition = rememberInfiniteTransition(label = "typing")
    Row(
        modifier = Modifier.padding(top = IndelibleSpacing.step4),
        horizontalArrangement = Arrangement.spacedBy(IndelibleSpacing.step4),
    ) {
        repeat(TYPING_DOT_COUNT) { index ->
            val alpha by transition.animateFloat(
                initialValue = 0.35f,
                targetValue = 1f,
                animationSpec =
                    infiniteRepeatable(
                        animation = tween(durationMillis = TYPING_DOT_PERIOD_MS),
                        repeatMode = RepeatMode.Reverse,
                        initialStartOffset = StartOffset(index * TYPING_DOT_STAGGER_MS),
                    ),
                label = "typingDot$index",
            )
            Box(
                modifier =
                    Modifier
                        .size(IndelibleSpacing.step6)
                        .alpha(alpha)
                        .background(MaterialTheme.colorScheme.onSurfaceVariant, CircleShape),
            )
        }
    }
}

@Composable
private fun SourceChip(
    sourceRef: SourceRef,
    onClick: () -> Unit,
) {
    Surface(
        shape = IndelibleShape.sm,
        color = MaterialTheme.colorScheme.surfaceContainer,
        border = BorderStroke(IndelibleSpacing.hairline, MaterialTheme.colorScheme.outline),
        modifier = Modifier.clickable(onClick = onClick),
    ) {
        Row(
            modifier =
                Modifier.padding(
                    horizontal = IndelibleSpacing.step8,
                    vertical = IndelibleSpacing.step4,
                ),
            verticalAlignment = Alignment.CenterVertically,
            horizontalArrangement = Arrangement.spacedBy(IndelibleSpacing.step6),
        ) {
            Box(
                modifier =
                    Modifier
                        .size(IndelibleSpacing.step6)
                        .background(MaterialTheme.colorScheme.primary, CircleShape),
            )
            Text(
                text = sourceRef.itemTitle.take(MAX_SOURCE_TITLE_LENGTH),
                style = milaMonoStyle(),
                color = MaterialTheme.colorScheme.onSurfaceVariant,
                maxLines = 1,
                overflow = TextOverflow.Ellipsis,
            )
        }
    }
}

@Composable
private fun milaEyebrowStyle() =
    MaterialTheme.typography.labelSmall.copy(
        fontFamily = geistMonoFontFamily(),
        fontWeight = FontWeight.SemiBold,
        letterSpacing = 0.12.em,
    )

@Composable
private fun milaMonoStyle() =
    MaterialTheme.typography.labelSmall.copy(
        fontFamily = geistMonoFontFamily(),
        fontWeight = FontWeight.SemiBold,
    )

private const val MAX_SOURCE_TITLE_LENGTH = 24
private const val TYPING_DOT_COUNT = 3
private const val TYPING_DOT_PERIOD_MS = 600
private const val TYPING_DOT_STAGGER_MS = 160
