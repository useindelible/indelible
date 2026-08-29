#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use ind_test_support::TestDb;
use sqlx::Row;

/// Columns covered by COMPOSITE foreign keys to their owning table
/// (collections / library_entries / tags) rather than a direct FK to users.
/// They are deleted transitively when their parent cascades.
const COMPOSITE_CASCADE: &[(&str, &str)] = &[
    ("collection_entries", "user_id"),
    ("library_entry_tags", "user_id"),
];

async fn discover_user_owned_columns(
    pool: &sqlx::PgPool,
) -> Vec<(String, String, Option<String>, Option<String>)> {
    sqlx::query(
        r#"
        SELECT c.relname             AS table_name,
               a.attname             AS column_name,
               con.conname           AS constraint_name,
               con.confdeltype::text AS delete_action
        FROM pg_class c
        JOIN pg_namespace n ON n.oid = c.relnamespace AND n.nspname = 'public'
        JOIN pg_attribute a ON a.attrelid = c.oid
                           AND a.attname IN ('user_id', 'owner_user_id')
                           AND a.attnum > 0 AND NOT a.attisdropped
        LEFT JOIN pg_constraint con
               ON con.conrelid = c.oid AND con.contype = 'f'
              AND con.confrelid = 'public.users'::regclass
              AND a.attnum = ANY (con.conkey)
        WHERE c.relkind = 'r'
        ORDER BY 1, 2
        "#,
    )
    .fetch_all(pool)
    .await
    .unwrap()
    .into_iter()
    .map(|row| {
        (
            row.get::<String, _>("table_name"),
            row.get::<String, _>("column_name"),
            row.get::<Option<String>, _>("constraint_name"),
            row.get::<Option<String>, _>("delete_action"),
        )
    })
    .collect()
}

/// Every user-owned column must ride the cascade. A new table with a user_id
/// column and no ON DELETE CASCADE FK to users(id) means account deletion
/// silently leaves that table's rows behind — this test fails before that ships.
#[tokio::test]
async fn every_user_owned_column_cascades_from_users() {
    let db = TestDb::new().await;
    let rows = discover_user_owned_columns(db.pool()).await;

    // Anti-vacuity: if the catalog query breaks, fail loudly rather than pass empty.
    assert!(
        rows.len() >= 50,
        "only {} user-owned columns discovered; the catalog query is broken",
        rows.len()
    );

    let mut problems = Vec::new();
    for (table, column, constraint, action) in &rows {
        let pair = (table.as_str(), column.as_str());
        // 'c' = CASCADE in pg_constraint.confdeltype
        let ok = match action.as_deref() {
            Some("c") => true,
            None => COMPOSITE_CASCADE.contains(&pair),
            Some(_) => false,
        };
        if !ok {
            problems.push(format!(
                "{table}.{column}: constraint {constraint:?} action {action:?} — must be \
                 ON DELETE CASCADE (or covered by a composite cascade and listed in \
                 COMPOSITE_CASCADE)"
            ));
        }
    }
    assert!(
        problems.is_empty(),
        "account purge coverage broken:\n  {}",
        problems.join("\n  ")
    );

    for pair in COMPOSITE_CASCADE {
        assert!(
            rows.iter()
                .any(|(t, c, _, _)| (t.as_str(), c.as_str()) == *pair),
            "{}.{} listed in COMPOSITE_CASCADE but missing from schema",
            pair.0,
            pair.1
        );
    }
}

use ind_application::repos::account_purge::AccountPurgeRepository;
use ind_persistence::repos::PgAccountPurgeRepository;
use ind_test_support::factories::{DocumentFactory, UserFactory};

#[tokio::test]
async fn purge_account_deletes_user_content_and_enqueues_storage_purge() {
    let db = TestDb::new().await;
    let pool = db.pool().clone();
    let user = UserFactory::new().insert(&pool).await;
    let doc = DocumentFactory::new(user.id).insert(&pool).await;

    sqlx::query(
        "INSERT INTO archive_assets \
         (id, document_id, asset_kind, s3_bucket, s3_key, content_type, size_bytes, status, created_at) \
         VALUES (gen_random_uuid(), $1, 'readable_html', 'bucket', 'legacy/one/readable.html', \
                 'text/html', 10, 'completed', now())",
    )
    .bind(doc.id.into_uuid())
    .execute(&pool)
    .await
    .unwrap();

    let outcome = PgAccountPurgeRepository::new(pool.clone())
        .purge_account(user.id)
        .await
        .unwrap();
    assert_eq!(outcome.documents_deleted, 1);

    let users: i64 = sqlx::query_scalar("SELECT count(*) FROM users WHERE id = $1")
        .bind(user.id.into_uuid())
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(users, 0, "the account row must be gone");
    let docs: i64 = sqlx::query_scalar("SELECT count(*) FROM documents WHERE user_id = $1")
        .bind(user.id.into_uuid())
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(docs, 0, "owned documents must be gone");

    // The cleanup job row must survive the user delete inside the same transaction.
    let payload: serde_json::Value = sqlx::query_scalar(
        "SELECT payload FROM job_outbox WHERE job_type = 'account.storage_purge' AND dedupe_key = $1",
    )
    .bind(format!("account-storage-purge:{}", user.id))
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(payload["prefixes"].as_array().unwrap().len(), 9);
    assert!(
        payload["residual_keys"]
            .as_array()
            .unwrap()
            .iter()
            .any(|k| k == "legacy/one/readable.html"),
        "harvested legacy key must be carried by the cleanup job"
    );
}

