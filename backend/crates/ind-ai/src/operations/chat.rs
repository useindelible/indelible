use super::helpers::*;
use super::*;

impl MilaChatPort for MilaOperationsService {
    fn stream_chat(
        &self,
        user_id: UserId,
        request: MilaStreamRequest,
    ) -> BoxFuture<'_, Result<MilaStreamOutputStream, AppError>> {
        Box::pin(async move {
            ensure_mila_enabled(&self.service, user_id).await?;

            let stream = self
                .chat_service
                .stream_chat(MilaChatRequest {
                    user_id,
                    session_id: request.session_id,
                    question: request.question,
                    highlight_text: request.highlight_text,
                    highlight_offset: request.highlight_offset,
                })
                .await?;

            let mapped: MilaStreamOutputStream = Box::pin(stream.map(|result| {
                result.map(|delta| MilaStreamDeltaOutput {
                    delta: delta.content,
                    retrieval_degraded: delta.retrieval_degraded,
                })
            }));
            Ok(mapped)
        })
    }
}
