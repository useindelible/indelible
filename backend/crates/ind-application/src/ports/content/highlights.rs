use super::*;

pub trait HighlightOperations: Send + Sync {
    fn update_highlight_color(
        &self,
        user_id: UserId,
        highlight_id: HighlightId,
        color: String,
    ) -> BoxFuture<'_, Result<Highlight, AppError>>;

    fn delete_highlight(
        &self,
        user_id: UserId,
        highlight_id: HighlightId,
    ) -> BoxFuture<'_, Result<(), AppError>>;

    fn upsert_note(
        &self,
        user_id: UserId,
        highlight_id: HighlightId,
        body: String,
    ) -> BoxFuture<'_, Result<HighlightNote, AppError>>;

    fn delete_note(
        &self,
        user_id: UserId,
        highlight_id: HighlightId,
    ) -> BoxFuture<'_, Result<(), AppError>>;

    fn list_recent_highlights(
        &self,
        user_id: UserId,
        limit: i64,
    ) -> BoxFuture<'_, Result<Vec<HighlightWithNote>, AppError>>;

    fn list_highlight_tags(
        &self,
        user_id: UserId,
        highlight_id: HighlightId,
    ) -> BoxFuture<'_, Result<Vec<Tag>, AppError>>;

    fn set_highlight_tags(
        &self,
        user_id: UserId,
        highlight_id: HighlightId,
        tag_names: Vec<String>,
    ) -> BoxFuture<'_, Result<Vec<Tag>, AppError>>;
}
