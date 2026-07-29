ALTER TABLE ONLY apalis.jobs
    ADD CONSTRAINT jobs_pkey PRIMARY KEY (id);

ALTER TABLE ONLY apalis.workers
    ADD CONSTRAINT workers_pkey PRIMARY KEY (id);

ALTER TABLE ONLY public.authorization_codes
    ADD CONSTRAINT authorization_codes_code_hash_key UNIQUE (code_hash);

ALTER TABLE ONLY public.authorization_codes
    ADD CONSTRAINT authorization_codes_pkey PRIMARY KEY (id);

ALTER TABLE ONLY public.billing_usage_events
    ADD CONSTRAINT billing_usage_events_pkey PRIMARY KEY (id);

ALTER TABLE ONLY public.document_playback_states
    ADD CONSTRAINT document_playback_states_pkey PRIMARY KEY (user_id, document_id, playback_kind);

ALTER TABLE ONLY public.oauth_flows
    ADD CONSTRAINT oauth_flows_pkey PRIMARY KEY (state_hash);

ALTER TABLE ONLY public.obsidian_export_artifact_items
    ADD CONSTRAINT obsidian_export_artifact_items_pkey PRIMARY KEY (artifact_id, library_entry_id);

ALTER TABLE ONLY public.obsidian_export_artifacts
    ADD CONSTRAINT obsidian_export_artifacts_pkey PRIMARY KEY (id);

ALTER TABLE ONLY public.obsidian_export_refresh_queue
    ADD CONSTRAINT obsidian_export_refresh_queue_pkey PRIMARY KEY (connection_id, library_entry_id);

ALTER TABLE ONLY public.obsidian_export_runs
    ADD CONSTRAINT obsidian_export_runs_pkey PRIMARY KEY (id);

ALTER TABLE ONLY public.ai_outputs
    ADD CONSTRAINT pk_ai_outputs PRIMARY KEY (id);

ALTER TABLE ONLY public.ai_prompt_presets
    ADD CONSTRAINT pk_ai_prompt_presets PRIMARY KEY (id);

ALTER TABLE ONLY public.ai_runs
    ADD CONSTRAINT pk_ai_runs PRIMARY KEY (id);

ALTER TABLE ONLY public.api_tokens
    ADD CONSTRAINT pk_api_tokens PRIMARY KEY (id);

ALTER TABLE ONLY public.archive_assets
    ADD CONSTRAINT pk_archive_assets PRIMARY KEY (id);

ALTER TABLE ONLY public.background_job_recoveries
    ADD CONSTRAINT pk_background_job_recoveries PRIMARY KEY (id);

ALTER TABLE ONLY public.billing_account_members
    ADD CONSTRAINT pk_billing_account_members PRIMARY KEY (billing_account_id, user_id);

ALTER TABLE ONLY public.billing_accounts
    ADD CONSTRAINT pk_billing_accounts PRIMARY KEY (id);

ALTER TABLE ONLY public.billing_events
    ADD CONSTRAINT pk_billing_events PRIMARY KEY (id);

ALTER TABLE ONLY public.collection_entries
    ADD CONSTRAINT pk_collection_entries PRIMARY KEY (collection_id, library_entry_id);

ALTER TABLE ONLY public.collections
    ADD CONSTRAINT pk_collections PRIMARY KEY (id);

ALTER TABLE ONLY public.content_vectors
    ADD CONSTRAINT pk_content_vectors PRIMARY KEY (id);

ALTER TABLE ONLY public.dead_letter_jobs
    ADD CONSTRAINT pk_dead_letter_jobs PRIMARY KEY (id);

ALTER TABLE ONLY public.document_origins
    ADD CONSTRAINT pk_document_origins PRIMARY KEY (user_id, origin_type, origin_id);

ALTER TABLE ONLY public.document_video_metadata
    ADD CONSTRAINT pk_document_video_metadata PRIMARY KEY (document_id);

ALTER TABLE ONLY public.documents
    ADD CONSTRAINT pk_documents PRIMARY KEY (id);

ALTER TABLE ONLY public.domain_events
    ADD CONSTRAINT pk_domain_events PRIMARY KEY (id);

ALTER TABLE ONLY public.email_aliases
    ADD CONSTRAINT pk_email_aliases PRIMARY KEY (id);

ALTER TABLE ONLY public.email_ingest_log
    ADD CONSTRAINT pk_email_ingest_log PRIMARY KEY (id);

ALTER TABLE ONLY public.email_senders
    ADD CONSTRAINT pk_email_senders PRIMARY KEY (id);

ALTER TABLE ONLY public.email_unsubscribe_targets
    ADD CONSTRAINT pk_email_unsubscribe_targets PRIMARY KEY (sender_id);

ALTER TABLE ONLY public.email_verification_tokens
    ADD CONSTRAINT pk_email_verification_tokens PRIMARY KEY (id);

ALTER TABLE ONLY public.entities
    ADD CONSTRAINT pk_entities PRIMARY KEY (id);

ALTER TABLE ONLY public.entitlement_snapshots
    ADD CONSTRAINT pk_entitlement_snapshots PRIMARY KEY (id);

ALTER TABLE ONLY public.entity_aliases
    ADD CONSTRAINT pk_entity_aliases PRIMARY KEY (user_id, entity_type, name);

ALTER TABLE ONLY public.entity_mentions
    ADD CONSTRAINT pk_entity_mentions PRIMARY KEY (id);

ALTER TABLE ONLY public.feed_deliveries
    ADD CONSTRAINT pk_feed_deliveries PRIMARY KEY (id);

ALTER TABLE ONLY public.feed_provider_instances
    ADD CONSTRAINT pk_feed_provider_instances PRIMARY KEY (id);

ALTER TABLE ONLY public.feed_source_entries
    ADD CONSTRAINT pk_feed_source_entries PRIMARY KEY (id);

ALTER TABLE ONLY public.feed_sources
    ADD CONSTRAINT pk_feed_sources PRIMARY KEY (id);

ALTER TABLE ONLY public.feed_subscriptions
    ADD CONSTRAINT pk_feed_subscriptions PRIMARY KEY (id);

ALTER TABLE ONLY public.highlight_notes
    ADD CONSTRAINT pk_highlight_notes PRIMARY KEY (id);

ALTER TABLE ONLY public.highlight_tags
    ADD CONSTRAINT pk_highlight_tags PRIMARY KEY (highlight_id, tag_id);

ALTER TABLE ONLY public.highlights
    ADD CONSTRAINT pk_highlights PRIMARY KEY (id);

ALTER TABLE ONLY public.import_job_items
    ADD CONSTRAINT pk_import_job_items PRIMARY KEY (id);

ALTER TABLE ONLY public.import_jobs
    ADD CONSTRAINT pk_import_jobs PRIMARY KEY (id);

ALTER TABLE ONLY public.integration_connections
    ADD CONSTRAINT pk_integration_connections PRIMARY KEY (id);

ALTER TABLE ONLY public.integration_export_cursor
    ADD CONSTRAINT pk_integration_export_cursor PRIMARY KEY (connection_id, library_entry_id);

ALTER TABLE ONLY public.integration_oauth_tokens
    ADD CONSTRAINT pk_integration_oauth_tokens PRIMARY KEY (id);

ALTER TABLE ONLY public.item_notes
    ADD CONSTRAINT pk_item_notes PRIMARY KEY (id);

ALTER TABLE ONLY public.job_outbox
    ADD CONSTRAINT pk_job_outbox PRIMARY KEY (id);

ALTER TABLE ONLY public.library_entries
    ADD CONSTRAINT pk_library_entries PRIMARY KEY (id);

ALTER TABLE ONLY public.library_entry_tags
    ADD CONSTRAINT pk_library_entry_tags PRIMARY KEY (library_entry_id, tag_id);

ALTER TABLE ONLY public.lifecycle_actions
    ADD CONSTRAINT pk_lifecycle_actions PRIMARY KEY (id);

ALTER TABLE ONLY public.maintenance_tasks
    ADD CONSTRAINT pk_maintenance_tasks PRIMARY KEY (task_name);

