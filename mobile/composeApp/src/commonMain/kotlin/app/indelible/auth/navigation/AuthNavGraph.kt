package app.indelible.auth.navigation

import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.runtime.remember
import androidx.navigation.compose.NavHost
import androidx.navigation.compose.composable
import androidx.navigation.compose.rememberNavController
import app.indelible.auth.server.ServerUrlForm
import app.indelible.auth.ui.ConnectServerScreen
import app.indelible.auth.ui.ForgotPasswordScreen
import app.indelible.auth.ui.LoginScreen
import app.indelible.auth.ui.RegisterScreen
import app.indelible.auth.ui.VerifyEmailScreen
import app.indelible.auth.viewmodel.AuthState
import app.indelible.auth.viewmodel.AuthViewModel
import app.indelible.auth.viewmodel.ConnectServerViewModel
import app.indelible.auth.viewmodel.ServerSetupState
import app.indelible.ui.components.AppStartupSplash

object AuthRoutes {
    const val CONNECT_SERVER = "connect_server"
    const val LOGIN = "login"
    const val REGISTER = "register"
    const val FORGOT_PASSWORD = "forgot_password"
    const val VERIFY_EMAIL = "verify_email/{email}"

    fun verifyEmail(email: String): String = "verify_email/$email"
}

@Composable
fun AuthNavigation(
    viewModel: AuthViewModel,
    connectServerViewModel: ConnectServerViewModel,
) {
    val serverSetup by connectServerViewModel.setupState.collectAsState()
    if (serverSetup is ServerSetupState.Unknown) {
        AppStartupSplash()
        return
    }

    val navController = rememberNavController()
    val authState by viewModel.authState.collectAsState()
    val setupRequired by viewModel.setupRequired.collectAsState()
    // Resolved once when setup leaves Unknown; later transitions navigate explicitly.
    val startDestination =
        remember {
            if (serverSetup is ServerSetupState.Required) AuthRoutes.CONNECT_SERVER else AuthRoutes.LOGIN
        }
    val serverHost =
        (serverSetup as? ServerSetupState.Configured)?.let { ServerUrlForm.displayHost(it.serverUrl) }

    val verificationEmail =
        when (val state = authState) {
            is AuthState.NeedsVerification -> state.user.email
            else -> null
        }

    LaunchedEffect(authState) {
        if (authState is AuthState.NeedsVerification) {
            val email = (authState as AuthState.NeedsVerification).user.email
            navController.navigate(AuthRoutes.verifyEmail(email)) {
                launchSingleTop = true
            }
        }
    }

    LaunchedEffect(setupRequired) {
        if (setupRequired) {
            navController.navigate(AuthRoutes.REGISTER) {
                launchSingleTop = true
            }
        }
    }

    NavHost(
        navController = navController,
        startDestination = startDestination,
    ) {
        composable(AuthRoutes.CONNECT_SERVER) {
            ConnectServerScreen(
                viewModel = connectServerViewModel,
                onConnected = {
                    viewModel.loadOAuthProviders()
                    navController.navigate(AuthRoutes.LOGIN) {
                        popUpTo(AuthRoutes.CONNECT_SERVER) { inclusive = true }
                        launchSingleTop = true
                    }
                },
            )
        }
        composable(AuthRoutes.LOGIN) {
            LoginScreen(
                viewModel = viewModel,
                onNavigateToRegister = {
                    viewModel.resetRegisterState()
                    navController.navigate(AuthRoutes.REGISTER)
                },
                onNavigateToForgotPassword = {
                    viewModel.resetForgotPasswordState()
                    navController.navigate(AuthRoutes.FORGOT_PASSWORD)
                },
                serverHost = serverHost,
                onChangeServer = {
                    navController.navigate(AuthRoutes.CONNECT_SERVER) {
                        launchSingleTop = true
                    }
                },
            )
        }
        composable(AuthRoutes.REGISTER) {
            RegisterScreen(
                viewModel = viewModel,
                onNavigateToLogin = {
                    viewModel.resetLoginState()
                    navController.popBackStack(AuthRoutes.LOGIN, inclusive = false)
                },
            )
        }
        composable(AuthRoutes.FORGOT_PASSWORD) {
            ForgotPasswordScreen(
                viewModel = viewModel,
                onNavigateToLogin = {
                    viewModel.resetLoginState()
                    navController.popBackStack(AuthRoutes.LOGIN, inclusive = false)
                },
            )
        }
        composable(AuthRoutes.VERIFY_EMAIL) {
            VerifyEmailScreen(
                viewModel = viewModel,
                email = verificationEmail ?: "",
            )
        }
    }
}
