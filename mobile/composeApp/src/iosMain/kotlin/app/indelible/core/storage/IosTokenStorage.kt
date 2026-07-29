package app.indelible.core.storage

import app.indelible.auth.oauth.PendingOAuthFlow
import kotlinx.cinterop.COpaquePointer
import kotlinx.cinterop.ExperimentalForeignApi
import kotlinx.cinterop.alloc
import kotlinx.cinterop.memScoped
import kotlinx.cinterop.ptr
import kotlinx.cinterop.value
import kotlinx.serialization.json.Json
import platform.CoreFoundation.CFDictionaryCreateMutable
import platform.CoreFoundation.CFDictionaryRef
import platform.CoreFoundation.CFDictionarySetValue
import platform.CoreFoundation.CFMutableDictionaryRef
import platform.CoreFoundation.CFRelease
import platform.CoreFoundation.CFTypeRefVar
import platform.CoreFoundation.kCFBooleanTrue
import platform.CoreFoundation.kCFTypeDictionaryKeyCallBacks
import platform.CoreFoundation.kCFTypeDictionaryValueCallBacks
import platform.Foundation.CFBridgingRelease
import platform.Foundation.CFBridgingRetain
import platform.Foundation.NSData
import platform.Foundation.NSString
import platform.Foundation.NSUTF8StringEncoding
import platform.Foundation.NSUserDefaults
import platform.Foundation.create
import platform.Foundation.dataUsingEncoding
import platform.Security.SecItemAdd
import platform.Security.SecItemCopyMatching
import platform.Security.SecItemDelete
import platform.Security.kSecAttrAccount
import platform.Security.kSecAttrService
import platform.Security.kSecClass
import platform.Security.kSecClassGenericPassword
import platform.Security.kSecMatchLimit
import platform.Security.kSecMatchLimitOne
import platform.Security.kSecReturnData
import platform.Security.kSecValueData
import platform.darwin.noErr

@OptIn(ExperimentalForeignApi::class)
class IosTokenStorage : TokenStorage {
    override suspend fun saveToken(token: String) {
        saveToKeychain(KEY_TOKEN, token)
    }

    override suspend fun getToken(): String? = readFromKeychain(KEY_TOKEN)

    override suspend fun clearToken() {
        deleteFromKeychain(KEY_TOKEN)
    }

    // App Group suite is required so the share extension process can read the server URL.
    // standardUserDefaults is sandboxed per-process and invisible to the extension.
    private val appGroupDefaults = NSUserDefaults(suiteName = APP_GROUP)

    override suspend fun saveServerUrl(url: String) {
        appGroupDefaults?.setObject(url, KEY_SERVER_URL)
    }

    override suspend fun getServerUrl(): String? = appGroupDefaults?.stringForKey(KEY_SERVER_URL)

    override suspend fun saveRefreshToken(token: String) {
        saveToKeychain(KEY_REFRESH_TOKEN, token)
    }

    override suspend fun getRefreshToken(): String? = readFromKeychain(KEY_REFRESH_TOKEN)

    override suspend fun saveExpiresAt(epochSeconds: Long) {
        saveToKeychain(KEY_EXPIRES_AT, epochSeconds.toString())
    }

    override suspend fun getExpiresAt(): Long? = readFromKeychain(KEY_EXPIRES_AT)?.toLongOrNull()

    override suspend fun savePendingOAuthFlow(flow: PendingOAuthFlow) {
        saveToKeychain(KEY_PENDING_OAUTH, Json.encodeToString(flow))
    }

    override suspend fun getPendingOAuthFlow(): PendingOAuthFlow? =
        readFromKeychain(KEY_PENDING_OAUTH)?.let {
            runCatching { Json.decodeFromString<PendingOAuthFlow>(it) }.getOrNull()
        }

    override suspend fun clearPendingOAuthFlow() {
        deleteFromKeychain(KEY_PENDING_OAUTH)
    }

