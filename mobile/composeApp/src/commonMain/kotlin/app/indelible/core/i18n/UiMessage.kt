package app.indelible.core.i18n

import androidx.compose.runtime.Composable
import org.jetbrains.compose.resources.StringResource
import org.jetbrains.compose.resources.getString
import org.jetbrains.compose.resources.stringResource

data class UiMessage(
    val resource: StringResource,
    val formatArgs: List<Any> = emptyList(),
)

@Composable
fun UiMessage.resolve(): String = stringResource(resource, *formatArgs.toTypedArray())

suspend fun UiMessage.resolveString(): String = getString(resource, *formatArgs.toTypedArray())
