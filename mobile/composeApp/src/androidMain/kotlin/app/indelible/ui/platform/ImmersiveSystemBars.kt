package app.indelible.ui.platform

import android.app.Activity
import android.content.Context
import android.content.ContextWrapper
import androidx.compose.runtime.Composable
import androidx.compose.runtime.DisposableEffect
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.remember
import androidx.compose.ui.platform.LocalView
import androidx.core.view.WindowCompat
import androidx.core.view.WindowInsetsCompat
import androidx.core.view.WindowInsetsControllerCompat

@Composable
actual fun ImmersiveSystemBars(hidden: Boolean) {
    val view = LocalView.current
    val controller =
        remember(view) {
            view.context.findActivity()?.window?.let { window ->
                WindowCompat.getInsetsController(window, view)
            }
        }
    LaunchedEffect(controller, hidden) {
        val insets = controller ?: return@LaunchedEffect
        // Let a swipe transiently reveal the bars so the user is never trapped.
        insets.systemBarsBehavior =
            WindowInsetsControllerCompat.BEHAVIOR_SHOW_TRANSIENT_BARS_BY_SWIPE
        if (hidden) {
            // Both bars, not just the status bar: the reader retires the navigation
            // bar too, so the page gets the whole screen rather than losing a strip
            // at the foot to a gesture handle it never uses.
            insets.hide(WindowInsetsCompat.Type.systemBars())
        } else {
            insets.show(WindowInsetsCompat.Type.systemBars())
        }
    }
    DisposableEffect(controller) {
        onDispose { controller?.show(WindowInsetsCompat.Type.systemBars()) }
    }
}

@Composable
actual fun StatusBarAppearance(lightStatusBars: Boolean) {
    val view = LocalView.current
    val controller =
        remember(view) {
            view.context.findActivity()?.window?.let { window ->
                WindowCompat.getInsetsController(window, view)
            }
        }
    DisposableEffect(controller, lightStatusBars) {
        if (controller == null) {
            onDispose { }
        } else {
            val previous = controller.isAppearanceLightStatusBars
            controller.isAppearanceLightStatusBars = lightStatusBars
            onDispose { controller.isAppearanceLightStatusBars = previous }
        }
    }
}

private fun Context.findActivity(): Activity? {
    var context: Context = this
    while (context is ContextWrapper) {
        if (context is Activity) return context
        context = context.baseContext
    }
    return null
}
