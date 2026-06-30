CREATE TABLE checkouts (
    id                  TEXT PRIMARY KEY,
    status              TEXT NOT NULL CHECK (status IN (
                            'incomplete',
                            'ready_for_complete',
                            'complete_in_progress',
                            'completed',
                            'requires_escalation',
                            'canceled'
                        )),
    line_items          JSONB NOT NULL,
    buyer               JSONB NOT NULL,
    total               BIGINT NOT NULL,
    currency            TEXT NOT NULL,
    messages            JSONB NOT NULL DEFAULT '[]'::jsonb,
    continue_url        TEXT,
    payment_handler_id  TEXT,
    created_at          TIMESTAMPTZ NOT NULL,
    updated_at          TIMESTAMPTZ NOT NULL
);

CREATE INDEX idx_checkouts_status ON checkouts (status);
