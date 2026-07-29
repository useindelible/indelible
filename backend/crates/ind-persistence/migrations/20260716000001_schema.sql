CREATE SCHEMA apalis;

CREATE EXTENSION IF NOT EXISTS btree_gin WITH SCHEMA public;

CREATE EXTENSION IF NOT EXISTS pg_trgm WITH SCHEMA public;

CREATE EXTENSION IF NOT EXISTS vector WITH SCHEMA public;

CREATE TABLE apalis.jobs (
    job bytea NOT NULL,
    id text NOT NULL,
    job_type text NOT NULL,
    status text DEFAULT 'Pending'::text NOT NULL,
    attempts integer DEFAULT 0 NOT NULL,
    max_attempts integer DEFAULT 25 NOT NULL,
    run_at timestamp with time zone DEFAULT now() NOT NULL,
    last_result jsonb,
    lock_at timestamp with time zone,
    lock_by text,
    done_at timestamp with time zone,
    priority integer DEFAULT 0,
    metadata jsonb
);

CREATE FUNCTION apalis.get_jobs(worker_id text, v_job_type text, v_job_count integer DEFAULT 5) RETURNS SETOF apalis.jobs
    LANGUAGE plpgsql
    AS $$
BEGIN
    RETURN QUERY
    UPDATE apalis.jobs
    SET status = 'Queued',
        lock_by = worker_id,
        lock_at = now()
    WHERE id IN (
        SELECT id
        FROM apalis.jobs
        WHERE (status = 'Pending' OR (status = 'Failed' AND attempts < max_attempts))
            AND run_at < now()
            AND job_type = v_job_type
        ORDER BY priority DESC, run_at ASC
        LIMIT v_job_count
        FOR UPDATE SKIP LOCKED
    )
    RETURNING *;
END;
$$;

CREATE FUNCTION apalis.notify_new_jobs() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
BEGIN
    IF NEW.run_at <= now() THEN
        PERFORM pg_notify(
            'apalis::job::insert',
            json_build_object(
                'job_type', NEW.job_type,
                'id', NEW.id,
                'run_at', NEW.run_at
            )::text
        );
    END IF;
    RETURN NEW;
END;
$$;

CREATE FUNCTION public.fts_config_for_language(language text) RETURNS regconfig
    LANGUAGE sql IMMUTABLE PARALLEL SAFE
    AS $$
SELECT CASE
    WHEN $1 IS NULL
         OR lower(split_part(replace(btrim($1), '_', '-'), '-', 1)) IN ('', 'und', 'en', 'eng')
        THEN 'english'::regconfig
    ELSE 'simple'::regconfig
END;
$$;

CREATE FUNCTION public.fts_relaxed_query(config regconfig, query_text text) RETURNS tsquery
    LANGUAGE sql IMMUTABLE PARALLEL SAFE
    AS $$
WITH parsed AS MATERIALIZED (
    SELECT lexeme, positions[1] AS first_position
    FROM unnest(to_tsvector($1, COALESCE($2, '')))
), terms AS (
    SELECT lexeme, first_position
    FROM parsed
    WHERE char_length(lexeme) > 1
       OR NOT EXISTS (SELECT 1 FROM parsed WHERE char_length(lexeme) > 1)
    ORDER BY first_position, lexeme
    LIMIT 32
)
SELECT COALESCE(string_agg(quote_literal(lexeme), ' | ' ORDER BY first_position), '')::tsquery
FROM terms;
$$;

CREATE FUNCTION public.feed_source_entries_tsv_update() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
DECLARE
    search_config regconfig;
BEGIN
    search_config := public.fts_config_for_language(NEW.language);
    NEW.search_tsv :=
        setweight(to_tsvector(search_config, COALESCE(NEW.title, '')), 'A') ||
        setweight(to_tsvector(search_config, COALESCE(NEW.author, '')), 'B') ||
        setweight(to_tsvector(search_config, COALESCE(NEW.excerpt, '')), 'C') ||
        setweight(to_tsvector(search_config, COALESCE(NEW.content_html, '')), 'D');
    RETURN NEW;
END;
$$;

CREATE FUNCTION public.notify_domain_events_insert() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
BEGIN
    PERFORM pg_notify(
        'indelible_domain_events',
        json_build_object(
            'user_id', NEW.user_id::text,
            'event_id', NEW.id::text
        )::text
    );
    RETURN NEW;
END;
$$;

CREATE TABLE apalis.workers (
    id text NOT NULL,
    worker_type text NOT NULL,
    storage_name text NOT NULL,
    layers text DEFAULT ''::text NOT NULL,
    last_seen timestamp with time zone DEFAULT now() NOT NULL,
    started_at timestamp with time zone
);

CREATE TABLE public.ai_outputs (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    output_type text NOT NULL,
    content jsonb NOT NULL,
    ai_run_id uuid,
    created_at timestamp with time zone NOT NULL,
    document_id uuid NOT NULL
);

CREATE TABLE public.ai_prompt_presets (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    user_id uuid,
    name text NOT NULL,
    action text NOT NULL,
    system_prompt text NOT NULL,
    is_default boolean DEFAULT false NOT NULL,
    created_at timestamp with time zone NOT NULL,
    is_system boolean DEFAULT false NOT NULL,
    CONSTRAINT chk_ai_prompt_presets_user_or_system CHECK ((((is_system = true) AND (user_id IS NULL)) OR ((is_system = false) AND (user_id IS NOT NULL))))
);

CREATE TABLE public.ai_runs (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    user_id uuid NOT NULL,
    action text NOT NULL,
    provider text NOT NULL,
    model text NOT NULL,
    input_tokens integer,
    output_tokens integer,
    is_byok boolean NOT NULL,
    status text NOT NULL,
    error_message text,
    started_at timestamp with time zone NOT NULL,
    completed_at timestamp with time zone,
    document_id uuid
);

CREATE TABLE public.api_tokens (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    user_id uuid NOT NULL,
    name text NOT NULL,
    token_hash text NOT NULL,
    prefix text NOT NULL,
    permissions text[] DEFAULT '{}'::text[] NOT NULL,
    last_used_at timestamp with time zone,
    expires_at timestamp with time zone,
    created_at timestamp with time zone NOT NULL
);

