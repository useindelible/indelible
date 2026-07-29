#[tokio::test]
async fn collection_fallbacks_emit_precise_warnings_and_keep_dual_scope() {
    for (embedding, vector, fts, warning) in [
        (true, false, false, "embedding_failed"),
        (false, true, false, "vector_failed"),
        (false, false, true, "fts_failed"),
    ] {
        let h = ChatHarness::new(MilaSessionType::Collection);
        *h.vector_hits.lock().unwrap() = vec![hit(DocumentId::new(), "a", "vector")];
        *h.fts_hits.lock().unwrap() = vec![hit(DocumentId::new(), "b", "fts")];
        *h.provider.fail_embedding.lock().unwrap() = embedding;
        *h.fail_vector.lock().unwrap() = vector;
        *h.fail_fts.lock().unwrap() = fts;
        let deltas = h.run().await;
        assert_eq!(deltas[0].retrieval_degraded.as_deref(), Some(warning));
        let collection = h.session.lock().unwrap().collection_id.unwrap();
        assert!(
            h.collection_fts
                .lock()
                .unwrap()
                .iter()
                .all(|q| q.collection_id == collection && q.include_descendants)
        );
        if !embedding {
            assert!(
                h.collection_vectors
                    .lock()
                    .unwrap()
                    .iter()
                    .all(|q| q.collection_id == collection && q.include_descendants)
            );
        }
    }
}

#[tokio::test]
async fn cross_item_applies_document_diversity_after_fusion() {
    let h = ChatHarness::new(MilaSessionType::CrossItem);
    let first = DocumentId::new();
    let second = DocumentId::new();
    *h.vector_hits.lock().unwrap() = vec![
        hit(first, "a", "one"),
        hit(first, "b", "two"),
        hit(second, "c", "three"),
    ];
    h.run().await;
    let assistant = h
        .messages
        .lock()
        .unwrap()
        .iter()
        .find(|m| m.role == MessageRole::Assistant)
        .unwrap()
        .clone();
    assert_eq!(assistant.source_chunks.len(), 2);
}

#[tokio::test]
async fn unsectioned_epub_fts_uses_hit_snippet_not_first_parent() {
    let h = ChatHarness::new(MilaSessionType::SingleDocument);
    h.prepared.lock().unwrap().root_text = "long ".repeat(200);
    let mut unsectioned = hit(h.document.id, "", "authoritative snippet");
    unsectioned.source_chunk_id = None;
    unsectioned.section = None;
    *h.provider.fail_embedding.lock().unwrap() = true;
    *h.fts_hits.lock().unwrap() = vec![unsectioned];
    h.run().await;
    let requests = h.provider.chat_requests.lock().unwrap();
    let request = &requests[0];
    let transcript = request
        .messages
        .iter()
        .map(|m| m.content.as_str())
        .collect::<Vec<_>>()
        .join("\n");
    assert!(transcript.contains("authoritative snippet"));
    assert!(!transcript.contains("wrong parent body"));
}

#[tokio::test]
async fn exact_root_hits_never_expand_to_a_structured_parent() {
    for lexical_fallback in [false, true] {
        let h = ChatHarness::new(MilaSessionType::SingleDocument);
        h.prepared.lock().unwrap().root_text = "long ".repeat(2_000);
        let mut root_hit = hit(h.document.id, "", "bounded root chunk");
        root_hit.section = None;
        if lexical_fallback {
            *h.provider.fail_embedding.lock().unwrap() = true;
            *h.fts_hits.lock().unwrap() = vec![root_hit];
        } else {
            *h.vector_hits.lock().unwrap() = vec![root_hit];
        }

        h.run().await;

        let requests = h.provider.chat_requests.lock().unwrap();
        let transcript = requests[0]
            .messages
            .iter()
            .map(|message| message.content.as_str())
            .collect::<Vec<_>>()
            .join("\n");
        assert!(transcript.contains("bounded root chunk"));
        assert!(!transcript.contains("wrong parent body"));
    }
}

#[tokio::test]
async fn structured_parent_is_emitted_once_without_a_duplicate_child_excerpt() {
    let h = ChatHarness::new(MilaSessionType::SingleDocument);
    h.prepared.lock().unwrap().root_text = "long ".repeat(2_000);
    h.prepared.lock().unwrap().parents[0].text = "alpha matched beta".into();
    *h.vector_hits.lock().unwrap() = vec![
        hit(h.document.id, "chapter-1", "matched"),
        hit(h.document.id, "chapter-1", "beta"),
    ];

    h.run().await;

    let requests = h.provider.chat_requests.lock().unwrap();
    let transcript = requests[0]
        .messages
        .iter()
        .map(|message| message.content.as_str())
        .collect::<Vec<_>>()
        .join("\n");
    assert_eq!(transcript.matches("Context: alpha matched beta").count(), 1);
    assert!(!transcript.contains("Matched excerpt:"));
}