ALTER TABLE ONLY public.mila_config
    ADD CONSTRAINT pk_mila_config PRIMARY KEY (user_id);

ALTER TABLE ONLY public.mila_messages
    ADD CONSTRAINT pk_mila_messages PRIMARY KEY (id);

ALTER TABLE ONLY public.mila_sessions
    ADD CONSTRAINT pk_mila_sessions PRIMARY KEY (id);

ALTER TABLE ONLY public.notification_preferences
    ADD CONSTRAINT pk_notification_preferences PRIMARY KEY (user_id);

ALTER TABLE ONLY public.notifications
    ADD CONSTRAINT pk_notifications PRIMARY KEY (id);

ALTER TABLE ONLY public.notion_export_item_selection
    ADD CONSTRAINT pk_notion_export_item_selection PRIMARY KEY (connection_id, library_entry_id);

ALTER TABLE ONLY public.oauth_identities
    ADD CONSTRAINT pk_oauth_identities PRIMARY KEY (id);

ALTER TABLE ONLY public.password_reset_tokens
    ADD CONSTRAINT pk_password_reset_tokens PRIMARY KEY (id);

ALTER TABLE ONLY public.plans
    ADD CONSTRAINT pk_plans PRIMARY KEY (id);

ALTER TABLE ONLY public.projector_cursors
    ADD CONSTRAINT pk_projector_cursors PRIMARY KEY (projector_name);

ALTER TABLE ONLY public.push_tokens
    ADD CONSTRAINT pk_push_tokens PRIMARY KEY (id);

ALTER TABLE ONLY public.recent_searches
    ADD CONSTRAINT pk_recent_searches PRIMARY KEY (id);

ALTER TABLE ONLY public.referral_credits
    ADD CONSTRAINT pk_referral_credits PRIMARY KEY (id);

ALTER TABLE ONLY public.review_cards
    ADD CONSTRAINT pk_review_cards PRIMARY KEY (id);

ALTER TABLE ONLY public.review_events
    ADD CONSTRAINT pk_review_events PRIMARY KEY (id);

ALTER TABLE ONLY public.search_documents
    ADD CONSTRAINT pk_search_documents PRIMARY KEY (id);

ALTER TABLE ONLY public.smart_lists
    ADD CONSTRAINT pk_smart_lists PRIMARY KEY (id);

ALTER TABLE ONLY public.storage_add_ons
    ADD CONSTRAINT pk_storage_add_ons PRIMARY KEY (id);

ALTER TABLE ONLY public.subscriptions
    ADD CONSTRAINT pk_subscriptions PRIMARY KEY (id);

ALTER TABLE ONLY public.tag_aliases
    ADD CONSTRAINT pk_tag_aliases PRIMARY KEY (id);

ALTER TABLE ONLY public.tags
    ADD CONSTRAINT pk_tags PRIMARY KEY (id);

ALTER TABLE ONLY public.usage_counters
    ADD CONSTRAINT pk_usage_counters PRIMARY KEY (id);

ALTER TABLE ONLY public.user_document_state
    ADD CONSTRAINT pk_user_document_state PRIMARY KEY (user_id, document_id);

ALTER TABLE ONLY public.users
    ADD CONSTRAINT pk_users PRIMARY KEY (id);

ALTER TABLE ONLY public.webhook_deliveries
    ADD CONSTRAINT pk_webhook_deliveries PRIMARY KEY (id);

ALTER TABLE ONLY public.webhook_dispatches
    ADD CONSTRAINT pk_webhook_dispatches PRIMARY KEY (id);

ALTER TABLE ONLY public.webhook_endpoints
    ADD CONSTRAINT pk_webhook_endpoints PRIMARY KEY (id);

ALTER TABLE ONLY public.refresh_tokens
    ADD CONSTRAINT refresh_tokens_pkey PRIMARY KEY (id);

ALTER TABLE ONLY public.refresh_tokens
    ADD CONSTRAINT refresh_tokens_token_hash_key UNIQUE (token_hash);

ALTER TABLE ONLY public.search_index_state
    ADD CONSTRAINT search_index_state_pkey PRIMARY KEY (singleton);

ALTER TABLE ONLY public.tts_audio_assets
    ADD CONSTRAINT tts_audio_assets_chunk_record_id_key UNIQUE (chunk_record_id);

ALTER TABLE ONLY public.tts_audio_assets
    ADD CONSTRAINT tts_audio_assets_pkey PRIMARY KEY (id);

ALTER TABLE ONLY public.tts_chunks
    ADD CONSTRAINT tts_chunks_pkey PRIMARY KEY (id);

ALTER TABLE ONLY public.tts_chunks
    ADD CONSTRAINT tts_chunks_user_id_cache_key_key UNIQUE (user_id, cache_key);

ALTER TABLE ONLY public.tts_element_timings
    ADD CONSTRAINT tts_element_timings_pkey PRIMARY KEY (chunk_record_id, element_index);

ALTER TABLE ONLY public.tts_session_chunks
    ADD CONSTRAINT tts_session_chunks_pkey PRIMARY KEY (session_id, chunk_id);

ALTER TABLE ONLY public.tts_sessions
    ADD CONSTRAINT tts_sessions_pkey PRIMARY KEY (id);

ALTER TABLE ONLY public.tts_voice_personas
    ADD CONSTRAINT tts_voice_personas_pkey PRIMARY KEY (id);

ALTER TABLE ONLY public.api_tokens
    ADD CONSTRAINT uq_api_tokens_token_hash UNIQUE (token_hash);

ALTER TABLE ONLY public.background_job_recoveries
    ADD CONSTRAINT uq_background_job_recoveries_recovery_key UNIQUE (recovery_key);

ALTER TABLE ONLY public.billing_accounts
    ADD CONSTRAINT uq_billing_accounts_stripe UNIQUE (stripe_customer_id);

ALTER TABLE ONLY public.billing_events
    ADD CONSTRAINT uq_billing_events_stripe UNIQUE (stripe_event_id);

ALTER TABLE ONLY public.billing_usage_events
    ADD CONSTRAINT uq_billing_usage_events_idempotency UNIQUE (idempotency_key);

ALTER TABLE ONLY public.collections
    ADD CONSTRAINT uq_collections_rss_token UNIQUE (rss_token);

ALTER TABLE ONLY public.documents
    ADD CONSTRAINT uq_documents_id_user UNIQUE (id, user_id);

ALTER TABLE ONLY public.email_aliases
    ADD CONSTRAINT uq_email_aliases_dest_local UNIQUE (destination, local_part);

ALTER TABLE ONLY public.email_ingest_log
    ADD CONSTRAINT uq_email_ingest_delivery UNIQUE (provider, provider_email_id, user_id, destination);

ALTER TABLE ONLY public.email_senders
    ADD CONSTRAINT uq_email_senders_id_user UNIQUE (id, user_id);

ALTER TABLE ONLY public.email_senders
    ADD CONSTRAINT uq_email_senders_user_canonical UNIQUE (user_id, canonical_addr);

ALTER TABLE ONLY public.email_verification_tokens
    ADD CONSTRAINT uq_email_verification_token_hash UNIQUE (token_hash);

ALTER TABLE ONLY public.entities
    ADD CONSTRAINT uq_entities_user_name_type UNIQUE (user_id, name, entity_type);

ALTER TABLE ONLY public.entitlement_snapshots
    ADD CONSTRAINT uq_entitlement_snapshots_user UNIQUE (user_id);

ALTER TABLE ONLY public.feed_provider_instances
    ADD CONSTRAINT uq_feed_provider_instances_type_url UNIQUE (provider_type, base_url);

ALTER TABLE ONLY public.feed_source_entries
    ADD CONSTRAINT uq_feed_source_entries_source_guid UNIQUE (source_id, guid);

ALTER TABLE ONLY public.feed_sources
    ADD CONSTRAINT uq_feed_sources_canonical_key UNIQUE (canonical_key);