CREATE TABLE public.archive_assets (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    asset_kind text NOT NULL,
    s3_key text NOT NULL,
    s3_bucket text NOT NULL,
    content_type text NOT NULL,
    size_bytes bigint NOT NULL,
    created_at timestamp with time zone NOT NULL,
    status text DEFAULT 'completed'::text NOT NULL,
    failed_reason text,
    document_id uuid NOT NULL
);

CREATE TABLE public.authorization_codes (
    id uuid NOT NULL,
    user_id uuid NOT NULL,
    code_hash text NOT NULL,
    code_challenge text NOT NULL,
    code_challenge_method text DEFAULT 'S256'::text NOT NULL,
    client_type text NOT NULL,
    redirect_uri text NOT NULL,
    used_at timestamp with time zone,
    expires_at timestamp with time zone NOT NULL,
    created_at timestamp with time zone NOT NULL
);

CREATE TABLE public.background_job_recoveries (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    recovery_key text NOT NULL,
    job_type text NOT NULL,
    payload jsonb NOT NULL,
    dedupe_key text,
    outbox_id uuid,
    subject_kind text,
    subject_id text,
    status text NOT NULL,
    failure_class text NOT NULL,
    failure_reason_code text NOT NULL,
    error_message text NOT NULL,
    apalis_attempts integer DEFAULT 0 NOT NULL,
    recovery_attempts integer DEFAULT 0 NOT NULL,
    next_retry_at timestamp with time zone,
    lease_owner text,
    lease_expires_at timestamp with time zone,
    first_failed_at timestamp with time zone NOT NULL,
    last_failed_at timestamp with time zone NOT NULL,
    resolved_at timestamp with time zone,
    created_at timestamp with time zone NOT NULL,
    updated_at timestamp with time zone NOT NULL,
    CONSTRAINT chk_background_job_recoveries_failure_class CHECK ((failure_class = ANY (ARRAY['retryable'::text, 'terminal'::text, 'patient'::text]))),
    CONSTRAINT chk_background_job_recoveries_status CHECK ((status = ANY (ARRAY['waiting'::text, 'leased'::text, 'terminal'::text, 'resolved'::text]))),
    CONSTRAINT chk_background_job_recoveries_subject_kind CHECK (((subject_kind IS NULL) OR (subject_kind = ANY (ARRAY['document'::text, 'library_entry'::text, 'feed_delivery'::text, 'feed_source'::text, 'integration_connection'::text, 'import_job'::text])))),
    CONSTRAINT chk_background_job_recoveries_waiting_has_next_retry_at CHECK (((status <> 'waiting'::text) OR (next_retry_at IS NOT NULL)))
);

CREATE TABLE public.billing_account_members (
    billing_account_id uuid NOT NULL,
    user_id uuid NOT NULL,
    role text NOT NULL,
    plan_override_id uuid,
    invited_at timestamp with time zone NOT NULL,
    accepted_at timestamp with time zone
);

CREATE TABLE public.billing_accounts (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    owner_user_id uuid NOT NULL,
    stripe_customer_id text,
    account_type text NOT NULL,
    status text NOT NULL,
    created_at timestamp with time zone NOT NULL,
    updated_at timestamp with time zone NOT NULL
);

CREATE TABLE public.billing_events (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    stripe_event_id text NOT NULL,
    event_type text NOT NULL,
    payload jsonb NOT NULL,
    processed_at timestamp with time zone NOT NULL
);

CREATE TABLE public.billing_usage_events (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    user_id uuid NOT NULL,
    billing_account_id uuid,
    product_area text NOT NULL,
    event_type text NOT NULL,
    provider text,
    billing_mode text NOT NULL,
    resource_type text NOT NULL,
    resource_id uuid,
    units jsonb DEFAULT '{}'::jsonb NOT NULL,
    cost_units bigint DEFAULT 0 NOT NULL,
    amount_cents integer,
    currency text,
    idempotency_key text NOT NULL,
    metadata jsonb DEFAULT '{}'::jsonb NOT NULL,
    occurred_at timestamp with time zone DEFAULT now() NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    CONSTRAINT chk_billing_usage_amount CHECK (((amount_cents IS NULL) OR (amount_cents >= 0))),
    CONSTRAINT chk_billing_usage_billing_mode CHECK ((billing_mode = 'managed'::text)),
    CONSTRAINT chk_billing_usage_currency CHECK (((currency IS NULL) OR (currency ~ '^[A-Z]{3}$'::text))),
    CONSTRAINT chk_billing_usage_event_type CHECK ((event_type = ANY (ARRAY['tts_synthesis'::text, 'mila_chat_completion'::text, 'mila_embedding'::text, 'storage_bytes'::text, 'render_job'::text]))),
    CONSTRAINT chk_billing_usage_nonnegative_cost CHECK ((cost_units >= 0)),
    CONSTRAINT chk_billing_usage_product_area CHECK ((product_area = ANY (ARRAY['tts'::text, 'mila'::text, 'storage'::text, 'render'::text]))),
    CONSTRAINT chk_billing_usage_resource_type CHECK ((resource_type = ANY (ARRAY['tts_chunk'::text, 'mila_session'::text, 'item'::text, 'archive_asset'::text, 'render_job'::text]))),
    CONSTRAINT chk_billing_usage_tts_shape CHECK (((product_area <> 'tts'::text) OR ((event_type = 'tts_synthesis'::text) AND (resource_type = 'tts_chunk'::text) AND ((units ? 'characters'::text) OR (units ? 'audio_seconds'::text) OR (units ? 'cost_units'::text))))),
    CONSTRAINT chk_billing_usage_units_object CHECK ((jsonb_typeof(units) = 'object'::text))
);

CREATE TABLE public.collection_entries (
    user_id uuid NOT NULL,
    collection_id uuid NOT NULL,
    library_entry_id uuid NOT NULL,
    added_at timestamp with time zone DEFAULT now() NOT NULL
);

CREATE TABLE public.collections (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    user_id uuid NOT NULL,
    parent_id uuid,
    name text NOT NULL,
    description text,
    icon text,
    color text,
    sort_order integer DEFAULT 0 NOT NULL,
    rss_token text,
    created_at timestamp with time zone NOT NULL,
    updated_at timestamp with time zone NOT NULL,
    is_pinned boolean DEFAULT false NOT NULL
);