#[tokio::test]
async fn purge_account_returns_not_found_for_unknown_user() {
    let db = TestDb::new().await;
    let missing: ind_domain::UserId = "usr_01890000-0000-7000-8000-00000000dead".parse().unwrap();
    let err = PgAccountPurgeRepository::new(db.pool().clone())
        .purge_account(missing)
        .await
        .unwrap_err();
    assert!(
        matches!(
            err,
            ind_application::error::AppError::Domain(ind_domain::DomainError::NotFound { .. })
        ),
        "expected NotFound, got {err:?}"
    );
}

#[tokio::test]
async fn purge_account_leaves_other_tenants_untouched() {
    let db = TestDb::new().await;
    let pool = db.pool().clone();
    let victim = UserFactory::new().insert(&pool).await;
    let bystander = UserFactory::new().insert(&pool).await;
    DocumentFactory::new(victim.id).insert(&pool).await;
    DocumentFactory::new(bystander.id).insert(&pool).await;

    PgAccountPurgeRepository::new(pool.clone())
        .purge_account(victim.id)
        .await
        .unwrap();

    let bystander_users: i64 = sqlx::query_scalar("SELECT count(*) FROM users WHERE id = $1")
        .bind(bystander.id.into_uuid())
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(bystander_users, 1);
    let bystander_docs: i64 =
        sqlx::query_scalar("SELECT count(*) FROM documents WHERE user_id = $1")
            .bind(bystander.id.into_uuid())
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(bystander_docs, 1);
}

/// Tables deliberately not seeded by `seed_user_owned_rows`, with the reason.
/// The FK cascade for every one of these is proven by
/// `every_user_owned_column_cascades_from_users`; a direct seed adds runtime
/// proof and should migrate rows out of this ledger over time.
const UNSEEDED_ACKNOWLEDGED: &[(&str, &str)] = &[
    ("ai_prompt_presets", "user_id"),
    ("ai_runs", "user_id"),
    ("authorization_codes", "user_id"),
    ("billing_account_members", "user_id"),
    ("billing_accounts", "owner_user_id"),
    ("billing_usage_events", "user_id"),
    ("collection_entries", "user_id"),
    ("content_vectors", "user_id"),
    ("document_origins", "user_id"),
    ("document_playback_states", "user_id"),
    ("email_aliases", "user_id"),
    ("email_ingest_log", "user_id"),
    ("email_senders", "user_id"),
    ("email_verification_tokens", "user_id"),
    ("entities", "user_id"),
    ("entitlement_snapshots", "user_id"),
    ("entity_aliases", "user_id"),
    ("feed_deliveries", "user_id"),
    ("import_jobs", "user_id"),
    ("integration_connections", "user_id"),
    ("integration_oauth_tokens", "user_id"),
    ("library_entry_tags", "user_id"),
    ("lifecycle_actions", "user_id"),
    ("mila_sessions", "user_id"),
    ("notification_preferences", "user_id"),
    ("oauth_identities", "user_id"),
    ("obsidian_export_artifacts", "user_id"),
    ("obsidian_export_runs", "user_id"),
    ("password_reset_tokens", "user_id"),
    ("push_tokens", "user_id"),
    ("referral_credits", "user_id"),
    ("refresh_tokens", "user_id"),
    ("review_cards", "user_id"),
    ("search_documents", "user_id"),
    ("storage_add_ons", "user_id"),
    ("tts_audio_assets", "user_id"),
    ("tts_chunks", "user_id"),
    ("tts_sessions", "user_id"),
    ("tts_voice_personas", "user_id"),
    ("user_document_state", "user_id"),
];

