use super::*;

impl DocumentReaderOperations for DocumentReaderService {
    fn get_reader(
        &self,
        user_id: UserId,
        document_id: DocumentId,
    ) -> BoxFuture<'_, Result<DocumentReaderView, AppError>> {
        Box::pin(self.get_reader(user_id, document_id))
    }

    fn get_completed_asset(
        &self,
        user_id: UserId,
        document_id: DocumentId,
        kind: ArchiveAssetKind,
    ) -> BoxFuture<'_, Result<DocumentAsset, AppError>> {
        Box::pin(self.get_completed_asset(user_id, document_id, kind))
    }

    fn get_epub_toc_asset(
        &self,
        user_id: UserId,
        document_id: DocumentId,
    ) -> BoxFuture<'_, Result<EpubTocAsset, AppError>> {
        Box::pin(self.get_epub_toc_asset(user_id, document_id))
    }

    fn reprocess_document(
        &self,
        user_id: UserId,
        document_id: DocumentId,
    ) -> BoxFuture<'_, Result<DocumentReprocessOutput, AppError>> {
        Box::pin(self.reprocess_document(user_id, document_id))
    }

    fn create_highlight(
        &self,
        user_id: UserId,
        document_id: DocumentId,
        request: CreateHighlightRequest,
    ) -> BoxFuture<'_, Result<HighlightCreation, AppError>> {
        Box::pin(self.create_highlight(user_id, document_id, request))
    }

    fn list_highlights(
        &self,
        user_id: UserId,
        document_id: DocumentId,
    ) -> BoxFuture<'_, Result<Vec<HighlightWithNote>, AppError>> {
        Box::pin(self.list_highlights(user_id, document_id))
    }

    fn get_note(
        &self,
        user_id: UserId,
        document_id: DocumentId,
    ) -> BoxFuture<'_, Result<Option<DocumentNote>, AppError>> {
        Box::pin(self.get_note(user_id, document_id))
    }

    fn upsert_note(
        &self,
        user_id: UserId,
        document_id: DocumentId,
        body: String,
        base: Option<DateTime<Utc>>,
    ) -> BoxFuture<'_, Result<NoteWrite, AppError>> {
        Box::pin(self.upsert_note(user_id, document_id, body, base))
    }

    fn update_progress(
        &self,
        user_id: UserId,
        document_id: DocumentId,
        progress_percent: i32,
        position: Option<ReadingPosition>,
        origin: EventOrigin,
    ) -> BoxFuture<'_, Result<UserDocumentState, AppError>> {
        Box::pin(self.update_progress(user_id, document_id, progress_percent, position, origin))
    }

    fn append_reading_events(
        &self,
        user_id: UserId,
        document_id: DocumentId,
        events: Vec<NewReadingEvent>,
    ) -> BoxFuture<'_, Result<AppendOutcome, AppError>> {
        Box::pin(self.append_reading_events(user_id, document_id, events))
    }

    fn mark_unread(
        &self,
        user_id: UserId,
        document_id: DocumentId,
    ) -> BoxFuture<'_, Result<(), AppError>> {
        Box::pin(self.mark_unread(user_id, document_id))
    }
}