CREATE TABLE public.content_vectors (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    user_id uuid NOT NULL,
    chunk_index integer NOT NULL,
    content text NOT NULL,
    token_count integer NOT NULL,
    created_at timestamp with time zone NOT NULL,
    section_kind text DEFAULT 'item'::text NOT NULL,
    section_key text DEFAULT ''::text NOT NULL,
    embedding public.vector(768) NOT NULL,
    document_id uuid NOT NULL,
    embedding_model text DEFAULT 'text-embedding-3-small'::text NOT NULL,
    embedding_dim integer DEFAULT 768 NOT NULL,
    search_config regconfig DEFAULT 'english'::regconfig NOT NULL,
    content_tsv tsvector GENERATED ALWAYS AS (to_tsvector(search_config, content)) STORED NOT NULL,
    CONSTRAINT ck_content_vectors_embedding_dim_768 CHECK ((embedding_dim = 768))
);

CREATE TABLE public.dead_letter_jobs (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    original_job_type text NOT NULL,
    original_payload jsonb NOT NULL,
    error_message text NOT NULL,
    attempts integer NOT NULL,
    failed_at timestamp with time zone NOT NULL,
    original_dedupe_key text,
    replayed_at timestamp with time zone,
    replay_outbox_id uuid,
    failure_reason_code text,
    CONSTRAINT ck_dead_letter_jobs_replay_link CHECK ((((replayed_at IS NULL) AND (replay_outbox_id IS NULL)) OR ((replayed_at IS NOT NULL) AND (replay_outbox_id IS NOT NULL))))
);

CREATE TABLE public.document_origins (
    user_id uuid NOT NULL,
    document_id uuid NOT NULL,
    origin_type text NOT NULL,
    origin_id uuid NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL
);

CREATE TABLE public.document_playback_states (
    user_id uuid NOT NULL,
    document_id uuid NOT NULL,
    playback_kind text NOT NULL,
    position_seconds double precision DEFAULT 0 NOT NULL,
    playback_speed double precision DEFAULT 1 NOT NULL,
    element_index integer,
    tts_chunk_id text,
    tts_voice_persona_id uuid,
    is_playing boolean DEFAULT false NOT NULL,
    updated_at timestamp with time zone DEFAULT now() NOT NULL
);

CREATE TABLE public.document_video_metadata (
    document_id uuid NOT NULL,
    duration_seconds integer,
    channel_name text,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    updated_at timestamp with time zone DEFAULT now() NOT NULL
);

CREATE TABLE public.documents (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    user_id uuid NOT NULL,
    document_type text NOT NULL,
    canonical_url text,
    original_url text,
    content_hash text,
    title text NOT NULL,
    author text,
    excerpt text,
    published_at timestamp with time zone,
    language text,
    domain text,
    lead_image_url text,
    thumbnail_url text,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    updated_at timestamp with time zone DEFAULT now() NOT NULL,
    sender_id uuid,
    word_count integer,
    reading_time_minutes integer,
    CONSTRAINT ck_documents_type CHECK ((document_type = ANY (ARRAY['article'::text, 'book'::text, 'email'::text, 'pdf'::text, 'tweet'::text, 'video'::text, 'podcast'::text])))
);

CREATE TABLE public.domain_events (
    id uuid NOT NULL,
    event_type text NOT NULL,
    aggregate_type text NOT NULL,
    aggregate_id uuid NOT NULL,
    user_id uuid NOT NULL,
    payload jsonb NOT NULL,
    created_at timestamp with time zone NOT NULL
);

CREATE TABLE public.email_aliases (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    user_id uuid NOT NULL,
    destination text NOT NULL,
    local_part text NOT NULL,
    status text DEFAULT 'active'::text NOT NULL,
    is_default boolean DEFAULT false NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    retire_at timestamp with time zone,
    retired_at timestamp with time zone,
    CONSTRAINT chk_email_aliases_destination CHECK ((destination = ANY (ARRAY['feed'::text, 'library'::text]))),
    CONSTRAINT chk_email_aliases_status CHECK ((status = ANY (ARRAY['active'::text, 'retired'::text])))
);

CREATE TABLE public.email_ingest_log (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    provider text NOT NULL,
    provider_email_id text NOT NULL,
    user_id uuid NOT NULL,
    destination text NOT NULL,
    status text DEFAULT 'pending'::text NOT NULL,
    error text,
    raw_payload bytea,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    processed_at timestamp with time zone,
    CONSTRAINT chk_email_ingest_log_status CHECK ((status = ANY (ARRAY['pending'::text, 'blocked'::text, 'duplicate'::text, 'failed'::text, 'processed'::text, 'gmail_confirmation'::text])))
);

CREATE TABLE public.email_senders (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    user_id uuid NOT NULL,
    canonical_addr text NOT NULL,
    list_id text,
    display_name text,
    render_default text DEFAULT 'reader'::text NOT NULL,
    routing_default text,
    blocked_at timestamp with time zone,
    first_seen_at timestamp with time zone DEFAULT now() NOT NULL,
    last_seen_at timestamp with time zone DEFAULT now() NOT NULL,
    delivery_count integer DEFAULT 0 NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    updated_at timestamp with time zone DEFAULT now() NOT NULL,
    CONSTRAINT chk_email_senders_render_default CHECK ((render_default = ANY (ARRAY['reader'::text, 'original'::text]))),
    CONSTRAINT chk_email_senders_routing_default CHECK (((routing_default IS NULL) OR (routing_default = ANY (ARRAY['feed'::text, 'library'::text]))))
);

CREATE TABLE public.email_unsubscribe_targets (
    sender_id uuid NOT NULL,
    one_click_post_url text,
    mailto_addr text,
    web_url text,
    last_seen_at timestamp with time zone DEFAULT now() NOT NULL
);

CREATE TABLE public.email_verification_tokens (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    user_id uuid NOT NULL,
    token_hash text NOT NULL,
    expires_at timestamp with time zone NOT NULL,
    created_at timestamp with time zone NOT NULL
);