/// Seeds one row per covered user-owned table and returns the set of seeded
/// table names.
async fn seed_user_owned_rows(
    pool: &sqlx::PgPool,
    user: &ind_domain::User,
) -> std::collections::HashSet<&'static str> {
    use ind_test_support::factories::{
        CollectionFactory, FeedSubscriptionFactory, SavedDocumentFactory,
    };

    let saved = SavedDocumentFactory::new(user.id).insert(pool).await;
    CollectionFactory::new(user.id).insert(pool).await;
    FeedSubscriptionFactory::new(user.id).insert(pool).await;

    let uid = user.id.into_uuid();
    let doc = saved.document_id.into_uuid();
    for sql in [
        "INSERT INTO highlights (id, user_id, document_id, text_content, locator, created_at, updated_at) \
         VALUES (gen_random_uuid(), $1, $2, 'seeded highlight', '{}'::jsonb, now(), now())",
        "INSERT INTO item_notes (id, user_id, document_id, body, created_at, updated_at) \
         VALUES (gen_random_uuid(), $1, $2, 'seeded note', now(), now())",
        "INSERT INTO tags (id, user_id, name, created_at) VALUES (gen_random_uuid(), $1, 'seeded-tag', now())",
        "INSERT INTO api_tokens (id, user_id, name, token_hash, prefix, created_at) \
         VALUES (gen_random_uuid(), $1, 'seeded', 'hash-' || $1::text, 'ind_seed', now())",
        "INSERT INTO webhook_endpoints (id, user_id, url, secret_hash, events, created_at, updated_at) \
         VALUES (gen_random_uuid(), $1, 'https://example.test/hook', 'hash-' || $1::text, ARRAY['library_entry.saved'], now(), now())",
        "INSERT INTO notifications (id, user_id, notification_type, title, created_at) \
         VALUES (gen_random_uuid(), $1, 'system', 'seeded', now())",
        "INSERT INTO usage_counters (id, user_id, quota_name, period_start, period_end, limit_value) \
         VALUES (gen_random_uuid(), $1, 'seeded_quota', now(), now() + interval '1 day', 10)",
        "INSERT INTO domain_events (id, event_type, aggregate_type, aggregate_id, user_id, payload, created_at) \
         VALUES (gen_random_uuid(), 'seed.event', 'document', $2, $1, '{}'::jsonb, now())",
        "INSERT INTO recent_searches (id, user_id, raw_query, normalized_query, last_searched_at, created_at, updated_at) \
         VALUES (gen_random_uuid(), $1, 'Seed Query', 'seed query', now(), now(), now())",
        "INSERT INTO smart_lists (id, user_id, name, filter_expression, created_at, updated_at) \
         VALUES (gen_random_uuid(), $1, 'seeded view', '{}'::jsonb, now(), now())",
        "INSERT INTO user_preferences (user_id) VALUES ($1) ON CONFLICT (user_id) DO NOTHING",
        "INSERT INTO mila_config (user_id, chat_model, embedding_model, embedding_dim, model_context_window, created_at, updated_at) \
         VALUES ($1, 'seed-chat', 'seed-embed', 768, 16000, now(), now())",
        "INSERT INTO reading_events (id, user_id, document_id, origin, origin_seq, event_kind, progress_basis_points, active_ms, recorded_at, effective_at) \
         VALUES (gen_random_uuid(), $1, $2, 'surface:web', 0, 'progress', 4200, 1000, now(), now())",
    ] {
        sqlx::query(sql)
            .bind(uid)
            .bind(doc)
            .execute(pool)
            .await
            .unwrap_or_else(|e| panic!("seed failed: {sql}: {e}"));
    }

    [
        "users",
        "documents",
        "library_entries",
        "collections",
        "feed_subscriptions",
        "highlights",
        "item_notes",
        "tags",
        "api_tokens",
        "webhook_endpoints",
        "notifications",
        "usage_counters",
        "domain_events",
        "recent_searches",
        "smart_lists",
        "user_preferences",
        "mila_config",
        "reading_events",
    ]
    .into_iter()
    .collect()
}

