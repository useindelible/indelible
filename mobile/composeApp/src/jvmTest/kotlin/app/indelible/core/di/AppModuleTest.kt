package app.indelible.core.di

import app.indelible.auth.oauth.NoopOAuthBrowserLauncher
import app.indelible.core.offline.ConnectivityObserver
import app.indelible.core.offline.OfflineFilesRoot
import app.indelible.core.storage.InMemoryTokenStorage
import app.indelible.core.storage.InMemoryUserPreferencesStorage
import app.indelible.db.DatabaseDriverFactory
import app.indelible.home.repository.HomeRepository
import app.indelible.library.repository.LibraryRepository
import app.indelible.reader.repository.ReaderRepository
import app.indelible.share.FakePendingSaveRepository
import org.koin.dsl.koinApplication
import java.nio.file.Files
import kotlin.test.Test
import kotlin.test.assertNotNull

class AppModuleTest {
    @Test
    fun theRepositoriesThatKeepListsResolve() {
        val directory = Files.createTempDirectory("indelible-di").toFile()
        val database = DatabaseDriverFactory(directory)
        val deps =
            AppDependencies(
                InMemoryTokenStorage(),
                InMemoryUserPreferencesStorage(),
                FakePendingSaveRepository(),
                database,
                ConnectivityObserver(),
                OfflineFilesRoot(directory),
            )
        val app = koinApplication { modules(appModule(deps, NoopOAuthBrowserLauncher) {}) }
        try {
            assertNotNull(app.koin.get<HomeRepository>())
            assertNotNull(app.koin.get<LibraryRepository>())
            assertNotNull(app.koin.get<ReaderRepository>())
        } finally {
            app.close()
            database.close()
            directory.deleteRecursively()
        }
    }
}