CREATE TABLE public.entities (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    user_id uuid NOT NULL,
    name text NOT NULL,
    entity_type text NOT NULL,
    description text,
    created_at timestamp with time zone NOT NULL
);

CREATE TABLE public.entitlement_snapshots (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    user_id uuid NOT NULL,
    effective_plan_id uuid NOT NULL,
    entitlements jsonb NOT NULL,
    quotas jsonb NOT NULL,
    computed_at timestamp with time zone NOT NULL
);

CREATE TABLE public.entity_aliases (
    user_id uuid NOT NULL,
    entity_type text NOT NULL,
    name text NOT NULL,
    entity_id uuid NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL
);

CREATE TABLE public.entity_mentions (
    entity_id uuid NOT NULL,
    mention_count integer DEFAULT 1 NOT NULL,
    first_seen_at timestamp with time zone NOT NULL,
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    document_id uuid NOT NULL
);

CREATE TABLE public.feed_deliveries (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    user_id uuid NOT NULL,
    subscription_id uuid NOT NULL,
    source_id uuid NOT NULL,
    source_entry_id uuid NOT NULL,
    document_id uuid,
    delivered_at timestamp with time zone DEFAULT now() NOT NULL,
    seen_at timestamp with time zone,
    dismissed_at timestamp with time zone,
    hidden_at timestamp with time zone,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    updated_at timestamp with time zone DEFAULT now() NOT NULL
);

CREATE TABLE public.feed_provider_instances (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    provider_type text NOT NULL,
    base_url text NOT NULL,
    priority integer DEFAULT 100 NOT NULL,
    enabled boolean DEFAULT true NOT NULL,
    last_success_at timestamp with time zone,
    last_failure_at timestamp with time zone,
    consecutive_failures integer DEFAULT 0 NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    updated_at timestamp with time zone DEFAULT now() NOT NULL
);

CREATE TABLE public.feed_source_entries (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    source_id uuid NOT NULL,
    guid text NOT NULL,
    title text NOT NULL,
    url text,
    author text,
    excerpt text,
    content_html text,
    language text,
    published_at timestamp with time zone,
    discovered_at timestamp with time zone NOT NULL,
    canonical_url text,
    search_tsv tsvector,
    lead_image_url text
);

CREATE TABLE public.feed_sources (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    canonical_key text NOT NULL,
    source_url text NOT NULL,
    poll_url text NOT NULL,
    title text NOT NULL,
    description text,
    site_url text,
    image_url text,
    domain text,
    feed_type text NOT NULL,
    visibility text DEFAULT 'public'::text NOT NULL,
    provider text,
    is_resolvable boolean DEFAULT false NOT NULL,
    popularity integer DEFAULT 0 NOT NULL,
    last_entry_added_at timestamp with time zone,
    last_polled_at timestamp with time zone,
    next_poll_at timestamp with time zone,
    last_etag text,
    last_modified text,
    consecutive_failures integer DEFAULT 0 NOT NULL,
    last_error text,
    lease_owner text,
    lease_expires_at timestamp with time zone,
    created_at timestamp with time zone NOT NULL,
    updated_at timestamp with time zone NOT NULL
);

CREATE TABLE public.feed_subscriptions (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    user_id uuid NOT NULL,
    auto_save boolean DEFAULT false NOT NULL,
    auto_save_collection_id uuid,
    status text DEFAULT 'active'::text NOT NULL,
    created_at timestamp with time zone NOT NULL,
    updated_at timestamp with time zone NOT NULL,
    source_id uuid NOT NULL,
    input_url text NOT NULL,
    title_override text,
    poll_interval_override_minutes integer
);

CREATE TABLE public.highlight_notes (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    highlight_id uuid NOT NULL,
    body text NOT NULL,
    created_at timestamp with time zone NOT NULL,
    updated_at timestamp with time zone NOT NULL
);

CREATE TABLE public.highlight_tags (
    highlight_id uuid NOT NULL,
    tag_id uuid NOT NULL,
    added_at timestamp with time zone NOT NULL
);

CREATE TABLE public.highlights (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    user_id uuid NOT NULL,
    color text DEFAULT 'yellow'::text NOT NULL,
    text_content text NOT NULL,
    locator jsonb,
    created_at timestamp with time zone NOT NULL,
    updated_at timestamp with time zone NOT NULL,
    source_locator jsonb,
    document_id uuid NOT NULL,
    CONSTRAINT highlights_locator_present_check CHECK (((locator IS NOT NULL) OR (source_locator IS NOT NULL)))
);

CREATE TABLE public.import_job_items (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    import_job_id uuid NOT NULL,
    external_id text NOT NULL,
    outcome text NOT NULL,
    error text,
    created_at timestamp with time zone NOT NULL,
    diagnostics jsonb,
    CONSTRAINT ck_import_job_items_outcome CHECK ((outcome = ANY (ARRAY['imported'::text, 'updated'::text, 'duplicate'::text, 'skipped_private'::text, 'failed'::text])))
);

CREATE TABLE public.import_jobs (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    user_id uuid NOT NULL,
    import_source text NOT NULL,
    import_method text NOT NULL,
    status text DEFAULT 'awaiting_provider'::text NOT NULL,
    imported_count integer DEFAULT 0 NOT NULL,
    updated_count integer DEFAULT 0 NOT NULL,
    duplicate_count integer DEFAULT 0 NOT NULL,
    skipped_private_count integer DEFAULT 0 NOT NULL,
    failed_count integer DEFAULT 0 NOT NULL,
    raw_artifact_key text,
    error text,
    created_at timestamp with time zone NOT NULL,
    started_at timestamp with time zone,
    finished_at timestamp with time zone,
    provider_report jsonb,
    CONSTRAINT ck_import_jobs_method CHECK ((import_method = ANY (ARRAY['oauth'::text, 'csv'::text, 'zip'::text]))),
    CONSTRAINT ck_import_jobs_status CHECK ((status = ANY (ARRAY['awaiting_provider'::text, 'pending'::text, 'running'::text, 'completed'::text, 'failed'::text, 'partial'::text, 'rolled_back'::text])))
);