#[tokio::test]
async fn purge_removes_every_seeded_user_row_and_spares_the_bystander() {
    let db = TestDb::new().await;
    let pool = db.pool().clone();
    let victim = UserFactory::new().insert(&pool).await;
    let bystander = UserFactory::new().insert(&pool).await;

    let seeded = seed_user_owned_rows(&pool, &victim).await;
    seed_user_owned_rows(&pool, &bystander).await;

    let discovered = discover_user_owned_columns(&pool).await;
    for (table, column, _, _) in &discovered {
        let pair = (table.as_str(), column.as_str());
        assert!(
            seeded.contains(table.as_str()) || UNSEEDED_ACKNOWLEDGED.contains(&pair),
            "{table}.{column} is user-owned but neither seeded by seed_user_owned_rows \
             nor listed in UNSEEDED_ACKNOWLEDGED — extend the seeder or acknowledge it"
        );
    }

    PgAccountPurgeRepository::new(pool.clone())
        .purge_account(victim.id)
        .await
        .unwrap();

    for (table, column, _, _) in &discovered {
        // Identifiers come from pg_catalog, but keep the interpolation guarded.
        assert!(
            table.chars().all(|c| c.is_ascii_lowercase() || c == '_'),
            "unexpected table name {table:?}"
        );
        let sql = format!("SELECT count(*) FROM public.{table} WHERE {column} = $1");
        let remaining: i64 = sqlx::query_scalar(&sql)
            .bind(victim.id.into_uuid())
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(
            remaining, 0,
            "{table}.{column} still has {remaining} rows for the purged user"
        );

        if seeded.contains(table.as_str()) && *table != "users" {
            let survivors: i64 = sqlx::query_scalar(&sql)
                .bind(bystander.id.into_uuid())
                .fetch_one(&pool)
                .await
                .unwrap();
            assert!(
                survivors > 0,
                "{table}.{column}: purging one user destroyed another's rows"
            );
        }
    }

    let bystander_row: i64 = sqlx::query_scalar("SELECT count(*) FROM users WHERE id = $1")
        .bind(bystander.id.into_uuid())
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(bystander_row, 1);
}

#[tokio::test]
async fn purge_removes_user_scoped_queue_payloads_but_keeps_the_cleanup_job() {
    let db = TestDb::new().await;
    let pool = db.pool().clone();
    let victim = UserFactory::new().insert(&pool).await;
    let bystander = UserFactory::new().insert(&pool).await;

    for (user, marker) in [(&victim, "victim"), (&bystander, "bystander")] {
        let payload = serde_json::json!({
            "user_id": user.id.to_string(),
            "raw_payload": [104, 105],
            "marker": marker,
        });
        sqlx::query(
            "INSERT INTO job_outbox (id, job_type, payload, dedupe_key, available_at, created_at) \
             VALUES (gen_random_uuid(), 'email.ingest', $1, $2, now(), now())",
        )
        .bind(&payload)
        .bind(format!("seed-outbox-{marker}"))
        .execute(&pool)
        .await
        .unwrap();
        sqlx::query(
            "INSERT INTO background_job_recoveries \
             (id, recovery_key, job_type, payload, status, failure_class, failure_reason_code, \
              error_message, first_failed_at, last_failed_at, created_at, updated_at) \
             VALUES (gen_random_uuid(), $2, 'email.ingest', $1, 'terminal', 'terminal', \
                     'external_service_error', 'seeded', now(), now(), now(), now())",
        )
        .bind(&payload)
        .bind(format!("seed-recovery-{marker}"))
        .execute(&pool)
        .await
        .unwrap();
        sqlx::query(
            "INSERT INTO dead_letter_jobs \
             (id, original_job_type, original_payload, error_message, attempts, failed_at) \
             VALUES (gen_random_uuid(), 'email.ingest', $1, 'seeded', 3, now())",
        )
        .bind(&payload)
        .execute(&pool)
        .await
        .unwrap();
    }

    PgAccountPurgeRepository::new(pool.clone())
        .purge_account(victim.id)
        .await
        .unwrap();

    let victim_marker = victim.id.to_string();
    for (table, column, exclusion) in [
        // The storage-cleanup job is the one deliberate survivor.
        (
            "job_outbox",
            "payload",
            " AND job_type != 'account.storage_purge'",
        ),
        ("background_job_recoveries", "payload", ""),
        ("dead_letter_jobs", "original_payload", ""),
    ] {
        let leaked: i64 = sqlx::query_scalar(&format!(
            "SELECT count(*) FROM {table} WHERE {column}->>'user_id' = $1{exclusion}"
        ))
        .bind(&victim_marker)
        .fetch_one(&pool)
        .await
        .unwrap();
        assert_eq!(leaked, 0, "{table} still holds the purged user's payloads");

        let bystander_rows: i64 = sqlx::query_scalar(&format!(
            "SELECT count(*) FROM {table} WHERE {column}->>'user_id' = $1{exclusion}"
        ))
        .bind(bystander.id.to_string())
        .fetch_one(&pool)
        .await
        .unwrap();
        assert_eq!(
            bystander_rows, 1,
            "{table}: bystander payloads must survive"
        );
    }

    // The one deliberate survivor: the storage-cleanup job for this purge.
    let cleanup_jobs: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM job_outbox WHERE job_type = 'account.storage_purge' \
         AND dedupe_key = $1",
    )
    .bind(format!("account-storage-purge:{}", victim.id))
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(cleanup_jobs, 1);
}
