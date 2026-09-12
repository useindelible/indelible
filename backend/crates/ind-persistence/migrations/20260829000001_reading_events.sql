CREATE TABLE reading_events (
    id uuid PRIMARY KEY,
    user_id uuid NOT NULL REFERENCES public.users(id) ON DELETE CASCADE,
    document_id uuid NOT NULL REFERENCES public.documents(id) ON DELETE CASCADE,
    origin text NOT NULL,
    origin_seq bigint NOT NULL CHECK (origin_seq >= 0),
    event_kind text NOT NULL CHECK (event_kind IN ('opened', 'progress', 'finished')),
    -- Why the event happened. Ordering is by time; this is intent, which time cannot express.
    cause text NOT NULL DEFAULT 'reader'
        CHECK (cause IN ('reader', 'manual', 'import', 'sync', 'repair')),
    -- Groups one continuous sitting. Null from a client that does not track sessions.
    session_id uuid,
    -- Which pass through the document. A higher attempt outranks a lower one, which is what
    -- separates a deliberate reread from a stale device replaying old progress.
    attempt smallint NOT NULL DEFAULT 1 CHECK (attempt >= 1),
    -- Hundredths of a percent: whole percent loses eight pages of an 800-page book.
    progress_basis_points integer CHECK (progress_basis_points BETWEEN 0 AND 10000),
    position jsonb,
    position_version smallint NOT NULL DEFAULT 1 CHECK (position_version >= 1),
    -- Which readable representation the position refers to. Captured so a per-format
    -- projection can be built from history later; nothing reads it yet.
    asset_kind text,
    -- Active reading time accrued since the previous event in this session, never the session
    -- total, so SUM(active_ms) is meaningful.
    active_ms integer CHECK (active_ms >= 0),
    recorded_at timestamptz NOT NULL,
    received_at timestamptz NOT NULL DEFAULT now(),
    effective_at timestamptz NOT NULL,
    CONSTRAINT reading_events_origin_seq_unique UNIQUE (user_id, document_id, origin, origin_seq)
);

CREATE INDEX reading_events_user_document_effective_idx
    ON reading_events (user_id, document_id, effective_at DESC, received_at DESC, id DESC);

-- Surface and token writers keep no device-side counter, so the server orders them.
CREATE SEQUENCE reading_events_surface_seq;

ALTER TABLE user_document_state
    ADD COLUMN current_attempt smallint NOT NULL DEFAULT 1 CHECK (current_attempt >= 1),
    ADD COLUMN position_origin text,
    ADD COLUMN position_origin_seq bigint,
    ADD COLUMN position_effective_at timestamptz,
    ADD COLUMN position_received_at timestamptz,
    ADD COLUMN position_event_id uuid;