    override suspend fun clearAll() {
        deleteFromKeychain(KEY_TOKEN)
        deleteFromKeychain(KEY_REFRESH_TOKEN)
        deleteFromKeychain(KEY_EXPIRES_AT)
        deleteFromKeychain(KEY_PENDING_OAUTH)
        appGroupDefaults?.removeObjectForKey(KEY_PENDING_ITEMS)
    }

    // Build a CFDictionary directly from CF key/value pairs.
    // Avoids Kotlin Map -> NSDictionary bridging which corrupts CFStringRef keys
    // by boxing them as opaque pointers instead of toll-free bridged NSStrings.
    private fun cfDictionary(block: CFMutableDictionaryRef.() -> Unit): CFDictionaryRef? {
        val dict =
            CFDictionaryCreateMutable(
                null,
                0,
                kCFTypeDictionaryKeyCallBacks?.ptr,
                kCFTypeDictionaryValueCallBacks?.ptr,
            ) ?: return null
        dict.block()
        return dict
    }

    private fun CFMutableDictionaryRef.set(
        key: COpaquePointer?,
        value: COpaquePointer?,
    ) {
        CFDictionarySetValue(this, key, value)
    }

    private fun CFMutableDictionaryRef.set(
        key: COpaquePointer?,
        value: String,
    ) {
        val cfValue = CFBridgingRetain(value)
        CFDictionarySetValue(this, key, cfValue)
    }

    // kSecAttrAccessGroup is intentionally omitted from keychain queries.
    // The main app uses its default keychain access group which always works.
    // Share extension keychain sharing requires matching keychain-access-groups
    // entitlements with a valid team ID — that is configured separately in
    // ShareExtensionBridge when provisioning is set up.

    private fun saveToKeychain(
        key: String,
        value: String,
    ) {
        deleteFromKeychain(key)
        val data = (value as NSString).dataUsingEncoding(NSUTF8StringEncoding) ?: return
        val cfData = CFBridgingRetain(data)
        val query =
            cfDictionary {
                set(kSecClass, kSecClassGenericPassword)
                set(kSecAttrService, SERVICE_NAME)
                set(kSecAttrAccount, key)
                set(kSecValueData, cfData)
            }
        SecItemAdd(query, null)
        if (query != null) CFRelease(query)
    }

    private fun readFromKeychain(key: String): String? =
        memScoped {
            val query =
                cfDictionary {
                    set(kSecClass, kSecClassGenericPassword)
                    set(kSecAttrService, SERVICE_NAME)
                    set(kSecAttrAccount, key)
                    set(kSecReturnData, kCFBooleanTrue)
                    set(kSecMatchLimit, kSecMatchLimitOne)
                }
            val result = alloc<CFTypeRefVar>()
            val status = SecItemCopyMatching(query, result.ptr)
            if (query != null) CFRelease(query)
            if (status != noErr.toInt()) return null
            val data = CFBridgingRelease(result.value) as? NSData ?: return null
            NSString.create(data, NSUTF8StringEncoding) as? String
        }

    private fun deleteFromKeychain(key: String) {
        val query =
            cfDictionary {
                set(kSecClass, kSecClassGenericPassword)
                set(kSecAttrService, SERVICE_NAME)
                set(kSecAttrAccount, key)
            }
        SecItemDelete(query)
        if (query != null) CFRelease(query)
    }

    companion object {
        private const val SERVICE_NAME = "com.useindelible.app"
        private const val APP_GROUP = "group.com.useindelible.app"
        private const val KEY_TOKEN = "auth_token"
        private const val KEY_REFRESH_TOKEN = "refresh_token"
        private const val KEY_EXPIRES_AT = "expires_at"
        private const val KEY_SERVER_URL = "server_url"
        private const val KEY_PENDING_OAUTH = "pending_oauth_flow"
        private const val KEY_PENDING_ITEMS = "pending_items"
    }
}
