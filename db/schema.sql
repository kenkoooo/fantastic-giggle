CREATE TABLE "follower" (
    source_id BIGINT NOT NULL,
    target_id BIGINT NOT NULL,
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (source_id, target_id)
);
CREATE INDEX "follower_source_id" ON "follower" (source_id);
CREATE TABLE "friend" (
    source_id BIGINT NOT NULL,
    target_id BIGINT NOT NULL,
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (source_id, target_id)
);
CREATE INDEX "friend_source_id" ON "friend" (source_id);
CREATE TABLE "user" (
    id BIGINT NOT NULL PRIMARY KEY,
    access_key TEXT NOT NULL,
    access_secret TEXT NOT NULL
);
CREATE TABLE "whitelist" (
    source_id BIGINT NOT NULL,
    target_id BIGINT NOT NULL,
    PRIMARY KEY (source_id, target_id)
);
CREATE INDEX "whitelist_source_id" ON "whitelist" (source_id);