ALTER TABLE ONLY public.highlight_notes
    ADD CONSTRAINT uq_highlight_notes_highlight UNIQUE (highlight_id);

ALTER TABLE ONLY public.integration_connections
    ADD CONSTRAINT uq_integration_connections_user_provider UNIQUE (user_id, provider);

ALTER TABLE ONLY public.integration_oauth_tokens
    ADD CONSTRAINT uq_integration_oauth_tokens_user_provider UNIQUE (user_id, provider);

ALTER TABLE ONLY public.oauth_identities
    ADD CONSTRAINT uq_oauth_provider_user UNIQUE (provider, provider_user_id);

ALTER TABLE ONLY public.password_reset_tokens
    ADD CONSTRAINT uq_password_reset_token_hash UNIQUE (token_hash);

ALTER TABLE ONLY public.plans
    ADD CONSTRAINT uq_plans_slug UNIQUE (slug);

ALTER TABLE ONLY public.push_tokens
    ADD CONSTRAINT uq_push_tokens_user_token UNIQUE (user_id, token);

ALTER TABLE ONLY public.recent_searches
    ADD CONSTRAINT uq_recent_searches_user_query UNIQUE (user_id, normalized_query);

ALTER TABLE ONLY public.review_cards
    ADD CONSTRAINT uq_review_cards_highlight UNIQUE (highlight_id);

ALTER TABLE ONLY public.subscriptions
    ADD CONSTRAINT uq_subscriptions_stripe UNIQUE (stripe_subscription_id);

ALTER TABLE ONLY public.tag_aliases
    ADD CONSTRAINT uq_tag_aliases_tag_alias UNIQUE (tag_id, alias);

ALTER TABLE ONLY public.usage_counters
    ADD CONSTRAINT uq_usage_counters_user_quota_period UNIQUE (user_id, quota_name, period_start);

ALTER TABLE ONLY public.users
    ADD CONSTRAINT uq_users_email UNIQUE (email);

ALTER TABLE ONLY public.webhook_dispatches
    ADD CONSTRAINT uq_webhook_dispatches_event_endpoint UNIQUE (domain_event_id, endpoint_id);

ALTER TABLE ONLY public.user_preferences
    ADD CONSTRAINT user_preferences_pkey PRIMARY KEY (user_id);

CREATE INDEX apalis_jobs_job_type_idx ON apalis.jobs USING btree (job_type);

CREATE INDEX apalis_jobs_lock_by_idx ON apalis.jobs USING btree (lock_by);

CREATE INDEX apalis_jobs_status_idx ON apalis.jobs USING btree (status);

CREATE INDEX apalis_workers_last_seen_idx ON apalis.workers USING btree (last_seen);

CREATE INDEX apalis_workers_worker_type_idx ON apalis.workers USING btree (worker_type);

CREATE INDEX idx_ai_prompt_presets_user ON public.ai_prompt_presets USING btree (user_id);

CREATE INDEX idx_ai_runs_document ON public.ai_runs USING btree (document_id) WHERE (document_id IS NOT NULL);

CREATE INDEX idx_ai_runs_user ON public.ai_runs USING btree (user_id);

CREATE INDEX idx_api_tokens_user ON public.api_tokens USING btree (user_id);

CREATE INDEX idx_archive_assets_document ON public.archive_assets USING btree (document_id);

CREATE INDEX idx_background_job_recoveries_due ON public.background_job_recoveries USING btree (next_retry_at, lease_expires_at) WHERE (status = ANY (ARRAY['waiting'::text, 'leased'::text]));

CREATE INDEX idx_background_job_recoveries_status_job_type ON public.background_job_recoveries USING btree (status, job_type, updated_at);

CREATE INDEX idx_background_job_recoveries_subject ON public.background_job_recoveries USING btree (subject_kind, subject_id, status) WHERE (subject_kind IS NOT NULL);

CREATE INDEX idx_bam_user ON public.billing_account_members USING btree (user_id);

CREATE INDEX idx_billing_accounts_owner ON public.billing_accounts USING btree (owner_user_id);

CREATE INDEX idx_billing_usage_events_product_time ON public.billing_usage_events USING btree (product_area, event_type, occurred_at DESC);

CREATE INDEX idx_billing_usage_events_user_time ON public.billing_usage_events USING btree (user_id, occurred_at DESC);

CREATE INDEX idx_collection_entries_library_entry ON public.collection_entries USING btree (library_entry_id);

CREATE INDEX idx_collections_parent ON public.collections USING btree (parent_id);

CREATE INDEX idx_collections_user ON public.collections USING btree (user_id);

CREATE INDEX idx_collections_user_pinned ON public.collections USING btree (user_id, is_pinned) WHERE (is_pinned = true);

CREATE INDEX idx_content_vectors_embedding ON public.content_vectors USING hnsw (embedding public.vector_cosine_ops);

CREATE INDEX idx_content_vectors_user ON public.content_vectors USING btree (user_id);

CREATE INDEX idx_content_vectors_user_content_tsv ON public.content_vectors USING gin (user_id, content_tsv) WITH (fastupdate='on', gin_pending_list_limit='4096');

CREATE INDEX idx_content_vectors_user_embedding_identity ON public.content_vectors USING btree (user_id, embedding_model, embedding_dim, document_id);

CREATE INDEX idx_dead_letter_jobs_unresolved ON public.dead_letter_jobs USING btree (failed_at DESC, id DESC) WHERE (replayed_at IS NULL);

CREATE INDEX idx_deliveries_dispatch ON public.webhook_deliveries USING btree (dispatch_id, created_at DESC);

CREATE INDEX idx_deliveries_endpoint ON public.webhook_deliveries USING btree (endpoint_id, created_at DESC);

CREATE INDEX idx_deliveries_retry ON public.webhook_deliveries USING btree (next_retry_at) WHERE ((status_code IS NULL) OR (status_code >= 400));

CREATE INDEX idx_document_origins_document ON public.document_origins USING btree (document_id);

CREATE INDEX idx_document_playback_states_document ON public.document_playback_states USING btree (user_id, document_id);

CREATE INDEX idx_documents_user_author_lower ON public.documents USING btree (user_id, lower(author) text_pattern_ops) WHERE ((author IS NOT NULL) AND (author <> ''::text));

CREATE INDEX idx_documents_user_content_hash ON public.documents USING btree (user_id, content_hash) WHERE (content_hash IS NOT NULL);

CREATE INDEX idx_documents_user_sender ON public.documents USING btree (user_id, sender_id) WHERE (sender_id IS NOT NULL);

CREATE INDEX idx_email_aliases_lookup ON public.email_aliases USING btree (destination, local_part) WHERE (status = 'active'::text);

CREATE INDEX idx_email_aliases_retire ON public.email_aliases USING btree (retire_at) WHERE ((status = 'active'::text) AND (retire_at IS NOT NULL));

CREATE INDEX idx_email_senders_user_blocked ON public.email_senders USING btree (user_id, blocked_at) WHERE (blocked_at IS NOT NULL);

CREATE INDEX idx_email_senders_user_list_id ON public.email_senders USING btree (user_id, list_id) WHERE (list_id IS NOT NULL);

CREATE INDEX idx_email_verification_user ON public.email_verification_tokens USING btree (user_id);

CREATE INDEX idx_entities_norm ON public.entities USING btree (user_id, entity_type, lower(btrim(name)));

CREATE INDEX idx_entities_user_type ON public.entities USING btree (user_id, entity_type);

CREATE INDEX idx_entity_aliases_entity ON public.entity_aliases USING btree (entity_id);

CREATE INDEX idx_entity_mentions_document ON public.entity_mentions USING btree (document_id) WHERE (document_id IS NOT NULL);

CREATE INDEX idx_events_aggregate ON public.domain_events USING btree (aggregate_type, aggregate_id);

CREATE INDEX idx_events_type ON public.domain_events USING btree (event_type, created_at);

CREATE INDEX idx_events_user ON public.domain_events USING btree (user_id, created_at DESC);

CREATE INDEX idx_events_user_id_asc ON public.domain_events USING btree (user_id, id);