CREATE TABLE public.integration_connections (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    user_id uuid NOT NULL,
    provider text NOT NULL,
    config jsonb NOT NULL,
    status text DEFAULT 'active'::text NOT NULL,
    last_sync_at timestamp with time zone,
    last_error text,
    created_at timestamp with time zone NOT NULL,
    updated_at timestamp with time zone NOT NULL,
    version bigint DEFAULT 0 NOT NULL
);

CREATE TABLE public.integration_export_cursor (
    connection_id uuid NOT NULL,
    last_synced_at timestamp with time zone,
    last_attempted_at timestamp with time zone,
    cursor_version integer DEFAULT 1 NOT NULL,
    last_error text,
    created_at timestamp with time zone NOT NULL,
    updated_at timestamp with time zone NOT NULL,
    remote_page_id text,
    last_exported_highlight_created_at timestamp with time zone,
    last_exported_highlight_id uuid,
    last_delivered_at timestamp with time zone,
    last_exported_file_hash text,
    last_exported_full_document_hash text,
    generated_path text,
    generated_full_document_path text,
    explicit_reimport_requested_at timestamp with time zone,
    library_entry_id uuid NOT NULL
);

CREATE TABLE public.integration_oauth_tokens (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    user_id uuid NOT NULL,
    provider text NOT NULL,
    access_token_enc bytea NOT NULL,
    refresh_token_enc bytea,
    token_expires_at timestamp with time zone,
    extra jsonb DEFAULT '{}'::jsonb NOT NULL,
    created_at timestamp with time zone NOT NULL,
    updated_at timestamp with time zone NOT NULL
);

CREATE TABLE public.item_notes (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    user_id uuid NOT NULL,
    body text NOT NULL,
    created_at timestamp with time zone NOT NULL,
    updated_at timestamp with time zone NOT NULL,
    document_id uuid NOT NULL
);

CREATE TABLE public.job_outbox (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    job_type text NOT NULL,
    payload jsonb NOT NULL,
    dedupe_key text,
    available_at timestamp with time zone DEFAULT now() NOT NULL,
    dispatched_at timestamp with time zone,
    created_at timestamp with time zone NOT NULL
);

CREATE TABLE public.library_entries (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    user_id uuid NOT NULL,
    document_id uuid NOT NULL,
    saved_at timestamp with time zone DEFAULT now() NOT NULL,
    triage_state text DEFAULT 'inbox'::text NOT NULL,
    is_favorite boolean DEFAULT false NOT NULL,
    is_shortlisted boolean DEFAULT false NOT NULL,
    deleted_at timestamp with time zone,
    source text DEFAULT 'manual'::text NOT NULL,
    source_delivery_id uuid,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    updated_at timestamp with time zone DEFAULT now() NOT NULL,
    CONSTRAINT ck_library_entries_triage CHECK ((triage_state = ANY (ARRAY['inbox'::text, 'later'::text, 'archive'::text])))
);

CREATE TABLE public.library_entry_tags (
    user_id uuid NOT NULL,
    library_entry_id uuid NOT NULL,
    tag_id uuid NOT NULL,
    source text DEFAULT 'manual'::text NOT NULL,
    added_at timestamp with time zone DEFAULT now() NOT NULL
);

CREATE TABLE public.lifecycle_actions (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    user_id uuid NOT NULL,
    action_type text NOT NULL,
    triggered_at timestamp with time zone NOT NULL,
    executed_at timestamp with time zone,
    metadata jsonb
);

CREATE TABLE public.maintenance_tasks (
    task_name text NOT NULL,
    next_run_at timestamp with time zone DEFAULT now() NOT NULL,
    continuation_cursor text,
    lease_owner text,
    lease_expires_at timestamp with time zone,
    last_started_at timestamp with time zone,
    last_completed_at timestamp with time zone,
    last_error text,
    updated_at timestamp with time zone DEFAULT now() NOT NULL,
    CONSTRAINT ck_maintenance_tasks_lease CHECK ((((lease_owner IS NULL) AND (lease_expires_at IS NULL)) OR ((lease_owner IS NOT NULL) AND (lease_expires_at IS NOT NULL))))
);

CREATE TABLE public.mila_config (
    user_id uuid NOT NULL,
    chat_model text NOT NULL,
    embedding_model text NOT NULL,
    embedding_dim integer NOT NULL,
    chunk_size integer DEFAULT 512 NOT NULL,
    chunk_overlap integer DEFAULT 64 NOT NULL,
    top_k integer DEFAULT 6 NOT NULL,
    cross_item_top_k integer DEFAULT 20 NOT NULL,
    cross_item_max_per_item integer DEFAULT 3 NOT NULL,
    enabled boolean DEFAULT true NOT NULL,
    created_at timestamp with time zone NOT NULL,
    updated_at timestamp with time zone NOT NULL,
    supports_structured_output boolean DEFAULT true NOT NULL,
    model_context_window integer NOT NULL,
    chat_context_pct integer DEFAULT 70 NOT NULL,
    chat_api_base text DEFAULT 'https://api.openai.com/v1'::text NOT NULL,
    chat_api_key_enc bytea,
    embedding_api_base text DEFAULT 'https://api.openai.com/v1'::text NOT NULL,
    embedding_api_key_enc bytea,
    chat_cipher_version smallint DEFAULT 1 NOT NULL,
    embedding_cipher_version smallint DEFAULT 1 NOT NULL,
    byo_enabled boolean DEFAULT true NOT NULL,
    supports_reasoning_effort boolean DEFAULT false NOT NULL
);

CREATE TABLE public.mila_messages (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    session_id uuid NOT NULL,
    role text NOT NULL,
    content text NOT NULL,
    source_chunks uuid[] DEFAULT '{}'::uuid[] NOT NULL,
    created_at timestamp with time zone NOT NULL
);

CREATE TABLE public.mila_sessions (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    user_id uuid NOT NULL,
    session_type text NOT NULL,
    created_at timestamp with time zone NOT NULL,
    last_active timestamp with time zone NOT NULL,
    collection_id uuid,
    document_id uuid
);

