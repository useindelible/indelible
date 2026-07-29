package app.indelible.api

import app.indelible.core.network.ApiClient
import app.indelible.core.storage.InMemoryTokenStorage
import io.ktor.client.engine.mock.MockEngine
import io.ktor.client.engine.mock.respond
import io.ktor.http.HttpHeaders
import io.ktor.http.HttpMethod
import io.ktor.http.HttpStatusCode
import io.ktor.http.headersOf
import kotlinx.coroutines.test.runTest
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertTrue

class SettingsParityTest {
    private val jsonHeaders = headersOf(HttpHeaders.ContentType, "application/json")

    @Test
    fun getArchivalSettingsSendsGet() =
        runTest {
            val tokenStorage = InMemoryTokenStorage()
            tokenStorage.saveToken("test-token")
            tokenStorage.saveExpiresAt(FAR_FUTURE_EXPIRY)
            var capturedMethod: HttpMethod? = null
            var capturedPath: String? = null
            val engine =
                MockEngine { request ->
                    capturedMethod = request.method
                    capturedPath = request.url.encodedPath
                    respond(archivalSettingsJson(), HttpStatusCode.OK, jsonHeaders)
                }

            val apiClient = ApiClient(tokenStorage, engine = engine)
            val result = apiClient.getArchivalSettings()

            assertEquals(HttpMethod.Get, capturedMethod)
            assertEquals("/api/v1/settings/archival", capturedPath)
            assertTrue(result.isSuccess)
        }

    @Test
    fun updateArchivalSettingsSendsPatch() =
        runTest {
            val tokenStorage = InMemoryTokenStorage()
            tokenStorage.saveToken("test-token")
            tokenStorage.saveExpiresAt(FAR_FUTURE_EXPIRY)
            var capturedMethod: HttpMethod? = null
            var capturedPath: String? = null
            val engine =
                MockEngine { request ->
                    capturedMethod = request.method
                    capturedPath = request.url.encodedPath
                    respond(archivalSettingsJson(), HttpStatusCode.OK, jsonHeaders)
                }

            val apiClient = ApiClient(tokenStorage, engine = engine)
            val settings = apiClient.getArchivalSettings().getOrThrow()
            val updateEngine =
                MockEngine { request ->
                    capturedMethod = request.method
                    capturedPath = request.url.encodedPath
                    respond(archivalSettingsJson(), HttpStatusCode.OK, jsonHeaders)
                }
            val updateClient = ApiClient(tokenStorage, engine = updateEngine)
            val result = updateClient.updateArchivalSettings(settings)

            assertEquals(HttpMethod.Patch, capturedMethod)
            assertEquals("/api/v1/settings/archival", capturedPath)
            assertTrue(result.isSuccess)
        }

    @Test
    fun getNotificationsSettingsSendsGet() =
        runTest {
            val tokenStorage = InMemoryTokenStorage()
            tokenStorage.saveToken("test-token")
            tokenStorage.saveExpiresAt(FAR_FUTURE_EXPIRY)
            var capturedMethod: HttpMethod? = null
            var capturedPath: String? = null
            val engine =
                MockEngine { request ->
                    capturedMethod = request.method
                    capturedPath = request.url.encodedPath
                    respond(notificationsSettingsJson(), HttpStatusCode.OK, jsonHeaders)
                }

            val apiClient = ApiClient(tokenStorage, engine = engine)
            val result = apiClient.getNotificationsSettings()

            assertEquals(HttpMethod.Get, capturedMethod)
            assertEquals("/api/v1/settings/notifications", capturedPath)
            assertTrue(result.isSuccess)
        }

    @Test
    fun updateNotificationsSettingsSendsPatch() =
        runTest {
            val tokenStorage = InMemoryTokenStorage()
            tokenStorage.saveToken("test-token")
            tokenStorage.saveExpiresAt(FAR_FUTURE_EXPIRY)
            var capturedMethod: HttpMethod? = null
            var capturedPath: String? = null
            val engine =
                MockEngine { request ->
                    capturedMethod = request.method
                    capturedPath = request.url.encodedPath
                    respond(notificationsSettingsJson(), HttpStatusCode.OK, jsonHeaders)
                }

            val apiClient = ApiClient(tokenStorage, engine = engine)
            val settings = apiClient.getNotificationsSettings().getOrThrow()
            val updateEngine =
                MockEngine { request ->
                    capturedMethod = request.method
                    capturedPath = request.url.encodedPath
                    respond(notificationsSettingsJson(), HttpStatusCode.OK, jsonHeaders)
                }
            val updateClient = ApiClient(tokenStorage, engine = updateEngine)
            val result = updateClient.updateNotificationsSettings(settings)

            assertEquals(HttpMethod.Patch, capturedMethod)
            assertEquals("/api/v1/settings/notifications", capturedPath)
            assertTrue(result.isSuccess)
        }

