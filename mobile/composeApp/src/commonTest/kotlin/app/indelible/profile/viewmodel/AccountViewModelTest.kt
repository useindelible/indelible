package app.indelible.profile.viewmodel

import app.indelible.core.i18n.UiMessage
import app.indelible.profile.repository.AccountRepository
import indelible.composeapp.generated.resources.Res
import indelible.composeapp.generated.resources.profile_delete_failed
import kotlinx.coroutines.CoroutineStart
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.async
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.test.UnconfinedTestDispatcher
import kotlinx.coroutines.test.resetMain
import kotlinx.coroutines.test.runTest
import kotlinx.coroutines.test.setMain
import kotlin.test.AfterTest
import kotlin.test.BeforeTest
import kotlin.test.Test
import kotlin.test.assertEquals

@OptIn(ExperimentalCoroutinesApi::class)
class AccountViewModelTest {
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
    fun delete_failure_uses_operation_specific_message_without_exposing_throwable_text() =
        runTest(testDispatcher) {
            val viewModel = AccountViewModel(FakeAccountRepository(Result.failure(Exception("database details"))))
            val effect = async(start = CoroutineStart.UNDISPATCHED) { viewModel.effects.first() }

            viewModel.deleteAccount("DELETE")

            assertEquals(
                AccountEffect.ShowSnackbar(UiMessage(Res.string.profile_delete_failed)),
                effect.await(),
            )
        }

    private class FakeAccountRepository(
        private val deleteResult: Result<Unit>,
    ) : AccountRepository {
        override suspend fun deleteAccount(confirmation: String): Result<Unit> = deleteResult

        override suspend fun changePassword(
            currentPassword: String,
            newPassword: String,
        ): Result<Unit> = Result.success(Unit)
    }
}