#[tokio::test]
async fn unreachable_provider_fails_the_turn_before_any_message_is_persisted() {
    let h = ChatHarness::new(MilaSessionType::SingleDocument);
    *h.provider.fail_chat_stream.lock().unwrap() = true;
    let session_id = h.session.lock().unwrap().id;

    let result = h
        .service()
        .stream_chat(MilaChatRequest {
            user_id: h.user_id,
            session_id,
            question: "question".into(),
            highlight_text: None,
            highlight_offset: None,
        })
        .await;

    let Err(err) = result else {
        panic!("expected the turn to fail before streaming");
    };
    assert!(matches!(err, AppError::ProviderUnavailable { .. }));
    assert!(h.messages.lock().unwrap().is_empty());
}

#[tokio::test]
async fn successful_turn_persists_the_user_message_exactly_once() {
    let h = ChatHarness::new(MilaSessionType::SingleDocument);
    h.run().await;
    let user_messages = h
        .messages
        .lock()
        .unwrap()
        .iter()
        .filter(|m| m.role == MessageRole::User)
        .count();
    assert_eq!(user_messages, 1);
}

#[tokio::test]
async fn streamed_turn_strips_hallucinated_citations_before_persistence() {
    let h = ChatHarness::new(MilaSessionType::CrossItem);
    *h.vector_hits.lock().unwrap() = vec![hit(DocumentId::new(), "a", "source")];
    // `[S_]`/`[S]` are placeholders a model reaches for when it was told to cite but given no
    // labels; they resolve to nothing, so they must never reach a client either.
    *h.provider.chat_response.lock().unwrap() =
        "grounded [S1] invented [S9] placeholder [S_] bare [S]".into();
    h.run().await;
    let persisted = h
        .messages
        .lock()
        .unwrap()
        .iter()
        .find(|m| m.role == MessageRole::Assistant)
        .unwrap()
        .content
        .clone();
    assert_eq!(persisted, "grounded [S1] invented placeholder bare");
}

#[test]
fn reciprocal_rank_fusion_tables_caps_and_root_exception() {
    for (vector, fts, expected) in [
        (vec![hit(DocumentId::new(), "a", "v")], Vec::new(), 1),
        (Vec::new(), vec![hit(DocumentId::new(), "a", "f")], 1),
    ] {
        assert_eq!(reciprocal_rank_fusion(vector, fts, 2).len(), expected);
    }
    let id = DocumentId::new();
    let capped = reciprocal_rank_fusion(
        vec![
            hit(id, "chapter", "a"),
            hit(id, "chapter", "b"),
            hit(id, "chapter", "c"),
        ],
        Vec::new(),
        2,
    );
    assert_eq!(capped.len(), 2);
    let root = reciprocal_rank_fusion(
        vec![hit(id, "", "a"), hit(id, "", "b"), hit(id, "", "c")],
        Vec::new(),
        2,
    );
    assert_eq!(root.len(), 3);
}

#[test]
fn reciprocal_rank_fusion_merges_only_the_same_chunk() {
    let document_id = DocumentId::new();
    let chunk_id = ContentVectorId::new();
    let mut semantic = hit(document_id, "chapter", "canonical chunk content");
    semantic.source_chunk_id = Some(chunk_id);
    let mut same_chunk = hit(document_id, "chapter", "lexical rendering");
    same_chunk.source_chunk_id = Some(chunk_id);

    let fused = reciprocal_rank_fusion(vec![semantic], vec![same_chunk], 2);
    assert_eq!(fused.len(), 1);
    assert_eq!(fused[0].source_chunk_id, Some(chunk_id));
    assert_eq!(fused[0].snippet, "canonical chunk content");
    assert!((fused[0].score - (2.0 / 61.0)).abs() < 1e-12);

    let mut semantic = hit(document_id, "chapter", "semantic chunk");
    semantic.source_chunk_id = Some(ContentVectorId::new());
    let mut different_chunk = hit(document_id, "chapter", "different lexical chunk");
    different_chunk.source_chunk_id = Some(ContentVectorId::new());
    let mut coarse = hit(document_id, "chapter", "coarse fallback");
    coarse.source_chunk_id = None;

    let distinct = reciprocal_rank_fusion(vec![semantic], vec![different_chunk, coarse], 3);
    assert_eq!(distinct.len(), 3);
    assert!(distinct.iter().any(|candidate| candidate.source_chunk_id.is_none()));
}
