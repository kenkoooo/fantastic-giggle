CREATE TABLE relationship (
    source_id BIGINT NOT NULL,
    target_id BIGINT NOT NULL,
    following BOOLEAN NOT NULL DEFAULT FALSE,
    is_followed_by BOOLEAN NOT NULL DEFAULT FALSE,
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (source_id, target_id)
);
CREATE TABLE access_token (
    id BIGINT NOT NULL PRIMARY KEY,
    access_key TEXT NOT NULL,
    access_secret TEXT NOT NULL
);