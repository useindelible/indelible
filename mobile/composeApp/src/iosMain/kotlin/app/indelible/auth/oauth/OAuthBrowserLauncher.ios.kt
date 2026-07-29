package app.indelible.auth.oauth

import androidx.compose.runtime.Composable
import androidx.compose.runtime.remember
import kotlinx.cinterop.ExperimentalForeignApi
import kotlinx.coroutines.suspendCancellableCoroutine
import platform.AuthenticationServices.ASPresentationAnchor
import platform.AuthenticationServices.ASWebAuthenticationPresentationContextProvidingProtocol
import platform.AuthenticationServices.ASWebAuthenticationSession
import platform.Foundation.NSError
import platform.Foundation.NSURL
import platform.UIKit.UIApplication
import platform.UIKit.UIWindow
import platform.UIKit.UIWindowScene
import platform.darwin.NSObject
import kotlin.coroutines.resume

// ASWebAuthenticationSession requires a presentationContextProvider on iOS 13+ to locate
// the window it should present in. Without it the OS rejects the session with error 2
// (ASWebAuthenticationSessionErrorPresentationContextInvalid).
private class WindowContextProvider :
    NSObject(),
    ASWebAuthenticationPresentationContextProvidingProtocol {
    override fun presentationAnchorForWebAuthenticationSession(
        session: ASWebAuthenticationSession,
    ): ASPresentationAnchor {
        val windowScene =
            UIApplication.sharedApplication
                .connectedScenes
                .firstOrNull { it is UIWindowScene } as? UIWindowScene
        return windowScene
            ?.windows
            ?.filterIsInstance<UIWindow>()
            ?.firstOrNull { it.isKeyWindow() }
            ?: windowScene?.windows?.filterIsInstance<UIWindow>()?.firstOrNull()
            ?: UIWindow()
    }
}

class IosOAuthBrowserLauncher : OAuthBrowserLauncher {
    @OptIn(ExperimentalForeignApi::class)
    override suspend fun launch(url: String): Result<Unit> =
        suspendCancellableCoroutine { continuation ->
            val nsUrl = NSURL(string = url)
            val contextProvider = WindowContextProvider()

            val session =
                ASWebAuthenticationSession(
                    uRL = nsUrl,
                    callbackURLScheme = "com.useindelible.app",
                    completionHandler = { callbackUrl: NSURL?, error: NSError? ->
                        when {
                            callbackUrl != null -> {
                                OAuthCallbackBus.emit(callbackUrl.absoluteString ?: "")
                                continuation.resume(Result.success(Unit))
                            }
                            error != null ->
                                continuation.resume(
                                    Result.failure(Throwable(error.localizedDescription)),
                                )
                            else ->
                                continuation.resume(
                                    Result.failure(Throwable("OAuth sign-in was cancelled")),
                                )
                        }
                    },
                )
            session.presentationContextProvider = contextProvider
            session.prefersEphemeralWebBrowserSession = false
            continuation.invokeOnCancellation { session.cancel() }
            session.start()
        }
}

@Composable
actual fun rememberOAuthBrowserLauncher(): OAuthBrowserLauncher = remember { IosOAuthBrowserLauncher() }
