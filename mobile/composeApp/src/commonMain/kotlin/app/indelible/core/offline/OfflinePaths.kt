package app.indelible.core.offline

import app.indelible.auth.oauth.sha256
import okio.Path

// The scope holds a server URL and a user id; hashing keeps both out of the file system and
// gives every scope a fixed-length, path-safe directory name.
fun scopeDirName(scope: String): String = sha256(scope.encodeToByteArray()).toHexString()

fun OfflineFiles.scopeDir(scope: String): Path = root / scopeDirName(scope)

fun OfflineFiles.documentDir(
    scope: String,
    documentId: String,
): Path = scopeDir(scope) / documentId

fun OfflineFiles.generationDir(
    scope: String,
    documentId: String,
    generation: Long,
): Path = documentDir(scope, documentId) / generation.toString()
