package app.indelible.auth.viewmodel

import app.indelible.auth.oauth.OAuthCallbackResult
import app.indelible.auth.oauth.PendingOAuthFlow
import app.indelible.auth.oauth.isExpired
import app.indelible.auth.oauth.parseOAuthCallback
import app.indelible.auth.oauth.pendingFlowExpiry
import app.indelible.core.i18n.UiMessage
import app.indelible.core.network.resolvedServerUrl
import app.indelible.core.storage.TokenStorage
import indelible.composeapp.generated.resources.Res
import indelible.composeapp.generated.resources.auth_oauth_code_missing
import indelible.composeapp.generated.resources.auth_oauth_expired
import indelible.composeapp.generated.resources.auth_oauth_failed
import indelible.composeapp.generated.resources.auth_oauth_state_mismatch
import org.jetbrains.compose.resources.StringResource

internal sealed class OAuthValidationResult {
    data object ParseFailed : OAuthValidationResult()

    data class Rejected(
        val message: UiMessage,
    ) : OAuthValidationResult()

    data class Proceed(
        val code: String,
        val verifier: String,
    ) : OAuthValidationResult()
}

internal suspend fun TokenStorage.beginOAuthFlow(
    providerId: String,
    verifier: String,
    appState: String,
) {
    savePendingOAuthFlow(
        PendingOAuthFlow(
            providerId = providerId,
            verifier = verifier,
            appState = appState,
            serverUrl = resolvedServerUrl(),
            expiresAtEpochSeconds = pendingFlowExpiry(),
        ),
    )
}

internal suspend fun TokenStorage.validateOAuthCallback(url: String): OAuthValidationResult {
    val callback = parseOAuthCallback(url) ?: return OAuthValidationResult.ParseFailed
    val result = evaluate(callback, getPendingOAuthFlow())
    if (result is OAuthValidationResult.Rejected) clearPendingOAuthFlow()
    return result
}

private fun evaluate(
    callback: OAuthCallbackResult,
    pending: PendingOAuthFlow?,
): OAuthValidationResult {
    val code = callback.code
    return when {
        pending == null || isExpired(pending) -> rejected(Res.string.auth_oauth_expired)
        callback.state != pending.appState -> rejected(Res.string.auth_oauth_state_mismatch)
        callback.error != null -> rejected(Res.string.auth_oauth_failed)
        code.isNullOrBlank() -> rejected(Res.string.auth_oauth_code_missing)
        else -> OAuthValidationResult.Proceed(code = code, verifier = pending.verifier)
    }
}

private fun rejected(message: StringResource) = OAuthValidationResult.Rejected(UiMessage(message))
