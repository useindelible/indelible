package app.indelible.auth.viewmodel

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import app.indelible.auth.oauth.NoopOAuthBrowserLauncher
import app.indelible.auth.oauth.OAuthBrowserLauncher
import app.indelible.auth.oauth.OAuthCallbackBus
import app.indelible.auth.oauth.OAuthProviderUi
import app.indelible.auth.oauth.codeChallenge
import app.indelible.auth.oauth.generateAppState
import app.indelible.auth.oauth.generateCodeVerifier
import app.indelible.auth.repository.AuthRepository
import app.indelible.core.i18n.UiMessage
import app.indelible.core.model.AuthUser
import app.indelible.core.model.toAuthUser
import app.indelible.core.network.ApiException
import app.indelible.core.offline.OutboxWorker
import app.indelible.core.offline.ScopePurger
import app.indelible.core.offline.SessionTransitions
import app.indelible.core.offline.TransitionResult
import app.indelible.core.offline.currentOfflineScope
import app.indelible.core.storage.TokenStorage
import indelible.composeapp.generated.resources.Res
import indelible.composeapp.generated.resources.auth_login_failed
import indelible.composeapp.generated.resources.auth_login_invalid_credentials
import indelible.composeapp.generated.resources.auth_logout_revoke_failed
import indelible.composeapp.generated.resources.auth_oauth_browser_failed
import indelible.composeapp.generated.resources.auth_oauth_failed
import indelible.composeapp.generated.resources.auth_password_reset_failed
import indelible.composeapp.generated.resources.auth_register_failed
import indelible.composeapp.generated.resources.auth_session_load_failed
import kotlinx.coroutines.CancellationException
import kotlinx.coroutines.delay
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.launch
import kotlinx.coroutines.sync.Mutex
import kotlinx.coroutines.sync.withLock

