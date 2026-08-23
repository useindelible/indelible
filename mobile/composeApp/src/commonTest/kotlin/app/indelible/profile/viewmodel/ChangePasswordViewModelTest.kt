package app.indelible.profile.viewmodel

import app.indelible.core.i18n.UiMessage
import app.indelible.core.network.ApiException
import app.indelible.profile.repository.AccountRepository
import indelible.composeapp.generated.resources.Res
import indelible.composeapp.generated.resources.profile_password_change_failed
import indelible.composeapp.generated.resources.profile_password_changed
import indelible.composeapp.generated.resources.profile_password_incorrect
import kotlinx.coroutines.CoroutineStart
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.async
import kotlinx.coroutines.flow.take
import kotlinx.coroutines.flow.toList
import kotlinx.coroutines.test.UnconfinedTestDispatcher
import kotlinx.coroutines.test.resetMain
import kotlinx.coroutines.test.runTest
import kotlinx.coroutines.test.setMain
import kotlin.test.AfterTest
import kotlin.test.BeforeTest
import kotlin.test.Test
import kotlin.test.assertEquals

@OptIn(ExperimentalCoroutinesApi::class)
class ChangePasswordViewModelTest {
    private val testDispatcher = UnconfinedTestDispatcher()

    @BeforeTest
    fun setUp() {
        Dispatchers.setMain(testDispatcher)
    }

    @AfterTest
    fun tearDown() {
        Dispatchers.resetMain()
    }

    @Test
    fun success_emits_localized_confirmation_then_navigates_back() =
        runTest(testDispatcher) {
            val viewModel = ChangePasswordViewModel(FakeAccountRepository(Result.success(Unit)))
            val effects = async(start = CoroutineStart.UNDISPATCHED) { viewModel.effects.take(2).toList() }

            viewModel.changePassword("current", "new password")

            assertEquals(
                listOf(
                    ChangePasswordEffect.ShowSnackbar(UiMessage(Res.string.profile_password_changed)),
                    ChangePasswordEffect.NavigateBack,
                ),
                effects.await(),
            )
        }

    @Test
    fun unauthorized_failure_uses_specific_localized_message() =
        runTest(testDispatcher) {
            val viewModel =
                ChangePasswordViewModel(
                    FakeAccountRepository(Result.failure(ApiException(401, "server detail"))),
                )
            val effect = async(start = CoroutineStart.UNDISPATCHED) { viewModel.effects.take(1).toList() }

            viewModel.changePassword("wrong", "new password")

            assertEquals(
                listOf(ChangePasswordEffect.ShowSnackbar(UiMessage(Res.string.profile_password_incorrect))),
                effect.await(),
            )
        }

    @Test
    fun unknown_failure_uses_operation_specific_message_without_exposing_throwable_text() =
        runTest(testDispatcher) {
            val viewModel =
                ChangePasswordViewModel(
                    FakeAccountRepository(Result.failure(Exception("database details"))),
                )
            val effect = async(start = CoroutineStart.UNDISPATCHED) { viewModel.effects.take(1).toList() }

            viewModel.changePassword("current", "new password")

            assertEquals(
                listOf(ChangePasswordEffect.ShowSnackbar(UiMessage(Res.string.profile_password_change_failed))),
                effect.await(),
            )
        }

    private class FakeAccountRepository(
        private val changePasswordResult: Result<Unit>,
    ) : AccountRepository {
        override suspend fun deleteAccount(confirmation: String): Result<Unit> = Result.success(Unit)

        override suspend fun changePassword(
            currentPassword: String,
            newPassword: String,
        ): Result<Unit> = changePasswordResult
    }
}