CREATE TABLE public.notification_preferences (
    user_id uuid NOT NULL,
    email_digest boolean DEFAULT true NOT NULL,
    email_review_reminder boolean DEFAULT true NOT NULL,
    push_enabled boolean DEFAULT true NOT NULL,
    push_new_feed_items boolean DEFAULT false NOT NULL,
    push_ingestion_complete boolean DEFAULT false NOT NULL,
    push_review_reminder boolean DEFAULT true NOT NULL,
    in_app_enabled boolean DEFAULT true NOT NULL,
    updated_at timestamp with time zone NOT NULL,
    daily_review_reminder_enabled boolean DEFAULT true NOT NULL,
    daily_review_reminder_time text DEFAULT '09:00'::text NOT NULL,
    weekly_digest_enabled boolean DEFAULT true NOT NULL,
    new_highlights_sync boolean DEFAULT true NOT NULL,
    feed_updates boolean DEFAULT true NOT NULL,
    marketing_emails boolean DEFAULT false NOT NULL
);

CREATE TABLE public.notifications (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    user_id uuid NOT NULL,
    notification_type text NOT NULL,
    title text NOT NULL,
    body text,
    data jsonb,
    channels_sent text[] DEFAULT '{}'::text[] NOT NULL,
    read_at timestamp with time zone,
    created_at timestamp with time zone NOT NULL
);

CREATE TABLE public.notion_export_item_selection (
    connection_id uuid NOT NULL,
    selected boolean DEFAULT true NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    updated_at timestamp with time zone DEFAULT now() NOT NULL,
    library_entry_id uuid NOT NULL
);

CREATE TABLE public.oauth_flows (
    state_hash text NOT NULL,
    provider text NOT NULL,
    flow_kind text NOT NULL,
    sealed_flow bytea NOT NULL,
    used_at timestamp with time zone,
    expires_at timestamp with time zone NOT NULL,
    created_at timestamp with time zone NOT NULL
);

CREATE TABLE public.oauth_identities (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    user_id uuid NOT NULL,
    provider text NOT NULL,
    provider_user_id text NOT NULL,
    provider_email text,
    access_token_enc bytea,
    refresh_token_enc bytea,
    created_at timestamp with time zone NOT NULL
);

CREATE TABLE public.obsidian_export_artifact_items (
    artifact_id uuid NOT NULL,
    file_path text NOT NULL,
    full_document_path text,
    last_highlight_created_at timestamp with time zone,
    last_highlight_id uuid,
    content_hash text,
    full_document_hash text,
    delivered_at timestamp with time zone,
    last_error text,
    created_at timestamp with time zone NOT NULL,
    library_entry_id uuid NOT NULL
);

CREATE TABLE public.obsidian_export_artifacts (
    id uuid NOT NULL,
    run_id uuid NOT NULL,
    connection_id uuid NOT NULL,
    user_id uuid NOT NULL,
    content_type text NOT NULL,
    byte_size integer NOT NULL,
    bytes bytea NOT NULL,
    created_at timestamp with time zone NOT NULL
);

CREATE TABLE public.obsidian_export_refresh_queue (
    connection_id uuid NOT NULL,
    reason text NOT NULL,
    requested_at timestamp with time zone NOT NULL,
    delivery_attempts integer DEFAULT 0 NOT NULL,
    next_attempt_at timestamp with time zone,
    library_entry_id uuid NOT NULL
);

CREATE TABLE public.obsidian_export_runs (
    id uuid NOT NULL,
    connection_id uuid NOT NULL,
    user_id uuid NOT NULL,
    status text NOT NULL,
    total_documents integer DEFAULT 0 NOT NULL,
    documents_exported integer DEFAULT 0 NOT NULL,
    requested_by_user boolean DEFAULT false NOT NULL,
    auto boolean DEFAULT false NOT NULL,
    parent_folder_deleted boolean DEFAULT false NOT NULL,
    force_item_ids uuid[] DEFAULT '{}'::uuid[] NOT NULL,
    error text,
    created_at timestamp with time zone NOT NULL,
    updated_at timestamp with time zone NOT NULL,
    finished_at timestamp with time zone
);

CREATE TABLE public.password_reset_tokens (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    user_id uuid NOT NULL,
    token_hash text NOT NULL,
    expires_at timestamp with time zone NOT NULL,
    used_at timestamp with time zone,
    created_at timestamp with time zone NOT NULL
);

CREATE TABLE public.plans (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    slug text NOT NULL,
    name text NOT NULL,
    version integer NOT NULL,
    is_active boolean DEFAULT true NOT NULL,
    stripe_price_id_monthly text,
    stripe_price_id_annual text,
    entitlements jsonb NOT NULL,
    quotas jsonb NOT NULL,
    created_at timestamp with time zone NOT NULL
);

CREATE TABLE public.projector_cursors (
    projector_name text NOT NULL,
    last_seen_created_at timestamp with time zone,
    last_seen_event_id uuid,
    updated_at timestamp with time zone NOT NULL
);

CREATE TABLE public.push_tokens (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    user_id uuid NOT NULL,
    platform text NOT NULL,
    token text NOT NULL,
    device_name text,
    created_at timestamp with time zone NOT NULL,
    last_used_at timestamp with time zone
);

CREATE TABLE public.recent_searches (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    user_id uuid NOT NULL,
    raw_query text NOT NULL,
    normalized_query text NOT NULL,
    last_searched_at timestamp with time zone NOT NULL,
    created_at timestamp with time zone NOT NULL,
    updated_at timestamp with time zone NOT NULL
);

CREATE TABLE public.referral_credits (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    user_id uuid NOT NULL,
    source text NOT NULL,
    amount_cents integer NOT NULL,
    applied boolean DEFAULT false NOT NULL,
    created_at timestamp with time zone NOT NULL
);

CREATE TABLE public.refresh_tokens (
    id uuid NOT NULL,
    family_id uuid NOT NULL,
    user_id uuid NOT NULL,
    token_hash text NOT NULL,
    client_type text NOT NULL,
    ip_address inet,
    user_agent text,
    replaced_by uuid,
    revoked_at timestamp with time zone,
    expires_at timestamp with time zone NOT NULL,
    absolute_expires_at timestamp with time zone NOT NULL,
    last_used_at timestamp with time zone NOT NULL,
    created_at timestamp with time zone NOT NULL
);

