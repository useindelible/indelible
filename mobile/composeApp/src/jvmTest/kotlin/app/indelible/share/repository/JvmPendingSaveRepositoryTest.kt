package app.indelible.share.repository

import app.indelible.core.storage.InMemoryTokenStorage
import app.indelible.core.storage.pendingQueueOwner
import app.indelible.share.model.PendingItem
import kotlinx.coroutines.test.runTest
import java.util.UUID
import java.util.prefs.Preferences
import kotlin.test.Test
import kotlin.test.assertEquals

class JvmPendingSaveRepositoryTest {
    @Test
    fun persistsQueuedItemsAcrossRepositoryInstances() =
        runTest {
            withIsolatedPreferences { preferences ->
                val firstTokenState = tokenState("account-a")
                val item = item("one")

                JvmPendingSaveRepository(preferences) { firstTokenState.pendingQueueOwner() }.enqueue(item)

                val restartedTokenState = tokenState("account-a")

                assertEquals(
                    listOf(item),
                    JvmPendingSaveRepository(preferences) { restartedTokenState.pendingQueueOwner() }.drainAll(),
                )
            }
        }

    @Test
    fun persistsAcrossEquivalentServerUrlSpellings() =
        runTest {
            withIsolatedPreferences { preferences ->
                val firstTokenState = tokenState("account-a", "  https://library.useindelible.test/  ")
                val item = item("one")

                JvmPendingSaveRepository(preferences) { firstTokenState.pendingQueueOwner() }.enqueue(item)

                val restartedTokenState = tokenState("account-a", "https://library.useindelible.test")

                assertEquals(
                    listOf(item),
                    JvmPendingSaveRepository(preferences) { restartedTokenState.pendingQueueOwner() }.drainAll(),
                )
            }
        }

    @Test
    fun drainAllDeduplicatesItemsAndClearsTheQueue() =
        runTest {
            withIsolatedPreferences { preferences ->
                val repository = repository(preferences)
                val first = item("one")
                val second = item("two")

                repository.enqueue(first)
                repository.enqueue(first)
                repository.enqueue(second)

                assertEquals(listOf(first, second), repository.drainAll())
                assertEquals(emptyList(), repository.drainAll())
            }
        }

    @Test
    fun removeLeavesOtherQueuedItems() =
        runTest {
            withIsolatedPreferences { preferences ->
                val repository = repository(preferences)
                val first = item("one")
                val second = item("two")

                repository.enqueue(first)
                repository.enqueue(second)
                repository.remove(first.id)

                assertEquals(listOf(second), repository.drainAll())
            }
        }

    @Test
    fun enqueueLimitsTheQueueToFiftyItems() =
        runTest {
            withIsolatedPreferences { preferences ->
                val repository = repository(preferences)

                repeat(51) { repository.enqueue(item(it.toString())) }

                assertEquals(50, repository.drainAll().size)
            }
        }

    @Test
    fun unavailablePreferencesNodeDoesNotSurfaceStorageFailures() =
        runTest {
            val preferences = isolatedPreferences()
            preferences.removeNode()
            val repository = repository(preferences)

            repository.enqueue(item("one"))

            assertEquals(emptyList(), repository.drainAll())
        }

    private fun isolatedPreferences(): Preferences =
        Preferences.userRoot().node("indelible-test-${UUID.randomUUID()}")

    private suspend fun withIsolatedPreferences(block: suspend (Preferences) -> Unit) {
        val preferences = isolatedPreferences()
        try {
            block(preferences)
        } finally {
            runCatching { preferences.removeNode() }
        }
    }

    private fun item(id: String) =
        PendingItem(
            id = id,
            url = "https://example.com/$id",
            enqueuedAt = 1L,
        )

    private suspend fun repository(preferences: Preferences): JvmPendingSaveRepository {
        val tokenState = tokenState("account-a")
        return JvmPendingSaveRepository(preferences) { tokenState.pendingQueueOwner() }
    }

    private suspend fun tokenState(
        subject: String,
        serverUrl: String = "https://library.useindelible.test",
    ): InMemoryTokenStorage =
        InMemoryTokenStorage().also {
            it.saveServerUrl(serverUrl)
            it.saveToken(jwt(subject))
        }

    private fun jwt(subject: String) =
        "header.${java.util.Base64.getUrlEncoder().withoutPadding().encodeToString("{\"sub\":\"$subject\"}".encodeToByteArray())}.signature"
}
