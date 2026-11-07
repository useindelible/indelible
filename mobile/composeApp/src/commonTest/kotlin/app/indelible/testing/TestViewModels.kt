package app.indelible.testing

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import kotlinx.coroutines.NonCancellable
import kotlinx.coroutines.cancelAndJoin
import kotlinx.coroutines.job
import kotlinx.coroutines.test.TestResult
import kotlinx.coroutines.test.TestScope
import kotlinx.coroutines.withContext

/** Joins a test's view models, so a late engine answer cannot resume on Main while the next test resets it. */
internal class TestViewModels {
    private val made = mutableListOf<ViewModel>()

    fun <T : ViewModel> keep(viewModel: T): T = viewModel.also(made::add)

    suspend fun join() =
        withContext(NonCancellable) {
            made.forEach { it.scopeJob().cancelAndJoin() }
            made.clear()
        }

    private fun ViewModel.scopeJob() = viewModelScope.coroutineContext.job

    fun runTest(body: suspend TestScope.() -> Unit): TestResult =
        kotlinx.coroutines.test.runTest {
            try {
                body()
            } finally {
                join()
            }
        }
}