CREATE INDEX idx_export_cursor_highlight_cursor ON public.integration_export_cursor USING btree (connection_id, library_entry_id, last_exported_highlight_created_at, last_exported_highlight_id);

CREATE INDEX idx_export_cursor_stale ON public.integration_export_cursor USING btree (connection_id, last_attempted_at);

CREATE INDEX idx_feed_deliveries_adoption ON public.feed_deliveries USING btree (user_id, source_entry_id) WHERE (document_id IS NULL);

CREATE INDEX idx_feed_deliveries_document ON public.feed_deliveries USING btree (document_id);

CREATE INDEX idx_feed_deliveries_seen ON public.feed_deliveries USING btree (user_id, seen_at DESC) WHERE ((seen_at IS NOT NULL) AND (dismissed_at IS NULL) AND (hidden_at IS NULL));

CREATE INDEX idx_feed_deliveries_unseen ON public.feed_deliveries USING btree (user_id, delivered_at DESC) WHERE ((seen_at IS NULL) AND (dismissed_at IS NULL) AND (hidden_at IS NULL));

CREATE INDEX idx_feed_deliveries_user_entry_delivered ON public.feed_deliveries USING btree (user_id, source_entry_id, delivered_at DESC) WHERE (hidden_at IS NULL);

CREATE INDEX idx_feed_provider_instances_lookup ON public.feed_provider_instances USING btree (provider_type, enabled, priority, consecutive_failures);

CREATE INDEX idx_feed_source_entries_canonical_url ON public.feed_source_entries USING btree (canonical_url) WHERE (canonical_url IS NOT NULL);

CREATE INDEX idx_feed_source_entries_search_tsv ON public.feed_source_entries USING gin (search_tsv);

CREATE INDEX idx_feed_sources_lease ON public.feed_sources USING btree (lease_expires_at) WHERE (lease_expires_at IS NOT NULL);

CREATE INDEX idx_feed_sources_next_poll_at ON public.feed_sources USING btree (next_poll_at);

CREATE INDEX idx_feed_sources_visibility_type_popularity ON public.feed_sources USING btree (visibility, feed_type, popularity DESC, updated_at DESC);

CREATE INDEX idx_feeds_user_status ON public.feed_subscriptions USING btree (user_id, status);

CREATE INDEX idx_highlight_tags_tag ON public.highlight_tags USING btree (tag_id);

CREATE INDEX idx_highlights_user ON public.highlights USING btree (user_id, created_at DESC);

CREATE INDEX idx_highlights_user_document ON public.highlights USING btree (user_id, document_id, created_at) WHERE (document_id IS NOT NULL);

CREATE INDEX idx_import_job_items_job ON public.import_job_items USING btree (import_job_id);

CREATE INDEX idx_import_jobs_user_created ON public.import_jobs USING btree (user_id, created_at DESC);

CREATE INDEX idx_integration_connections_user ON public.integration_connections USING btree (user_id);

CREATE INDEX idx_integration_oauth_tokens_user ON public.integration_oauth_tokens USING btree (user_id);

CREATE INDEX idx_item_notes_user ON public.item_notes USING btree (user_id);

CREATE INDEX idx_job_outbox_pending ON public.job_outbox USING btree (available_at, created_at) WHERE (dispatched_at IS NULL);

CREATE INDEX idx_library_entries_user_saved ON public.library_entries USING btree (user_id, saved_at DESC) WHERE (deleted_at IS NULL);

CREATE INDEX idx_library_entries_user_triage ON public.library_entries USING btree (user_id, triage_state, saved_at DESC) WHERE (deleted_at IS NULL);

CREATE INDEX idx_library_entry_tags_tag ON public.library_entry_tags USING btree (tag_id);

CREATE INDEX idx_lifecycle_actions_user ON public.lifecycle_actions USING btree (user_id);

CREATE INDEX idx_maintenance_tasks_due ON public.maintenance_tasks USING btree (next_run_at, task_name) WHERE (lease_owner IS NULL);

CREATE INDEX idx_mila_messages_session ON public.mila_messages USING btree (session_id, created_at);

CREATE INDEX idx_mila_sessions_collection ON public.mila_sessions USING btree (collection_id);

CREATE INDEX idx_mila_sessions_user ON public.mila_sessions USING btree (user_id);

CREATE INDEX idx_mila_sessions_user_document ON public.mila_sessions USING btree (user_id, document_id) WHERE (document_id IS NOT NULL);

CREATE INDEX idx_notifications_user ON public.notifications USING btree (user_id, created_at DESC);

CREATE INDEX idx_notifications_user_unread ON public.notifications USING btree (user_id, read_at) WHERE (read_at IS NULL);

CREATE INDEX idx_notion_export_item_selection_selected ON public.notion_export_item_selection USING btree (connection_id, selected, library_entry_id);

CREATE INDEX idx_oauth_flows_expires ON public.oauth_flows USING btree (expires_at);

CREATE INDEX idx_oauth_user ON public.oauth_identities USING btree (user_id);

CREATE INDEX idx_obsidian_export_artifacts_run ON public.obsidian_export_artifacts USING btree (run_id, created_at);

CREATE INDEX idx_obsidian_export_runs_connection_created ON public.obsidian_export_runs USING btree (connection_id, created_at DESC);

CREATE INDEX idx_password_reset_user ON public.password_reset_tokens USING btree (user_id);

CREATE INDEX idx_push_user ON public.push_tokens USING btree (user_id);

CREATE INDEX idx_recent_searches_user_last_searched ON public.recent_searches USING btree (user_id, last_searched_at DESC);

CREATE INDEX idx_referral_credits_user ON public.referral_credits USING btree (user_id);

CREATE INDEX idx_refresh_tokens_expires ON public.refresh_tokens USING btree (expires_at);

CREATE INDEX idx_refresh_tokens_family ON public.refresh_tokens USING btree (family_id);

CREATE INDEX idx_refresh_tokens_user ON public.refresh_tokens USING btree (user_id);

CREATE INDEX idx_review_events_card ON public.review_events USING btree (card_id);

CREATE INDEX idx_review_user_due ON public.review_cards USING btree (user_id, status, next_due_at);

CREATE INDEX idx_search_documents_user_tsv ON public.search_documents USING gin (user_id, document_tsv) WITH (fastupdate='on', gin_pending_list_limit='4096');

CREATE INDEX idx_smart_lists_user ON public.smart_lists USING btree (user_id);

CREATE INDEX idx_storage_add_ons_user ON public.storage_add_ons USING btree (user_id);

CREATE INDEX idx_subscriptions_account ON public.subscriptions USING btree (billing_account_id);

CREATE INDEX idx_subscriptions_plan ON public.subscriptions USING btree (plan_id);

CREATE INDEX idx_tag_aliases_tag ON public.tag_aliases USING btree (tag_id);

CREATE INDEX idx_tags_parent ON public.tags USING btree (parent_id);

CREATE INDEX idx_tags_user ON public.tags USING btree (user_id);

CREATE INDEX idx_tts_chunks_document ON public.tts_chunks USING btree (user_id, document_id);

CREATE INDEX idx_tts_sessions_user_document ON public.tts_sessions USING btree (user_id, document_id);

CREATE INDEX idx_tts_voice_personas_builtin ON public.tts_voice_personas USING btree (is_builtin) WHERE (is_builtin = true);

CREATE INDEX idx_tts_voice_personas_user ON public.tts_voice_personas USING btree (user_id);

CREATE INDEX idx_usage_counters_user ON public.usage_counters USING btree (user_id);

CREATE UNIQUE INDEX idx_users_email_token ON public.users USING btree (email_token);

CREATE INDEX idx_users_status ON public.users USING btree (status);

CREATE INDEX idx_webhook_dispatches_endpoint ON public.webhook_dispatches USING btree (endpoint_id, status, created_at DESC);

CREATE INDEX idx_webhook_dispatches_event ON public.webhook_dispatches USING btree (domain_event_id);