CREATE TABLE public.review_cards (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    highlight_id uuid NOT NULL,
    user_id uuid NOT NULL,
    status text DEFAULT 'active'::text NOT NULL,
    current_interval integer DEFAULT 1 NOT NULL,
    repetition_count integer DEFAULT 0 NOT NULL,
    last_reviewed_at timestamp with time zone,
    next_due_at timestamp with time zone NOT NULL,
    created_at timestamp with time zone NOT NULL
);

CREATE TABLE public.review_events (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    card_id uuid NOT NULL,
    rating text NOT NULL,
    previous_interval integer NOT NULL,
    new_interval integer NOT NULL,
    reviewed_at timestamp with time zone NOT NULL
);

CREATE TABLE public.search_documents (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    user_id uuid NOT NULL,
    updated_at timestamp with time zone NOT NULL,
    document_kind text DEFAULT 'item'::text NOT NULL,
    section_key text DEFAULT ''::text NOT NULL,
    section_title text,
    title text DEFAULT ''::text NOT NULL,
    body_text text DEFAULT ''::text NOT NULL,
    highlight_text text DEFAULT ''::text NOT NULL,
    metadata_text text DEFAULT ''::text NOT NULL,
    search_config regconfig DEFAULT 'english'::regconfig NOT NULL,
    saved_at timestamp with time zone NOT NULL,
    document_tsv tsvector GENERATED ALWAYS AS (((((setweight(to_tsvector(search_config, COALESCE(title, ''::text)), 'A'::"char") || setweight(to_tsvector(search_config, COALESCE(section_title, ''::text)), 'B'::"char")) || setweight(to_tsvector(search_config, COALESCE(highlight_text, ''::text)), 'B'::"char")) || setweight(to_tsvector(search_config, COALESCE(metadata_text, ''::text)), 'C'::"char")) || setweight(to_tsvector(search_config, COALESCE(body_text, ''::text)), 'D'::"char"))) STORED,
    document_id uuid NOT NULL
)
WITH (autovacuum_vacuum_scale_factor='0.02', autovacuum_analyze_scale_factor='0.01', autovacuum_vacuum_threshold='50', autovacuum_analyze_threshold='50');

CREATE TABLE public.search_index_state (
    singleton boolean DEFAULT true NOT NULL,
    current_version integer DEFAULT 0 NOT NULL,
    updated_at timestamp with time zone DEFAULT now() NOT NULL,
    target_version integer,
    cursor_created_at timestamp with time zone,
    cursor_document_id uuid,
    CONSTRAINT ck_search_index_state_cursor_pair CHECK (((cursor_created_at IS NULL) = (cursor_document_id IS NULL))),
    CONSTRAINT search_index_state_current_version_check CHECK ((current_version >= 0)),
    CONSTRAINT search_index_state_singleton_check CHECK (singleton)
);

CREATE TABLE public.smart_lists (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    user_id uuid NOT NULL,
    name text NOT NULL,
    icon text,
    color text,
    is_pinned boolean DEFAULT false NOT NULL,
    filter_expression jsonb NOT NULL,
    default_sort text,
    created_at timestamp with time zone NOT NULL,
    updated_at timestamp with time zone NOT NULL
);

CREATE TABLE public.storage_add_ons (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    user_id uuid NOT NULL,
    bytes_purchased bigint NOT NULL,
    stripe_item_id text,
    created_at timestamp with time zone NOT NULL
);

CREATE TABLE public.subscriptions (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    billing_account_id uuid NOT NULL,
    plan_id uuid NOT NULL,
    stripe_subscription_id text,
    status text NOT NULL,
    current_period_start timestamp with time zone,
    current_period_end timestamp with time zone,
    cancel_at timestamp with time zone,
    created_at timestamp with time zone NOT NULL,
    updated_at timestamp with time zone NOT NULL
);

CREATE TABLE public.tag_aliases (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    tag_id uuid NOT NULL,
    alias text NOT NULL
);

CREATE TABLE public.tags (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    user_id uuid NOT NULL,
    name text NOT NULL,
    color text,
    parent_id uuid,
    created_at timestamp with time zone NOT NULL
);

CREATE TABLE public.tts_audio_assets (
    id uuid NOT NULL,
    user_id uuid NOT NULL,
    chunk_record_id uuid NOT NULL,
    s3_key text NOT NULL,
    content_type text NOT NULL,
    size_bytes bigint NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL
);

CREATE TABLE public.tts_chunks (
    id uuid NOT NULL,
    user_id uuid NOT NULL,
    document_id uuid NOT NULL,
    chunk_id text NOT NULL,
    cache_key text NOT NULL,
    voice_persona_id uuid,
    provider text NOT NULL,
    provider_model text,
    provider_voice_id text,
    pitch numeric(4,2) DEFAULT 1.00 NOT NULL,
    audio_format text NOT NULL,
    sample_rate integer NOT NULL,
    pronunciation_version integer DEFAULT 1 NOT NULL,
    chunking_version integer DEFAULT 1 NOT NULL,
    normalized_text_hash text NOT NULL,
    start_element_index integer NOT NULL,
    end_element_index integer NOT NULL,
    duration_seconds double precision,
    status text NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    updated_at timestamp with time zone DEFAULT now() NOT NULL
);

CREATE TABLE public.tts_element_timings (
    chunk_record_id uuid NOT NULL,
    element_index integer NOT NULL,
    start_timestamp double precision NOT NULL,
    end_timestamp double precision
);

CREATE TABLE public.tts_session_chunks (
    session_id uuid NOT NULL,
    chunk_id text NOT NULL,
    chunk_record_id uuid NOT NULL,
    "position" integer NOT NULL
);

CREATE TABLE public.tts_sessions (
    id uuid NOT NULL,
    user_id uuid NOT NULL,
    document_id uuid NOT NULL,
    voice_persona_id uuid,
    speed numeric(4,2) DEFAULT 1.00 NOT NULL,
    audio_format text DEFAULT 'mp3'::text NOT NULL,
    generation_scope text NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL
);

CREATE TABLE public.tts_voice_personas (
    id uuid NOT NULL,
    user_id uuid,
    display_name text NOT NULL,
    description text,
    provider text NOT NULL,
    provider_voice_id text,
    provider_model text,
    design_prompt text,
    style_prompt text,
    pace text,
    energy text,
    warmth text,
    formality text,
    pronunciation_prefs jsonb DEFAULT '{}'::jsonb NOT NULL,
    status text NOT NULL,
    is_builtin boolean DEFAULT false NOT NULL,
    prompt_hash text NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    updated_at timestamp with time zone DEFAULT now() NOT NULL,
    CONSTRAINT tts_voice_personas_check CHECK (((is_builtin = false) OR (user_id IS NULL)))
);

