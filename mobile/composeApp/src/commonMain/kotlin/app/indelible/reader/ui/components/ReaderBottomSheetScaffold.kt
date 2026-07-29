package app.indelible.reader.ui.components

import androidx.compose.animation.AnimatedVisibility
import androidx.compose.animation.core.FastOutSlowInEasing
import androidx.compose.animation.core.tween
import androidx.compose.animation.fadeIn
import androidx.compose.animation.fadeOut
import androidx.compose.animation.slideInVertically
import androidx.compose.animation.slideOutVertically
import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.interaction.MutableInteractionSource
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.ColumnScope
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxHeight
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.navigationBarsPadding
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.remember
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.tooling.preview.Preview
import app.indelible.ui.theme.AppTheme
import app.indelible.ui.theme.IndelibleShape
import app.indelible.ui.theme.IndelibleSpacing

/**
 * Shared container for the reader's bottom-anchored panels (display, highlight,
 * note/tags, etc.). Renders a tap-to-dismiss scrim and a rounded surface that
 * slides up from the bottom, leaving room below for the floating dock.
 */
@Composable
fun ReaderBottomSheetScaffold(
    visible: Boolean,
    eyebrow: String,
    onDismiss: () -> Unit,
    modifier: Modifier = Modifier,
    fillHeightFraction: Float? = null,
    trailing: (@Composable () -> Unit)? = null,
    content: @Composable ColumnScope.() -> Unit,
) {
    Box(modifier = modifier.fillMaxSize()) {
        AnimatedVisibility(
            visible = visible,
            enter = fadeIn(),
            exit = fadeOut(),
        ) {
            Box(
                modifier =
                    Modifier
                        .fillMaxSize()
                        .background(MaterialTheme.colorScheme.scrim.copy(alpha = 0.32f))
                        .clickable(
                            interactionSource = remember { MutableInteractionSource() },
                            indication = null,
                            onClick = onDismiss,
                        ),
            )
        }

        AnimatedVisibility(
            visible = visible,
            enter =
                slideInVertically(
                    animationSpec = tween(easing = FastOutSlowInEasing),
                    initialOffsetY = { it },
                ) + fadeIn(),
            exit =
                slideOutVertically(
                    animationSpec = tween(easing = FastOutSlowInEasing),
                    targetOffsetY = { it },
                ) + fadeOut(),
            modifier = Modifier.align(Alignment.BottomCenter),
        ) {
            Surface(
                modifier =
                    Modifier
                        .fillMaxWidth()
                        .then(
                            if (fillHeightFraction != null) {
                                Modifier.fillMaxHeight(fillHeightFraction)
                            } else {
                                Modifier
                            },
                        )
                        .padding(horizontal = IndelibleSpacing.step12)
                        .navigationBarsPadding()
                        .padding(bottom = IndelibleSpacing.step80),
                shape = IndelibleShape.xxl,
                color = MaterialTheme.colorScheme.surfaceContainer,
                shadowElevation = IndelibleSpacing.step8,
            ) {
                Column(
                    modifier =
                        Modifier
                            .fillMaxWidth()
                            .then(
                                if (fillHeightFraction != null) Modifier.fillMaxHeight() else Modifier,
                            )
                            .padding(
                                horizontal = IndelibleSpacing.screenPaddingH,
                                vertical = IndelibleSpacing.step16,
                            ),
                ) {
                    Box(
                        modifier =
                            Modifier
                                .align(Alignment.CenterHorizontally)
                                .width(IndelibleSpacing.step32)
                                .height(IndelibleSpacing.step4)
                                .clip(IndelibleShape.full)
                                .background(MaterialTheme.colorScheme.outlineVariant),
                    )

                    Spacer(modifier = Modifier.height(IndelibleSpacing.step16))

                    Row(
                        modifier = Modifier.fillMaxWidth(),
                        horizontalArrangement = Arrangement.SpaceBetween,
                        verticalAlignment = Alignment.CenterVertically,
                    ) {
                        Text(
                            text = eyebrow.uppercase(),
                            style = MaterialTheme.typography.labelSmall,
                            color = MaterialTheme.colorScheme.onSurfaceVariant,
                        )
                        trailing?.invoke()
                    }

                    Spacer(modifier = Modifier.height(IndelibleSpacing.step16))

                    if (fillHeightFraction != null) {
                        Column(
                            modifier =
                                Modifier
                                    .fillMaxWidth()
                                    .weight(1f)
                                    .verticalScroll(rememberScrollState()),
                        ) {
                            content()
                        }
                    } else {
                        content()
                    }
                }
            }
        }
    }
}

@Preview
@Composable
private fun ReaderBottomSheetScaffoldPreviewLight() {
    AppTheme(darkTheme = false) {
        ReaderBottomSheetScaffold(
            visible = true,
            eyebrow = "Display",
            onDismiss = {},
        ) {
            Text(
                text = "Panel content",
                style = MaterialTheme.typography.bodyMedium,
                color = MaterialTheme.colorScheme.onSurface,
            )
        }
    }
}

@Preview
@Composable
private fun ReaderBottomSheetScaffoldPreviewDark() {
    AppTheme(darkTheme = true) {
        ReaderBottomSheetScaffold(
            visible = true,
            eyebrow = "Highlight",
            onDismiss = {},
        ) {
            Text(
                text = "Panel content",
                style = MaterialTheme.typography.bodyMedium,
                color = MaterialTheme.colorScheme.onSurface,
            )
        }
    }
}