CREATE INDEX idx_webhook_endpoints_user ON public.webhook_endpoints USING btree (user_id);

CREATE INDEX idx_webhook_projector_scan ON public.domain_events USING btree (created_at, id);

CREATE UNIQUE INDEX uq_ai_outputs_document_type ON public.ai_outputs USING btree (document_id, output_type) WHERE (document_id IS NOT NULL);

CREATE UNIQUE INDEX uq_archive_assets_document_kind ON public.archive_assets USING btree (document_id, asset_kind) WHERE (document_id IS NOT NULL);

CREATE UNIQUE INDEX uq_collections_id_user ON public.collections USING btree (id, user_id);

CREATE UNIQUE INDEX uq_content_vectors_document_section_chunk ON public.content_vectors USING btree (document_id, section_key, chunk_index) WHERE (document_id IS NOT NULL);

CREATE UNIQUE INDEX uq_documents_user_canonical_url ON public.documents USING btree (user_id, canonical_url) WHERE (canonical_url IS NOT NULL);

CREATE UNIQUE INDEX uq_documents_user_content_hash_no_url ON public.documents USING btree (user_id, content_hash) WHERE ((canonical_url IS NULL) AND (content_hash IS NOT NULL));

CREATE UNIQUE INDEX uq_email_aliases_default_per_user_destination ON public.email_aliases USING btree (user_id, destination) WHERE (is_default AND (status = 'active'::text));

CREATE UNIQUE INDEX uq_entity_mentions_entity_document ON public.entity_mentions USING btree (entity_id, document_id) WHERE (document_id IS NOT NULL);

CREATE UNIQUE INDEX uq_feed_deliveries_user_sub_entry ON public.feed_deliveries USING btree (user_id, subscription_id, source_entry_id);

CREATE UNIQUE INDEX uq_feed_subscriptions_user_source ON public.feed_subscriptions USING btree (user_id, source_id);

CREATE UNIQUE INDEX uq_item_notes_user_document ON public.item_notes USING btree (user_id, document_id) WHERE (document_id IS NOT NULL);

CREATE UNIQUE INDEX uq_job_outbox_dedupe ON public.job_outbox USING btree (dedupe_key) WHERE (dedupe_key IS NOT NULL);

CREATE UNIQUE INDEX uq_library_entries_id_user ON public.library_entries USING btree (id, user_id);

CREATE UNIQUE INDEX uq_library_entries_user_document_active ON public.library_entries USING btree (user_id, document_id) WHERE (deleted_at IS NULL);

CREATE UNIQUE INDEX uq_search_documents_document_section ON public.search_documents USING btree (document_id, section_key) WHERE (document_id IS NOT NULL);

CREATE UNIQUE INDEX uq_tags_id_user ON public.tags USING btree (id, user_id);

CREATE UNIQUE INDEX uq_tags_user_lower_name ON public.tags USING btree (user_id, lower(name));

CREATE TRIGGER notify_workers AFTER INSERT ON apalis.jobs FOR EACH ROW EXECUTE FUNCTION apalis.notify_new_jobs();

CREATE TRIGGER trg_domain_events_notify_insert AFTER INSERT ON public.domain_events FOR EACH ROW EXECUTE FUNCTION public.notify_domain_events_insert();

CREATE TRIGGER trg_feed_source_entries_tsv BEFORE INSERT OR UPDATE OF title, author, excerpt, content_html, language ON public.feed_source_entries FOR EACH ROW EXECUTE FUNCTION public.feed_source_entries_tsv_update();

ALTER TABLE ONLY apalis.jobs
    ADD CONSTRAINT jobs_lock_by_fkey FOREIGN KEY (lock_by) REFERENCES apalis.workers(id);

ALTER TABLE ONLY public.authorization_codes
    ADD CONSTRAINT authorization_codes_user_id_fkey FOREIGN KEY (user_id) REFERENCES public.users(id) ON DELETE CASCADE;

ALTER TABLE ONLY public.billing_usage_events
    ADD CONSTRAINT billing_usage_events_billing_account_id_fkey FOREIGN KEY (billing_account_id) REFERENCES public.billing_accounts(id) ON DELETE SET NULL;

ALTER TABLE ONLY public.billing_usage_events
    ADD CONSTRAINT billing_usage_events_user_id_fkey FOREIGN KEY (user_id) REFERENCES public.users(id) ON DELETE CASCADE;

ALTER TABLE ONLY public.document_playback_states
    ADD CONSTRAINT document_playback_states_tts_voice_persona_id_fkey FOREIGN KEY (tts_voice_persona_id) REFERENCES public.tts_voice_personas(id) ON DELETE SET NULL;

ALTER TABLE ONLY public.document_playback_states
    ADD CONSTRAINT document_playback_states_user_id_fkey FOREIGN KEY (user_id) REFERENCES public.users(id) ON DELETE CASCADE;

ALTER TABLE ONLY public.email_ingest_log
    ADD CONSTRAINT email_ingest_log_user_id_fkey FOREIGN KEY (user_id) REFERENCES public.users(id) ON DELETE CASCADE;

ALTER TABLE ONLY public.entity_aliases
    ADD CONSTRAINT entity_aliases_entity_id_fkey FOREIGN KEY (entity_id) REFERENCES public.entities(id) ON DELETE CASCADE;

ALTER TABLE ONLY public.entity_aliases
    ADD CONSTRAINT entity_aliases_user_id_fkey FOREIGN KEY (user_id) REFERENCES public.users(id) ON DELETE CASCADE;

ALTER TABLE ONLY public.ai_outputs
    ADD CONSTRAINT fk_ai_outputs_document FOREIGN KEY (document_id) REFERENCES public.documents(id) ON DELETE CASCADE;

ALTER TABLE ONLY public.ai_outputs
    ADD CONSTRAINT fk_ai_outputs_run FOREIGN KEY (ai_run_id) REFERENCES public.ai_runs(id) ON DELETE SET NULL;

ALTER TABLE ONLY public.ai_prompt_presets
    ADD CONSTRAINT fk_ai_prompt_presets_user FOREIGN KEY (user_id) REFERENCES public.users(id) ON DELETE CASCADE;

ALTER TABLE ONLY public.ai_runs
    ADD CONSTRAINT fk_ai_runs_document FOREIGN KEY (document_id) REFERENCES public.documents(id) ON DELETE SET NULL;

ALTER TABLE ONLY public.ai_runs
    ADD CONSTRAINT fk_ai_runs_user FOREIGN KEY (user_id) REFERENCES public.users(id) ON DELETE CASCADE;

ALTER TABLE ONLY public.api_tokens
    ADD CONSTRAINT fk_api_tokens_user FOREIGN KEY (user_id) REFERENCES public.users(id) ON DELETE CASCADE;

ALTER TABLE ONLY public.archive_assets
    ADD CONSTRAINT fk_archive_assets_document FOREIGN KEY (document_id) REFERENCES public.documents(id) ON DELETE CASCADE;

ALTER TABLE ONLY public.billing_account_members
    ADD CONSTRAINT fk_bam_account FOREIGN KEY (billing_account_id) REFERENCES public.billing_accounts(id) ON DELETE CASCADE;

ALTER TABLE ONLY public.billing_account_members
    ADD CONSTRAINT fk_bam_plan_override FOREIGN KEY (plan_override_id) REFERENCES public.plans(id);

ALTER TABLE ONLY public.billing_account_members
    ADD CONSTRAINT fk_bam_user FOREIGN KEY (user_id) REFERENCES public.users(id) ON DELETE CASCADE;

-- Billing rows are deleted with the owning account: billing is not a live
-- feature (no HTTP routes consume these tables). An operator enabling real
-- billing must revisit retention here before relying on these tables as
-- financial records.
ALTER TABLE ONLY public.billing_accounts
    ADD CONSTRAINT fk_billing_accounts_owner FOREIGN KEY (owner_user_id) REFERENCES public.users(id) ON DELETE CASCADE;

ALTER TABLE ONLY public.collection_entries
    ADD CONSTRAINT fk_collection_entries_collection FOREIGN KEY (collection_id, user_id) REFERENCES public.collections(id, user_id) ON DELETE CASCADE;