    @Test
    fun getPreferencesSendsGet() =
        runTest {
            val tokenStorage = InMemoryTokenStorage()
            tokenStorage.saveToken("test-token")
            tokenStorage.saveExpiresAt(FAR_FUTURE_EXPIRY)
            var capturedMethod: HttpMethod? = null
            var capturedPath: String? = null
            val engine =
                MockEngine { request ->
                    capturedMethod = request.method
                    capturedPath = request.url.encodedPath
                    respond(preferencesJson(), HttpStatusCode.OK, jsonHeaders)
                }

            val apiClient = ApiClient(tokenStorage, engine = engine)
            val result = apiClient.getPreferences()

            assertEquals(HttpMethod.Get, capturedMethod)
            assertEquals("/api/v1/settings/preferences", capturedPath)
            assertTrue(result.isSuccess)
        }

    @Test
    fun updatePreferencesSendsPatch() =
        runTest {
            val tokenStorage = InMemoryTokenStorage()
            tokenStorage.saveToken("test-token")
            tokenStorage.saveExpiresAt(FAR_FUTURE_EXPIRY)
            var capturedMethod: HttpMethod? = null
            var capturedPath: String? = null
            val engine =
                MockEngine { request ->
                    capturedMethod = request.method
                    capturedPath = request.url.encodedPath
                    respond(preferencesJson(), HttpStatusCode.OK, jsonHeaders)
                }

            val apiClient = ApiClient(tokenStorage, engine = engine)
            val prefs = apiClient.getPreferences().getOrThrow()
            val updateEngine =
                MockEngine { request ->
                    capturedMethod = request.method
                    capturedPath = request.url.encodedPath
                    respond(preferencesJson(), HttpStatusCode.OK, jsonHeaders)
                }
            val updateClient = ApiClient(tokenStorage, engine = updateEngine)
            val result = updateClient.updatePreferences(prefs)

            assertEquals(HttpMethod.Patch, capturedMethod)
            assertEquals("/api/v1/settings/preferences", capturedPath)
            assertTrue(result.isSuccess)
        }

    private fun archivalSettingsJson() =
        """
        {
            "archive_formats": {
                "monolith": true,
                "pdf": false,
                "readable_html": true,
                "screenshot": false,
                "warc": false
            },
            "duplicate_detection": {
                "enabled": true,
                "on_duplicate": "notify_me",
                "sensitivity": "medium"
            },
            "processing": {
                "ai_auto_processing": true,
                "browser_timeout_secs": 30,
                "max_concurrent_archives": 5
            },
            "proxy": {
                "all_requests": false,
                "url": null
            }
        }
        """.trimIndent()

    private fun notificationsSettingsJson() =
        """
        {
            "daily_review_reminder_enabled": false,
            "daily_review_reminder_time": "09:00",
            "feed_updates": true,
            "marketing_emails": false,
            "new_highlights_sync": true,
            "updated_at": "2026-01-01T00:00:00Z",
            "weekly_digest_enabled": false
        }
        """.trimIndent()

    private fun preferencesJson() =
        """
        {
            "ai": {
                "mila_enabled": true,
                "custom_prompt": null
            },
            "appearance": {
                "accent_color": "blue"
            },
            "layout": {
                "default_view": "library",
                "list_density": "comfortable",
                "side_panel": "auto",
                "sidebar_mode": "expanded"
            },
            "reader": {
                "email_open_mode": "reader",
                "font_family": "sans",
                "font_size": "medium",
                "line_height": "relaxed"
            },
            "theme": "system",
            "workflow": {
                "auto_advance": false,
                "triage_mode": "manual"
            }
        }
        """.trimIndent()

    companion object {
        private const val FAR_FUTURE_EXPIRY = 4_102_444_800L
    }
}
