package app.indelible.reader.repository

import app.indelible.core.offline.Acquisition
import app.indelible.core.offline.DownloadFailure
import app.indelible.core.offline.OfflineFiles
import app.indelible.core.offline.OfflineStore
import app.indelible.core.offline.PDF_KINDS
import app.indelible.core.offline.ReadingCopies
import app.indelible.core.offline.Session
import app.indelible.core.offline.generationDir
import kotlinx.coroutines.NonCancellable
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.FlowCollector
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.distinctUntilChanged
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.flow.flow
import kotlinx.coroutines.flow.transformWhile
import kotlinx.coroutines.withContext
import okio.IOException

/** Where the PDF the reader wants stands; [Failed] and [Removed] end the flow. */
sealed interface PdfCopy {
    data class Downloading(
        val fraction: Float?,
    ) : PdfCopy

    data class Ready(
        val path: String,
    ) : PdfCopy

    data class Failed(
        val reason: PdfCopyFailure,
    ) : PdfCopy

    /** The copy disappeared after it was ready. */
    data object Removed : PdfCopy
}

enum class PdfCopyFailure { CONNECTION, NO_SPACE, SERVER }

/** What the reader needs from the download layer to wait for a copy. */
internal class PdfCopySource(
    private val store: OfflineStore,
    private val files: OfflineFiles,
    private val copies: ReadingCopies,
) {
    /** The copy's PDF, whole at its recorded size, or null. */
    suspend fun pdfFile(
        scope: String,
        documentId: String,
    ): String? {
        val copy = store.cachedDocument(scope, documentId) ?: return null
        val asset = store.assetsForDocument(scope, documentId).firstOrNull { it.kind in PDF_KINDS }
        val path = asset?.let { files.generationDir(scope, documentId, copy.generation) / it.path }
        val whole =
            try {
                path != null && files.size(path) == asset.bytes
            } catch (ignored: IOException) {
                false
            }
        return path?.toString()?.takeIf { whole }
    }

    /** Holds the document while collected; without a copy it asks for a reading download and follows it. */
    fun pdfCopy(
        session: () -> Session?,
        documentId: String,
    ): Flow<PdfCopy> =
        flow {
            val live = session() ?: return@flow emit(PdfCopy.Failed(PdfCopyFailure.CONNECTION))
            val scope = live.scope
            copies.hold(scope, documentId)
            try {
                val path = pdfFile(scope, documentId) ?: awaitCopy(live, documentId)
                if (path != null) {
                    emit(PdfCopy.Ready(path))
                    store.observeCatalog(scope).first { rows -> rows.none { it.documentId == documentId } }
                    emit(PdfCopy.Removed)
                }
            } finally {
                withContext(NonCancellable) { copies.release(scope, documentId) }
            }
        }

    // The path once a whole copy exists; null after the failure that ended the request was emitted.
    private suspend fun FlowCollector<PdfCopy>.awaitCopy(
        session: Session,
        documentId: String,
    ): String? {
        val scope = session.scope
        copies.requestForReading(session, documentId)
        var path: String? = null
        combine(store.observeCatalog(scope), copies.observeAcquisitions(scope)) { _, all -> all[documentId] }
            .distinctUntilChanged()
            .transformWhile { acquisition ->
                val file = pdfFile(scope, documentId)
                when {
                    file != null -> false.also { path = file }
                    acquisition == null -> false.also { emit(PdfCopy.Failed(PdfCopyFailure.SERVER)) }
                    else -> (acquisition !is Acquisition.Failed).also { emit(acquisition.toPdfCopy()) }
                }
            }.collect { emit(it) }
        return path
    }
}

private fun Acquisition.toPdfCopy(): PdfCopy =
    when (this) {
        is Acquisition.Downloading -> PdfCopy.Downloading(fraction)
        Acquisition.Waiting, Acquisition.WaitingForConnection -> PdfCopy.Downloading(null)
        is Acquisition.Failed ->
            PdfCopy.Failed(
                when (reason) {
                    DownloadFailure.NO_SPACE -> PdfCopyFailure.NO_SPACE
                    DownloadFailure.NETWORK -> PdfCopyFailure.CONNECTION
                    DownloadFailure.SERVER, DownloadFailure.SERVER_TOO_OLD -> PdfCopyFailure.SERVER
                },
            )
    }