CREATE TABLE public.usage_counters (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    user_id uuid NOT NULL,
    quota_name text NOT NULL,
    period_start timestamp with time zone NOT NULL,
    period_end timestamp with time zone NOT NULL,
    current_value bigint DEFAULT 0 NOT NULL,
    limit_value bigint NOT NULL
);

CREATE TABLE public.user_document_state (
    user_id uuid NOT NULL,
    document_id uuid NOT NULL,
    progress_percent integer,
    max_progress_percent integer,
    scroll_position jsonb,
    chapter_locator text,
    chapter_offset integer,
    last_read_at timestamp with time zone,
    finished_at timestamp with time zone,
    first_opened_at timestamp with time zone,
    last_opened_at timestamp with time zone,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    updated_at timestamp with time zone DEFAULT now() NOT NULL
);

CREATE TABLE public.user_preferences (
    user_id uuid NOT NULL,
    accent_color text DEFAULT 'blue'::text NOT NULL,
    sidebar_mode text DEFAULT 'expanded'::text NOT NULL,
    default_view text DEFAULT 'library'::text NOT NULL,
    list_density text DEFAULT 'comfortable'::text NOT NULL,
    side_panel text DEFAULT 'auto'::text NOT NULL,
    triage_mode text DEFAULT 'focus'::text NOT NULL,
    auto_advance boolean DEFAULT true NOT NULL,
    reader_font_family text DEFAULT 'serif'::text NOT NULL,
    reader_font_size text DEFAULT 'medium'::text NOT NULL,
    reader_line_height text DEFAULT 'relaxed'::text NOT NULL,
    ai_mila_enabled boolean DEFAULT true NOT NULL,
    ai_custom_prompt text,
    archival_monolith boolean DEFAULT true NOT NULL,
    archival_pdf boolean DEFAULT false NOT NULL,
    archival_screenshot boolean DEFAULT true NOT NULL,
    archival_warc boolean DEFAULT false NOT NULL,
    duplicate_detection_enabled boolean DEFAULT true NOT NULL,
    duplicate_sensitivity text DEFAULT 'medium'::text NOT NULL,
    duplicate_action text DEFAULT 'notify_me'::text NOT NULL,
    browser_timeout_secs integer DEFAULT 90 NOT NULL,
    max_concurrent_archives integer DEFAULT 3 NOT NULL,
    ai_auto_processing boolean DEFAULT false NOT NULL,
    proxy_url text,
    proxy_all_requests boolean DEFAULT false NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    updated_at timestamp with time zone DEFAULT now() NOT NULL,
    home_widget_config jsonb,
    reader_email_open_mode text DEFAULT 'reader'::text NOT NULL,
    CONSTRAINT user_preferences_email_open_mode_check CHECK ((reader_email_open_mode = ANY (ARRAY['reader'::text, 'original'::text])))
);

CREATE TABLE public.users (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    email text NOT NULL,
    password_hash text,
    display_name text NOT NULL,
    avatar_url text,
    locale text DEFAULT 'en'::text NOT NULL,
    theme text DEFAULT 'system'::text NOT NULL,
    email_verified boolean DEFAULT false NOT NULL,
    onboarding_completed boolean DEFAULT false NOT NULL,
    onboarding_step smallint DEFAULT 0 NOT NULL,
    status text DEFAULT 'active'::text NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    updated_at timestamp with time zone DEFAULT now() NOT NULL,
    timezone text DEFAULT 'UTC'::text NOT NULL,
    email_token text DEFAULT "left"(replace((gen_random_uuid())::text, '-'::text, ''::text), 8) NOT NULL
);

CREATE TABLE public.webhook_deliveries (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    dispatch_id uuid NOT NULL,
    domain_event_id uuid NOT NULL,
    endpoint_id uuid NOT NULL,
    event_type text NOT NULL,
    payload jsonb NOT NULL,
    status_code integer,
    response_body text,
    attempt_number integer NOT NULL,
    delivered_at timestamp with time zone,
    next_retry_at timestamp with time zone,
    created_at timestamp with time zone NOT NULL,
    latency_ms integer
);

CREATE TABLE public.webhook_dispatches (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    domain_event_id uuid NOT NULL,
    endpoint_id uuid NOT NULL,
    event_type text NOT NULL,
    status text DEFAULT 'pending'::text NOT NULL,
    first_enqueued_at timestamp with time zone,
    delivered_at timestamp with time zone,
    exhausted_at timestamp with time zone,
    last_error text,
    created_at timestamp with time zone NOT NULL,
    updated_at timestamp with time zone NOT NULL
);

CREATE TABLE public.webhook_endpoints (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    user_id uuid NOT NULL,
    url text NOT NULL,
    secret_hash text NOT NULL,
    events text[] NOT NULL,
    is_active boolean DEFAULT true NOT NULL,
    created_at timestamp with time zone NOT NULL,
    updated_at timestamp with time zone NOT NULL,
    name text DEFAULT 'Webhook endpoint'::text NOT NULL,
    secret_ciphertext bytea,
    secret_preview text DEFAULT 'whsec_••••'::text NOT NULL
);

CREATE FUNCTION public.sync_document_search_configs() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
DECLARE
    target_config regconfig;
BEGIN
    target_config := public.fts_config_for_language(NEW.language);
    UPDATE public.content_vectors
    SET search_config = target_config
    WHERE document_id = NEW.id
      AND search_config IS DISTINCT FROM target_config;
    UPDATE public.search_documents
    SET search_config = target_config
    WHERE document_id = NEW.id
      AND search_config IS DISTINCT FROM target_config;
    RETURN NEW;
END;
$$;

CREATE TRIGGER trg_documents_search_configs
    AFTER UPDATE OF language ON public.documents
    FOR EACH ROW
    WHEN (OLD.language IS DISTINCT FROM NEW.language)
    EXECUTE FUNCTION public.sync_document_search_configs();