ALTER TABLE ONLY public.collection_entries
    ADD CONSTRAINT fk_collection_entries_library_entry FOREIGN KEY (library_entry_id, user_id) REFERENCES public.library_entries(id, user_id) ON DELETE CASCADE;

ALTER TABLE ONLY public.collections
    ADD CONSTRAINT fk_collections_parent FOREIGN KEY (parent_id) REFERENCES public.collections(id);

ALTER TABLE ONLY public.collections
    ADD CONSTRAINT fk_collections_user FOREIGN KEY (user_id) REFERENCES public.users(id) ON DELETE CASCADE;

ALTER TABLE ONLY public.content_vectors
    ADD CONSTRAINT fk_content_vectors_document FOREIGN KEY (document_id, user_id) REFERENCES public.documents(id, user_id) ON DELETE CASCADE;

ALTER TABLE ONLY public.content_vectors
    ADD CONSTRAINT fk_content_vectors_user FOREIGN KEY (user_id) REFERENCES public.users(id) ON DELETE CASCADE;

ALTER TABLE ONLY public.dead_letter_jobs
    ADD CONSTRAINT fk_dead_letter_jobs_replay_outbox FOREIGN KEY (replay_outbox_id) REFERENCES public.job_outbox(id);

ALTER TABLE ONLY public.document_origins
    ADD CONSTRAINT fk_document_origins_document FOREIGN KEY (document_id, user_id) REFERENCES public.documents(id, user_id) ON DELETE CASCADE;

ALTER TABLE ONLY public.document_origins
    ADD CONSTRAINT fk_document_origins_user FOREIGN KEY (user_id) REFERENCES public.users(id) ON DELETE CASCADE;

ALTER TABLE ONLY public.document_playback_states
    ADD CONSTRAINT fk_document_playback_states_document FOREIGN KEY (document_id, user_id) REFERENCES public.documents(id, user_id) ON DELETE CASCADE;

ALTER TABLE ONLY public.document_video_metadata
    ADD CONSTRAINT fk_document_video_metadata_document FOREIGN KEY (document_id) REFERENCES public.documents(id) ON DELETE CASCADE;

ALTER TABLE ONLY public.documents
    ADD CONSTRAINT fk_documents_sender FOREIGN KEY (sender_id, user_id) REFERENCES public.email_senders(id, user_id) ON DELETE SET NULL (sender_id);

ALTER TABLE ONLY public.documents
    ADD CONSTRAINT fk_documents_user FOREIGN KEY (user_id) REFERENCES public.users(id) ON DELETE CASCADE;

ALTER TABLE ONLY public.domain_events
    ADD CONSTRAINT fk_domain_events_user FOREIGN KEY (user_id) REFERENCES public.users(id) ON DELETE CASCADE;

ALTER TABLE ONLY public.email_aliases
    ADD CONSTRAINT fk_email_aliases_user FOREIGN KEY (user_id) REFERENCES public.users(id) ON DELETE CASCADE;

ALTER TABLE ONLY public.email_senders
    ADD CONSTRAINT fk_email_senders_user FOREIGN KEY (user_id) REFERENCES public.users(id) ON DELETE CASCADE;

ALTER TABLE ONLY public.email_unsubscribe_targets
    ADD CONSTRAINT fk_email_unsubscribe_targets_sender FOREIGN KEY (sender_id) REFERENCES public.email_senders(id) ON DELETE CASCADE;

ALTER TABLE ONLY public.email_verification_tokens
    ADD CONSTRAINT fk_email_verification_user FOREIGN KEY (user_id) REFERENCES public.users(id) ON DELETE CASCADE;

ALTER TABLE ONLY public.entities
    ADD CONSTRAINT fk_entities_user FOREIGN KEY (user_id) REFERENCES public.users(id) ON DELETE CASCADE;

ALTER TABLE ONLY public.entitlement_snapshots
    ADD CONSTRAINT fk_entitlement_snapshots_plan FOREIGN KEY (effective_plan_id) REFERENCES public.plans(id);

ALTER TABLE ONLY public.entitlement_snapshots
    ADD CONSTRAINT fk_entitlement_snapshots_user FOREIGN KEY (user_id) REFERENCES public.users(id) ON DELETE CASCADE;

ALTER TABLE ONLY public.entity_mentions
    ADD CONSTRAINT fk_entity_mentions_document FOREIGN KEY (document_id) REFERENCES public.documents(id) ON DELETE CASCADE;

ALTER TABLE ONLY public.entity_mentions
    ADD CONSTRAINT fk_entity_mentions_entity FOREIGN KEY (entity_id) REFERENCES public.entities(id) ON DELETE CASCADE;

ALTER TABLE ONLY public.feed_deliveries
    ADD CONSTRAINT fk_feed_deliveries_document FOREIGN KEY (document_id, user_id) REFERENCES public.documents(id, user_id) ON DELETE SET NULL (document_id);

ALTER TABLE ONLY public.feed_deliveries
    ADD CONSTRAINT fk_feed_deliveries_source FOREIGN KEY (source_id) REFERENCES public.feed_sources(id) ON DELETE CASCADE;

ALTER TABLE ONLY public.feed_deliveries
    ADD CONSTRAINT fk_feed_deliveries_source_entry FOREIGN KEY (source_entry_id) REFERENCES public.feed_source_entries(id) ON DELETE CASCADE;

ALTER TABLE ONLY public.feed_deliveries
    ADD CONSTRAINT fk_feed_deliveries_subscription FOREIGN KEY (subscription_id) REFERENCES public.feed_subscriptions(id) ON DELETE CASCADE;

ALTER TABLE ONLY public.feed_deliveries
    ADD CONSTRAINT fk_feed_deliveries_user FOREIGN KEY (user_id) REFERENCES public.users(id) ON DELETE CASCADE;

ALTER TABLE ONLY public.feed_source_entries
    ADD CONSTRAINT fk_feed_source_entries_source FOREIGN KEY (source_id) REFERENCES public.feed_sources(id) ON DELETE CASCADE;

ALTER TABLE ONLY public.feed_subscriptions
    ADD CONSTRAINT fk_feed_subscriptions_collection FOREIGN KEY (auto_save_collection_id) REFERENCES public.collections(id) ON DELETE SET NULL;

ALTER TABLE ONLY public.feed_subscriptions
    ADD CONSTRAINT fk_feed_subscriptions_source FOREIGN KEY (source_id) REFERENCES public.feed_sources(id) ON DELETE CASCADE;

ALTER TABLE ONLY public.feed_subscriptions
    ADD CONSTRAINT fk_feed_subscriptions_user FOREIGN KEY (user_id) REFERENCES public.users(id) ON DELETE CASCADE;

ALTER TABLE ONLY public.highlight_notes
    ADD CONSTRAINT fk_highlight_notes_highlight FOREIGN KEY (highlight_id) REFERENCES public.highlights(id) ON DELETE CASCADE;

ALTER TABLE ONLY public.highlight_tags
    ADD CONSTRAINT fk_highlight_tags_highlight FOREIGN KEY (highlight_id) REFERENCES public.highlights(id) ON DELETE CASCADE;

ALTER TABLE ONLY public.highlight_tags
    ADD CONSTRAINT fk_highlight_tags_tag FOREIGN KEY (tag_id) REFERENCES public.tags(id) ON DELETE CASCADE;

ALTER TABLE ONLY public.highlights
    ADD CONSTRAINT fk_highlights_document FOREIGN KEY (document_id, user_id) REFERENCES public.documents(id, user_id) ON DELETE CASCADE;

ALTER TABLE ONLY public.highlights
    ADD CONSTRAINT fk_highlights_user FOREIGN KEY (user_id) REFERENCES public.users(id) ON DELETE CASCADE;

ALTER TABLE ONLY public.import_job_items
    ADD CONSTRAINT fk_import_job_items_job FOREIGN KEY (import_job_id) REFERENCES public.import_jobs(id) ON DELETE CASCADE;

