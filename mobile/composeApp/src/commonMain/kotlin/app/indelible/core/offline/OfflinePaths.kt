package app.indelible.core.offline

import app.indelible.auth.oauth.sha256
import okio.Path

// Hashing keeps the scope's server URL and user id out of the file system and fixes the directory name's length.
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
