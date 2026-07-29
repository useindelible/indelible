package app.indelible.core.storage

import app.indelible.core.network.resolvedServerUrl
import app.indelible.share.model.PendingItem
import app.indelible.share.repository.JvmPendingSaveRepository
import kotlinx.coroutines.test.runTest
import java.util.Base64
import java.util.UUID
import java.util.prefs.AbstractPreferences
import java.util.prefs.Preferences
import kotlin.test.Test
import kotlin.test.assertEquals

class JvmTokenStorageTest {
    @Test
    fun clearAllRemovesPersistentQueuedItems() =
        runTest {
            val preferences = Preferences.userRoot().node("indelible-test-${UUID.randomUUID()}")
            try {
                val tokenState = InMemoryTokenStorage()
                signIn(tokenState, "account-a")
                val repository = JvmPendingSaveRepository(preferences) { tokenState.pendingQueueOwner() }
                val tokenStorage = JvmTokenStorage(tokenState, repository)
                repository.enqueue(PendingItem("one", "https://example.com", 1L))

                tokenStorage.clearAll()

                val restartedTokenState = InMemoryTokenStorage()
                signIn(restartedTokenState, "account-a")

                assertEquals(
                    emptyList(),
                    JvmPendingSaveRepository(preferences) { restartedTokenState.pendingQueueOwner() }.drainAll(),
                )
            } finally {
                runCatching { preferences.removeNode() }
            }
        }

    @Test
    fun failedQueueRemovalDoesNotCrossSubjectsOnSameCanonicalServerAfterRestart() =
        runTest {
            assertFailedLogoutDoesNotCrossOwner(
                accountASubject = "account-a",
                accountAServerUrl = "https://library.useindelible.test",
                accountBSubject = "account-b",
                accountBServerUrl = "https://library.useindelible.test",
            )
        }

    @Test
    fun failedQueueRemovalDoesNotCrossServersForSameSubjectAfterRestart() =
        runTest {
            assertFailedLogoutDoesNotCrossOwner(
                accountASubject = "account-a",
                accountAServerUrl = "https://library.useindelible.test",
                accountBSubject = "account-a",
                accountBServerUrl = "https://other-library.useindelible.test",
            )
        }

    @Test
    fun pendingQueueOwnerUsesResolvedServerFallbackWhenNoServerIsStored() =
        runTest {
            val tokenState = InMemoryTokenStorage()
            tokenState.saveToken(jwt("account-a"))

            assertEquals(tokenState.resolvedServerUrl(), tokenState.pendingQueueOwner()?.serverUrl)
        }

    @Test
    fun pendingQueueOwnerAcceptsBackendAccessTokenClaims() =
        runTest {
            val tokenState = InMemoryTokenStorage()
            tokenState.saveServerUrl("https://library.useindelible.test")
            tokenState.saveToken(accessTokenJwt("account-a"))

            assertEquals("account-a", tokenState.pendingQueueOwner()?.userId)
        }

    @Test
    fun malformedJwtDoesNotDrainPendingItems() =
        runTest {
            val preferences = Preferences.userRoot().node("indelible-test-${UUID.randomUUID()}")
            try {
                val tokenState = InMemoryTokenStorage()
                tokenState.saveServerUrl("https://library.useindelible.test")
                tokenState.saveToken("not-a-jwt")
                val repository = JvmPendingSaveRepository(preferences) { tokenState.pendingQueueOwner() }

                repository.enqueue(PendingItem("one", "https://example.com", 1L))

                assertEquals(emptyList(), repository.drainAll())
            } finally {
                runCatching { preferences.removeNode() }
            }
        }

    private class FailingRemovePreferences : AbstractPreferences(null, "") {
        private val values = mutableMapOf<String, String>()
        var failRemovals = false

        override fun putSpi(
            key: String,
            value: String,
        ) {
            values[key] = value
        }

        override fun getSpi(key: String): String? = values[key]

        override fun removeSpi(key: String) {
            if (failRemovals) {
                throw IllegalStateException("pending queue removal unavailable")
            }
            values.remove(key)
        }

        override fun keysSpi(): Array<String> = values.keys.toTypedArray()

        override fun childrenNamesSpi(): Array<String> = emptyArray()

        override fun childSpi(name: String): AbstractPreferences =
            throw UnsupportedOperationException("child nodes are not used")

        override fun removeNodeSpi() {
            values.clear()
        }

        override fun syncSpi() = Unit

        override fun flushSpi() = Unit
    }

    private suspend fun assertFailedLogoutDoesNotCrossOwner(
        accountASubject: String,
        accountAServerUrl: String,
        accountBSubject: String,
        accountBServerUrl: String,
    ) {
        val preferences = FailingRemovePreferences()
        val accountATokenState = InMemoryTokenStorage()
        signIn(accountATokenState, accountASubject, accountAServerUrl)
        val accountARepository =
            JvmPendingSaveRepository(preferences) { accountATokenState.pendingQueueOwner() }
        val accountATokenStorage = JvmTokenStorage(accountATokenState, accountARepository)
        accountARepository.enqueue(PendingItem("account-a", "https://example.com/a", 1L))
        preferences.failRemovals = true

        accountATokenStorage.clearAll()

        val accountBTokenState = InMemoryTokenStorage()
        val accountBRepository =
            JvmPendingSaveRepository(preferences) { accountBTokenState.pendingQueueOwner() }
        val accountBTokenStorage = JvmTokenStorage(accountBTokenState, accountBRepository)
        signIn(accountBTokenStorage, accountBSubject, accountBServerUrl)
        val accountBItem = PendingItem("account-b", "https://example.com/b", 1L)

        accountBRepository.enqueue(accountBItem)

        assertEquals(listOf(accountBItem), accountBRepository.drainAll())
    }

    private suspend fun signIn(
        tokenStorage: TokenStorage,
        subject: String,
        serverUrl: String = "https://library.useindelible.test",
    ) {
        tokenStorage.saveServerUrl(serverUrl)
        tokenStorage.saveToken(jwt(subject))
    }

    private fun jwt(subject: String): String {
        val payload = Base64.getUrlEncoder().withoutPadding().encodeToString("{\"sub\":\"$subject\"}".encodeToByteArray())
        return "header.$payload.signature"
    }

    private fun accessTokenJwt(subject: String): String {
        val claims =
            """{"sub":"$subject","ct":"desktop","jti":"01990ef0-aed7-7000-8000-000000000001","iat":1786320000,"exp":1786320900}"""
        val payload = Base64.getUrlEncoder().withoutPadding().encodeToString(claims.encodeToByteArray())
        return "header.$payload.signature"
    }
}