ALTER TABLE ONLY public.import_jobs
    ADD CONSTRAINT fk_import_jobs_user FOREIGN KEY (user_id) REFERENCES public.users(id) ON DELETE CASCADE;

ALTER TABLE ONLY public.integration_connections
    ADD CONSTRAINT fk_integration_connections_user FOREIGN KEY (user_id) REFERENCES public.users(id) ON DELETE CASCADE;

ALTER TABLE ONLY public.integration_export_cursor
    ADD CONSTRAINT fk_integration_export_cursor_connection FOREIGN KEY (connection_id) REFERENCES public.integration_connections(id) ON DELETE CASCADE;

ALTER TABLE ONLY public.integration_export_cursor
    ADD CONSTRAINT fk_integration_export_cursor_library_entry FOREIGN KEY (library_entry_id) REFERENCES public.library_entries(id) ON DELETE CASCADE;

ALTER TABLE ONLY public.integration_oauth_tokens
    ADD CONSTRAINT fk_integration_oauth_tokens_user FOREIGN KEY (user_id) REFERENCES public.users(id) ON DELETE CASCADE;

ALTER TABLE ONLY public.item_notes
    ADD CONSTRAINT fk_item_notes_document FOREIGN KEY (document_id, user_id) REFERENCES public.documents(id, user_id) ON DELETE CASCADE;

ALTER TABLE ONLY public.item_notes
    ADD CONSTRAINT fk_item_notes_user FOREIGN KEY (user_id) REFERENCES public.users(id) ON DELETE CASCADE;

ALTER TABLE ONLY public.library_entries
    ADD CONSTRAINT fk_library_entries_delivery FOREIGN KEY (source_delivery_id) REFERENCES public.feed_deliveries(id) ON DELETE SET NULL;

ALTER TABLE ONLY public.library_entries
    ADD CONSTRAINT fk_library_entries_document FOREIGN KEY (document_id, user_id) REFERENCES public.documents(id, user_id) ON DELETE CASCADE;

ALTER TABLE ONLY public.library_entries
    ADD CONSTRAINT fk_library_entries_user FOREIGN KEY (user_id) REFERENCES public.users(id) ON DELETE CASCADE;

ALTER TABLE ONLY public.library_entry_tags
    ADD CONSTRAINT fk_library_entry_tags_library_entry FOREIGN KEY (library_entry_id, user_id) REFERENCES public.library_entries(id, user_id) ON DELETE CASCADE;

ALTER TABLE ONLY public.library_entry_tags
    ADD CONSTRAINT fk_library_entry_tags_tag FOREIGN KEY (tag_id, user_id) REFERENCES public.tags(id, user_id) ON DELETE CASCADE;

ALTER TABLE ONLY public.lifecycle_actions
    ADD CONSTRAINT fk_lifecycle_actions_user FOREIGN KEY (user_id) REFERENCES public.users(id) ON DELETE CASCADE;

ALTER TABLE ONLY public.mila_config
    ADD CONSTRAINT fk_mila_config_user FOREIGN KEY (user_id) REFERENCES public.users(id) ON DELETE CASCADE;

ALTER TABLE ONLY public.mila_messages
    ADD CONSTRAINT fk_mila_messages_session FOREIGN KEY (session_id) REFERENCES public.mila_sessions(id) ON DELETE CASCADE;

ALTER TABLE ONLY public.mila_sessions
    ADD CONSTRAINT fk_mila_sessions_collection FOREIGN KEY (collection_id) REFERENCES public.collections(id) ON DELETE CASCADE;

ALTER TABLE ONLY public.mila_sessions
    ADD CONSTRAINT fk_mila_sessions_document FOREIGN KEY (document_id) REFERENCES public.documents(id) ON DELETE SET NULL;

ALTER TABLE ONLY public.mila_sessions
    ADD CONSTRAINT fk_mila_sessions_user FOREIGN KEY (user_id) REFERENCES public.users(id) ON DELETE CASCADE;

ALTER TABLE ONLY public.notification_preferences
    ADD CONSTRAINT fk_notification_preferences_user FOREIGN KEY (user_id) REFERENCES public.users(id) ON DELETE CASCADE;

ALTER TABLE ONLY public.notifications
    ADD CONSTRAINT fk_notifications_user FOREIGN KEY (user_id) REFERENCES public.users(id) ON DELETE CASCADE;

ALTER TABLE ONLY public.notion_export_item_selection
    ADD CONSTRAINT fk_notion_export_item_selection_connection FOREIGN KEY (connection_id) REFERENCES public.integration_connections(id) ON DELETE CASCADE;

ALTER TABLE ONLY public.notion_export_item_selection
    ADD CONSTRAINT fk_notion_export_item_selection_library_entry FOREIGN KEY (library_entry_id) REFERENCES public.library_entries(id) ON DELETE CASCADE;

ALTER TABLE ONLY public.oauth_identities
    ADD CONSTRAINT fk_oauth_user FOREIGN KEY (user_id) REFERENCES public.users(id) ON DELETE CASCADE;

ALTER TABLE ONLY public.password_reset_tokens
    ADD CONSTRAINT fk_password_reset_user FOREIGN KEY (user_id) REFERENCES public.users(id) ON DELETE CASCADE;

ALTER TABLE ONLY public.push_tokens
    ADD CONSTRAINT fk_push_tokens_user FOREIGN KEY (user_id) REFERENCES public.users(id) ON DELETE CASCADE;

ALTER TABLE ONLY public.recent_searches
    ADD CONSTRAINT fk_recent_searches_user FOREIGN KEY (user_id) REFERENCES public.users(id) ON DELETE CASCADE;

ALTER TABLE ONLY public.referral_credits
    ADD CONSTRAINT fk_referral_credits_user FOREIGN KEY (user_id) REFERENCES public.users(id) ON DELETE CASCADE;

ALTER TABLE ONLY public.review_cards
    ADD CONSTRAINT fk_review_cards_highlight FOREIGN KEY (highlight_id) REFERENCES public.highlights(id) ON DELETE CASCADE;

ALTER TABLE ONLY public.review_cards
    ADD CONSTRAINT fk_review_cards_user FOREIGN KEY (user_id) REFERENCES public.users(id) ON DELETE CASCADE;

ALTER TABLE ONLY public.review_events
    ADD CONSTRAINT fk_review_events_card FOREIGN KEY (card_id) REFERENCES public.review_cards(id) ON DELETE CASCADE;

ALTER TABLE ONLY public.search_documents
    ADD CONSTRAINT fk_search_documents_document FOREIGN KEY (document_id, user_id) REFERENCES public.documents(id, user_id) ON DELETE CASCADE;

ALTER TABLE ONLY public.search_documents
    ADD CONSTRAINT fk_search_documents_user FOREIGN KEY (user_id) REFERENCES public.users(id) ON DELETE CASCADE;

ALTER TABLE ONLY public.smart_lists
    ADD CONSTRAINT fk_smart_lists_user FOREIGN KEY (user_id) REFERENCES public.users(id) ON DELETE CASCADE;

ALTER TABLE ONLY public.storage_add_ons
    ADD CONSTRAINT fk_storage_add_ons_user FOREIGN KEY (user_id) REFERENCES public.users(id) ON DELETE CASCADE;

ALTER TABLE ONLY public.subscriptions
    ADD CONSTRAINT fk_subscriptions_account FOREIGN KEY (billing_account_id) REFERENCES public.billing_accounts(id) ON DELETE CASCADE;

ALTER TABLE ONLY public.subscriptions
    ADD CONSTRAINT fk_subscriptions_plan FOREIGN KEY (plan_id) REFERENCES public.plans(id);

ALTER TABLE ONLY public.tag_aliases
    ADD CONSTRAINT fk_tag_aliases_tag FOREIGN KEY (tag_id) REFERENCES public.tags(id) ON DELETE CASCADE;

ALTER TABLE ONLY public.tags
    ADD CONSTRAINT fk_tags_parent FOREIGN KEY (parent_id) REFERENCES public.tags(id);