@Suppress("TooManyFunctions")
class AuthViewModel(
    private val repository: AuthRepository,
    private val tokenStorage: TokenStorage,
    private val outboxWorker: OutboxWorker,
    private val scopePurger: ScopePurger,
    private val sessions: SessionTransitions,
    private val oauthBrowserLauncher: OAuthBrowserLauncher = NoopOAuthBrowserLauncher,
) : ViewModel() {
    private val _authState = MutableStateFlow<AuthState>(AuthState.Loading)
    val authState: StateFlow<AuthState> = _authState.asStateFlow()

    private val _avatarBytes = MutableStateFlow<ByteArray?>(null)
    val avatarBytes: StateFlow<ByteArray?> = _avatarBytes.asStateFlow()

    val isAuthenticated: StateFlow<Boolean> =
        authState
            .map {
                it is AuthState.Authenticated
            }.stateIn(viewModelScope, SharingStarted.WhileSubscribed(), false)

    private val _loginState = MutableStateFlow(LoginState())
    val loginState: StateFlow<LoginState> = _loginState.asStateFlow()

    private val _registerState = MutableStateFlow(RegisterState())
    val registerState: StateFlow<RegisterState> = _registerState.asStateFlow()

    private val _forgotPasswordState = MutableStateFlow(ForgotPasswordState())
    val forgotPasswordState: StateFlow<ForgotPasswordState> = _forgotPasswordState.asStateFlow()

    private val _oauthProviders = MutableStateFlow<List<OAuthProviderUi>>(emptyList())
    val oauthProviders: StateFlow<List<OAuthProviderUi>> = _oauthProviders.asStateFlow()

    private val _signupsEnabled = MutableStateFlow(false)
    val signupsEnabled: StateFlow<Boolean> = _signupsEnabled.asStateFlow()

    private val _setupRequired = MutableStateFlow(false)
    val setupRequired: StateFlow<Boolean> = _setupRequired.asStateFlow()

    private val oauthCallbackMutex = Mutex()
    private var lastHandledOAuthCallbackUrl: String? = null

    init {
        initialize()
        loadOAuthProviders()
        observeOAuthCallbacks()
    }

    fun initialize() {
        viewModelScope.launch {
            _authState.value = AuthState.Loading
            val token = tokenStorage.getToken()
            val refreshToken = tokenStorage.getRefreshToken()
            if (token == null && refreshToken == null) {
                _authState.value = AuthState.Unauthenticated
                return@launch
            }
            val epoch = sessions.epoch()
            repository
                .getSession()
                .onSuccess { user -> signIn(epoch, user) }
                .onFailure { if (sessions.epoch() == epoch) _authState.value = AuthState.Unauthenticated }
        }
    }

    fun updateLoginEmail(email: String) {
        _loginState.value = _loginState.value.copy(email = email, emailError = null)
    }

    fun updateLoginPassword(password: String) {
        _loginState.value = _loginState.value.copy(password = password, passwordError = null)
    }

    fun login() {
        val validated = _loginState.value.validate()
        _loginState.value = validated
        if (!validated.isValid) return

        viewModelScope.launch {
            _loginState.value = _loginState.value.copy(isLoading = true, serverError = null)
            val epoch = sessions.epoch()
            repository
                .login(validated.email, validated.password)
                .onSuccess { response ->
                    _loginState.value = _loginState.value.copy(isLoading = false)
                    signIn(epoch, response.toAuthUser()) { tokenStorage.saveCredentials(response) }
                }.onFailure { error ->
                    _loginState.value =
                        _loginState.value.copy(
                            isLoading = false,
                            serverError = loginFailureMessage(error),
                        )
                }
        }
    }

    fun loadOAuthProviders() {
        viewModelScope.launch {
            repository
                .getOAuthProviders()
                .onSuccess { response ->
                    _signupsEnabled.value = response.signupsEnabled
                    _setupRequired.value = response.setupRequired
                    _oauthProviders.value =
                        response.providers
                            .filter { it.enabled }
                            .map { OAuthProviderUi(id = it.id, name = it.name) }
                }.onFailure {
                    _signupsEnabled.value = false
                    _setupRequired.value = false
                    _oauthProviders.value = emptyList()
                }
        }
    }

    fun startOAuth(providerId: String) {
        viewModelScope.launch {
            _loginState.value = _loginState.value.copy(isLoading = true, serverError = null)
            val verifier = generateCodeVerifier()
            val appState = generateAppState()
            val challenge = codeChallenge(verifier)
            val startUrl = repository.nativeOAuthStartUrl(providerId, challenge, appState)
            lastHandledOAuthCallbackUrl = null
            tokenStorage.beginOAuthFlow(providerId, verifier, appState)
            oauthBrowserLauncher
                .launch(startUrl)
                .onFailure {
                    tokenStorage.clearPendingOAuthFlow()
                    _loginState.value =
                        _loginState.value.copy(
                            isLoading = false,
                            serverError = UiMessage(Res.string.auth_oauth_browser_failed),
                        )
                }
        }
    }

    private fun observeOAuthCallbacks() {
        viewModelScope.launch {
            OAuthCallbackBus.callbacks.collect { handleOAuthCallback(it) }
        }
    }

    fun handleOAuthCallback(url: String) {
        viewModelScope.launch {
            oauthCallbackMutex.withLock {
                if (lastHandledOAuthCallbackUrl == url) return@withLock

                when (val result = tokenStorage.validateOAuthCallback(url)) {
                    is OAuthValidationResult.Rejected -> {
                        lastHandledOAuthCallbackUrl = url
                        _loginState.value =
                            _loginState.value.copy(isLoading = false, serverError = result.message)
                    }
                    is OAuthValidationResult.Proceed -> {
                        exchangeAndLoadSession(
                            code = result.code,
                            verifier = result.verifier,
                            url = url,
                        )
                    }
                    OAuthValidationResult.ParseFailed -> Unit
                }
            }
        }
    }

    private suspend fun exchangeAndLoadSession(
        code: String,
        verifier: String,
        url: String,
    ) {
        val epoch = sessions.epoch()
        repository
            .exchangeNativeOAuthCode(code, verifier)
            .onSuccess { tokenResponse ->
                _loginState.value = _loginState.value.copy(isLoading = false)
                lastHandledOAuthCallbackUrl = url
                var user: AuthUser? = null
                val result =
                    sessions.transition(epoch) {
                        tokenStorage.saveToken(tokenResponse.accessToken)
                        tokenStorage.saveRefreshToken(tokenResponse.refreshToken)
                        tokenStorage.saveExpiresAt(tokenResponse.expiresAt)
                        tokenStorage.clearPendingOAuthFlow()
                        repository.getSession().onSuccess { fetched ->
                            registerSession(fetched)
                            user = fetched
                        }
                    }
                val signedIn = user
                when {
                    result is TransitionResult.Rejected -> Unit
                    signedIn != null -> publishAuthenticated(signedIn)
                    else -> {
                        _authState.value = AuthState.Unauthenticated
                        _loginState.value =
                            _loginState.value.copy(
                                serverError = UiMessage(Res.string.auth_session_load_failed),
                            )
                    }
                }
            }.onFailure {
                tokenStorage.clearPendingOAuthFlow()
                _loginState.value =
                    _loginState.value.copy(
                        isLoading = false,
                        serverError = UiMessage(Res.string.auth_oauth_failed),
                    )
                lastHandledOAuthCallbackUrl = url
            }
    }

    fun updateRegisterDisplayName(name: String) {
        _registerState.value =
            _registerState.value.copy(
                displayName = name,
                displayNameError = null,
            )
    }

    fun updateRegisterEmail(email: String) {
        _registerState.value = _registerState.value.copy(email = email, emailError = null)
    }

    fun updateRegisterPassword(password: String) {
        _registerState.value =
            _registerState.value.copy(
                password = password,
                passwordError = null,
            )
    }

    fun updateRegisterConfirmPassword(confirmPassword: String) {
        _registerState.value =
            _registerState.value.copy(
                confirmPassword = confirmPassword,
                confirmPasswordError = null,
            )
    }

    fun register() {
        val validated = _registerState.value.validate()
        _registerState.value = validated
        if (!validated.isValid) return

        viewModelScope.launch {
            _registerState.value =
                _registerState.value.copy(
                    isLoading = true,
                    serverError = null,
                )
            val epoch = sessions.epoch()
            repository
                .register(validated.displayName, validated.email, validated.password)
                .onSuccess { response ->
                    _registerState.value = _registerState.value.copy(isLoading = false)
                    signIn(epoch, response.toAuthUser()) { tokenStorage.saveCredentials(response) }
                }.onFailure {
                    _registerState.value =
                        _registerState.value.copy(
                            isLoading = false,
                            serverError = UiMessage(Res.string.auth_register_failed),
                        )
                }
        }
    }

    fun updateForgotPasswordEmail(email: String) {
        _forgotPasswordState.value =
            _forgotPasswordState.value.copy(
                email = email,
                emailError = null,
            )
    }

    fun forgotPassword() {
        val state = _forgotPasswordState.value
        val emailErr = LoginState.validateEmail(state.email)
        if (emailErr != null) {
            _forgotPasswordState.value = state.copy(emailError = emailErr)
            return
        }

        viewModelScope.launch {
            _forgotPasswordState.value =
                state.copy(
                    isLoading = true,
                    serverError = null,
                )
            repository
                .forgotPassword(state.email)
                .onSuccess {
                    _forgotPasswordState.value =
                        _forgotPasswordState.value.copy(
                            isLoading = false,
                            isSubmitted = true,
                        )
                }.onFailure {
                    _forgotPasswordState.value =
                        _forgotPasswordState.value.copy(
                            isLoading = false,
                            serverError = UiMessage(Res.string.auth_password_reset_failed),
                        )
                }
        }
    }

    fun resendVerification(onResult: (Boolean) -> Unit = {}) {
        viewModelScope.launch {
            repository
                .resendVerification()
                .onSuccess { onResult(true) }
                .onFailure { onResult(false) }
        }
    }

    fun pollVerificationStatus() {
        viewModelScope.launch {
            while (true) {
                delay(VERIFICATION_POLL_INTERVAL_MS)
                val epoch = sessions.epoch()
                repository
                    .getSession()
                    .onSuccess { user ->
                        if (user.emailVerified) {
                            signIn(epoch, user)
                            return@launch
                        }
                    }
            }
        }
    }

    fun logout() {
        viewModelScope.launch {
            val epoch = sessions.epoch()
            var revocation: Result<Unit> = Result.success(Unit)
            val result =
                sessions.transition(epoch) { scope ->
                    revocation = repository.logout()
                    clearAuthState()
                    // Signing out must survive a failed purge, and the token clear must land first:
                    // purge() sets purge_pending before it deletes anything, so a crash the other way
                    // round would leave a signed-in account whose next launch wipes its live queue.
                    scope?.let {
                        runCatching { scopePurger.purge(it) }
                            .onFailure { failure -> if (failure is CancellationException) throw failure }
                    }
                }
            if (result is TransitionResult.Rejected) return@launch
            _authState.value = AuthState.Unauthenticated
            revocation.exceptionOrNull()?.let {
                _loginState.value =
                    LoginState(
                        serverError = UiMessage(Res.string.auth_logout_revoke_failed),
                    )
            }
        }
    }

    /** Local sign-out for the session of [epoch]; a newer sign-in since then makes it a no-op. */
    fun forceLogout(epoch: Long = sessions.epoch()) {
        viewModelScope.launch {
            if (sessions.transition(epoch) { clearAuthState() } is TransitionResult.Applied) {
                _authState.value = AuthState.Unauthenticated
            }
        }
    }

    fun resetLoginState() {
        _loginState.value = LoginState()
    }

    fun resetRegisterState() {
        _registerState.value = RegisterState()
    }

    fun resetForgotPasswordState() {
        _forgotPasswordState.value = ForgotPasswordState()
    }

    fun updateProfile(
        displayName: String,
        onComplete: (Boolean) -> Unit,
    ) {
        viewModelScope.launch {
            val epoch = sessions.epoch()
            repository
                .updateProfile(displayName)
                .onSuccess { user -> onComplete(publishIfCurrent(epoch, user)) }
                .onFailure { onComplete(false) }
        }
    }

    // A profile edit changes no credential or scope, so it must not open a transition: burning
    // an epoch would fail every refresh in flight just to show a new display name.
    private fun publishIfCurrent(
        epoch: Long,
        user: AuthUser,
    ): Boolean {
        val current = sessions.epoch() == epoch
        if (current) publishAuthenticated(user)
        return current
    }

    /** One transition per auth result: credentials and registration inside, state publication outside. */
    private suspend fun signIn(
        epoch: Long,
        user: AuthUser,
        saveCredentials: suspend () -> Unit = {},
    ): Boolean {
        val result =
            sessions.transition(epoch) {
                saveCredentials()
                registerSession(user)
            }
        if (result is TransitionResult.Rejected) return false
        publishAuthenticated(user)
        return true
    }

    private suspend fun registerSession(user: AuthUser) {
        tokenStorage.saveUserId(user.id)
        // Sweeping stale scopes is housekeeping, so a failure must not strand the session on the
        // splash: the scopes it could not clear keep purge_pending and are retried next launch.
        runCatching { scopePurger.purgeInactive(tokenStorage.currentOfflineScope()) }
            .onFailure { if (it is CancellationException) throw it }
        outboxWorker.resumeAuth()
    }

    private fun publishAuthenticated(user: AuthUser) {
        val wasSetupRequired = _setupRequired.value
        _authState.value =
            when {
                !user.emailVerified -> AuthState.NeedsVerification(user)
                !user.onboardingCompleted -> AuthState.NeedsOnboarding(user)
                else -> AuthState.Authenticated(user)
            }
        if (wasSetupRequired) loadOAuthProviders()
        fetchAvatarBytesIfNeeded(user.avatarUrl)
    }

    private fun fetchAvatarBytesIfNeeded(avatarUrl: String?) {
        val url = avatarUrl ?: return
        // External URLs (Google OAuth, presigned S3) can be loaded by Coil directly
        if (!url.contains("/api/v1/")) return
        viewModelScope.launch {
            repository
                .fetchAvatarBytes(url)
                .onSuccess { bytes ->
                    println("[UserAvatar] avatar bytes loaded: ${bytes.size}")
                    _avatarBytes.value = bytes
                }.onFailure { e ->
                    println("[UserAvatar] avatar fetch failed: $e")
                }
        }
    }

    // The signed-out state is published by the caller once its transition has completed, so a
    // sign-in started from that state captures the epoch the sign-out produced.
    private suspend fun clearAuthState() {
        tokenStorage.clearAll()
        lastHandledOAuthCallbackUrl = null
        _avatarBytes.value = null
        _loginState.value = LoginState()
        _registerState.value = RegisterState()
        _forgotPasswordState.value = ForgotPasswordState()
    }

    private fun loginFailureMessage(error: Throwable): UiMessage =
        if (error is ApiException && error.statusCode == UNAUTHORIZED_STATUS) {
            UiMessage(Res.string.auth_login_invalid_credentials)
        } else {
            UiMessage(Res.string.auth_login_failed)
        }

    companion object {
        private const val UNAUTHORIZED_STATUS = 401
        private const val VERIFICATION_POLL_INTERVAL_MS = 5000L
    }
}