ALTER TABLE ONLY public.tags
    ADD CONSTRAINT fk_tags_user FOREIGN KEY (user_id) REFERENCES public.users(id) ON DELETE CASCADE;

ALTER TABLE ONLY public.tts_chunks
    ADD CONSTRAINT fk_tts_chunks_document FOREIGN KEY (document_id, user_id) REFERENCES public.documents(id, user_id) ON DELETE CASCADE;

ALTER TABLE ONLY public.tts_sessions
    ADD CONSTRAINT fk_tts_sessions_document FOREIGN KEY (document_id, user_id) REFERENCES public.documents(id, user_id) ON DELETE CASCADE;

ALTER TABLE ONLY public.user_document_state
    ADD CONSTRAINT fk_uds_document FOREIGN KEY (document_id, user_id) REFERENCES public.documents(id, user_id) ON DELETE CASCADE;

ALTER TABLE ONLY public.user_document_state
    ADD CONSTRAINT fk_uds_user FOREIGN KEY (user_id) REFERENCES public.users(id) ON DELETE CASCADE;

ALTER TABLE ONLY public.usage_counters
    ADD CONSTRAINT fk_usage_counters_user FOREIGN KEY (user_id) REFERENCES public.users(id) ON DELETE CASCADE;

ALTER TABLE ONLY public.user_preferences
    ADD CONSTRAINT fk_user_preferences_user FOREIGN KEY (user_id) REFERENCES public.users(id) ON DELETE CASCADE;

ALTER TABLE ONLY public.webhook_deliveries
    ADD CONSTRAINT fk_webhook_deliveries_dispatch FOREIGN KEY (dispatch_id) REFERENCES public.webhook_dispatches(id) ON DELETE CASCADE;

ALTER TABLE ONLY public.webhook_deliveries
    ADD CONSTRAINT fk_webhook_deliveries_endpoint FOREIGN KEY (endpoint_id) REFERENCES public.webhook_endpoints(id) ON DELETE CASCADE;

ALTER TABLE ONLY public.webhook_deliveries
    ADD CONSTRAINT fk_webhook_deliveries_event FOREIGN KEY (domain_event_id) REFERENCES public.domain_events(id) ON DELETE CASCADE;

ALTER TABLE ONLY public.webhook_dispatches
    ADD CONSTRAINT fk_webhook_dispatches_endpoint FOREIGN KEY (endpoint_id) REFERENCES public.webhook_endpoints(id) ON DELETE CASCADE;

ALTER TABLE ONLY public.webhook_dispatches
    ADD CONSTRAINT fk_webhook_dispatches_event FOREIGN KEY (domain_event_id) REFERENCES public.domain_events(id) ON DELETE CASCADE;

ALTER TABLE ONLY public.webhook_endpoints
    ADD CONSTRAINT fk_webhook_endpoints_user FOREIGN KEY (user_id) REFERENCES public.users(id) ON DELETE CASCADE;

ALTER TABLE ONLY public.obsidian_export_artifact_items
    ADD CONSTRAINT obsidian_export_artifact_items_artifact_id_fkey FOREIGN KEY (artifact_id) REFERENCES public.obsidian_export_artifacts(id) ON DELETE CASCADE;

ALTER TABLE ONLY public.obsidian_export_artifact_items
    ADD CONSTRAINT obsidian_export_artifact_items_library_entry_id_fkey FOREIGN KEY (library_entry_id) REFERENCES public.library_entries(id) ON DELETE CASCADE;

ALTER TABLE ONLY public.obsidian_export_artifacts
    ADD CONSTRAINT obsidian_export_artifacts_connection_id_fkey FOREIGN KEY (connection_id) REFERENCES public.integration_connections(id) ON DELETE CASCADE;

ALTER TABLE ONLY public.obsidian_export_artifacts
    ADD CONSTRAINT obsidian_export_artifacts_run_id_fkey FOREIGN KEY (run_id) REFERENCES public.obsidian_export_runs(id) ON DELETE CASCADE;

ALTER TABLE ONLY public.obsidian_export_artifacts
    ADD CONSTRAINT obsidian_export_artifacts_user_id_fkey FOREIGN KEY (user_id) REFERENCES public.users(id) ON DELETE CASCADE;

ALTER TABLE ONLY public.obsidian_export_refresh_queue
    ADD CONSTRAINT obsidian_export_refresh_queue_connection_id_fkey FOREIGN KEY (connection_id) REFERENCES public.integration_connections(id) ON DELETE CASCADE;

ALTER TABLE ONLY public.obsidian_export_refresh_queue
    ADD CONSTRAINT obsidian_export_refresh_queue_library_entry_id_fkey FOREIGN KEY (library_entry_id) REFERENCES public.library_entries(id) ON DELETE CASCADE;

ALTER TABLE ONLY public.obsidian_export_runs
    ADD CONSTRAINT obsidian_export_runs_connection_id_fkey FOREIGN KEY (connection_id) REFERENCES public.integration_connections(id) ON DELETE CASCADE;

ALTER TABLE ONLY public.obsidian_export_runs
    ADD CONSTRAINT obsidian_export_runs_user_id_fkey FOREIGN KEY (user_id) REFERENCES public.users(id) ON DELETE CASCADE;

ALTER TABLE ONLY public.refresh_tokens
    ADD CONSTRAINT refresh_tokens_replaced_by_fkey FOREIGN KEY (replaced_by) REFERENCES public.refresh_tokens(id);

ALTER TABLE ONLY public.refresh_tokens
    ADD CONSTRAINT refresh_tokens_user_id_fkey FOREIGN KEY (user_id) REFERENCES public.users(id) ON DELETE CASCADE;

ALTER TABLE ONLY public.tts_audio_assets
    ADD CONSTRAINT tts_audio_assets_chunk_record_id_fkey FOREIGN KEY (chunk_record_id) REFERENCES public.tts_chunks(id) ON DELETE CASCADE;

ALTER TABLE ONLY public.tts_audio_assets
    ADD CONSTRAINT tts_audio_assets_user_id_fkey FOREIGN KEY (user_id) REFERENCES public.users(id) ON DELETE CASCADE;

ALTER TABLE ONLY public.tts_chunks
    ADD CONSTRAINT tts_chunks_user_id_fkey FOREIGN KEY (user_id) REFERENCES public.users(id) ON DELETE CASCADE;

ALTER TABLE ONLY public.tts_chunks
    ADD CONSTRAINT tts_chunks_voice_persona_id_fkey FOREIGN KEY (voice_persona_id) REFERENCES public.tts_voice_personas(id) ON DELETE SET NULL;

ALTER TABLE ONLY public.tts_element_timings
    ADD CONSTRAINT tts_element_timings_chunk_record_id_fkey FOREIGN KEY (chunk_record_id) REFERENCES public.tts_chunks(id) ON DELETE CASCADE;

ALTER TABLE ONLY public.tts_session_chunks
    ADD CONSTRAINT tts_session_chunks_chunk_record_id_fkey FOREIGN KEY (chunk_record_id) REFERENCES public.tts_chunks(id) ON DELETE CASCADE;

ALTER TABLE ONLY public.tts_session_chunks
    ADD CONSTRAINT tts_session_chunks_session_id_fkey FOREIGN KEY (session_id) REFERENCES public.tts_sessions(id) ON DELETE CASCADE;

ALTER TABLE ONLY public.tts_sessions
    ADD CONSTRAINT tts_sessions_user_id_fkey FOREIGN KEY (user_id) REFERENCES public.users(id) ON DELETE CASCADE;

ALTER TABLE ONLY public.tts_sessions
    ADD CONSTRAINT tts_sessions_voice_persona_id_fkey FOREIGN KEY (voice_persona_id) REFERENCES public.tts_voice_personas(id) ON DELETE SET NULL;

ALTER TABLE ONLY public.tts_voice_personas
    ADD CONSTRAINT tts_voice_personas_user_id_fkey FOREIGN KEY (user_id) REFERENCES public.users(id) ON DELETE CASCADE;
