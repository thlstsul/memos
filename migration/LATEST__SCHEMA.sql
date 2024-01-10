PRAGMA foreign_keys=OFF;
BEGIN TRANSACTION;
CREATE TABLE IF NOT EXISTS migration_history (
version TEXT NOT NULL PRIMARY KEY,
created_ts BIGINT NOT NULL DEFAULT (strftime('%s', 'now'))
);
CREATE TABLE IF NOT EXISTS system_setting (
name TEXT NOT NULL,
value TEXT NOT NULL,
description TEXT NOT NULL DEFAULT '',
UNIQUE(name)
);
CREATE TABLE IF NOT EXISTS user (
id INTEGER PRIMARY KEY AUTOINCREMENT,
created_ts BIGINT NOT NULL DEFAULT (strftime('%s', 'now')),
updated_ts BIGINT NOT NULL DEFAULT (strftime('%s', 'now')),
row_status TEXT NOT NULL CHECK (row_status IN ('NORMAL', 'ARCHIVED')) DEFAULT 'NORMAL',
username TEXT NOT NULL UNIQUE,
role TEXT NOT NULL CHECK (role IN ('HOST', 'ADMIN', 'USER')) DEFAULT 'USER',
email TEXT NOT NULL DEFAULT '',
nickname TEXT NOT NULL DEFAULT '',
password_hash TEXT NOT NULL,
avatar_url TEXT NOT NULL DEFAULT ''
);
CREATE TABLE IF NOT EXISTS user_setting (
user_id INTEGER NOT NULL,
key TEXT NOT NULL,
value TEXT NOT NULL,
UNIQUE(user_id, key)
);
CREATE TABLE IF NOT EXISTS memo (
id INTEGER PRIMARY KEY AUTOINCREMENT,
creator_id INTEGER NOT NULL,
created_ts BIGINT NOT NULL DEFAULT (strftime('%s', 'now')),
updated_ts BIGINT NOT NULL DEFAULT (strftime('%s', 'now')),
row_status TEXT NOT NULL CHECK (row_status IN ('NORMAL', 'ARCHIVED')) DEFAULT 'NORMAL',
content TEXT NOT NULL DEFAULT '',
visibility TEXT NOT NULL CHECK (visibility IN ('PUBLIC', 'PROTECTED', 'PRIVATE')) DEFAULT 'PRIVATE'
);
CREATE TABLE IF NOT EXISTS memo_organizer (
memo_id INTEGER NOT NULL,
user_id INTEGER NOT NULL,
pinned INTEGER NOT NULL CHECK (pinned IN (0, 1)) DEFAULT 0,
UNIQUE(memo_id, user_id)
);
CREATE TABLE IF NOT EXISTS memo_relation (
memo_id INTEGER NOT NULL,
related_memo_id INTEGER NOT NULL,
type TEXT NOT NULL,
UNIQUE(memo_id, related_memo_id, type)
);
CREATE TABLE IF NOT EXISTS resource (
id INTEGER PRIMARY KEY AUTOINCREMENT,
creator_id INTEGER NOT NULL,
created_ts BIGINT NOT NULL DEFAULT (strftime('%s', 'now')),
updated_ts BIGINT NOT NULL DEFAULT (strftime('%s', 'now')),
filename TEXT NOT NULL DEFAULT '',
blob BLOB DEFAULT NULL,
external_link TEXT NOT NULL DEFAULT '',
type TEXT NOT NULL DEFAULT '',
size INTEGER NOT NULL DEFAULT 0,
internal_path TEXT NOT NULL DEFAULT '',
memo_id INTEGER
);
CREATE TABLE IF NOT EXISTS tag (
name TEXT NOT NULL,
creator_id INTEGER NOT NULL,
UNIQUE(name, creator_id)
);
CREATE TABLE IF NOT EXISTS activity (
id INTEGER PRIMARY KEY AUTOINCREMENT,
creator_id INTEGER NOT NULL,
created_ts BIGINT NOT NULL DEFAULT (strftime('%s', 'now')),
type TEXT NOT NULL DEFAULT '',
level TEXT NOT NULL CHECK (level IN ('INFO', 'WARN', 'ERROR')) DEFAULT 'INFO',
payload TEXT NOT NULL DEFAULT '{}'
);
CREATE TABLE IF NOT EXISTS storage (
id INTEGER PRIMARY KEY AUTOINCREMENT,
name TEXT NOT NULL,
type TEXT NOT NULL,
config TEXT NOT NULL DEFAULT '{}'
);
CREATE TABLE IF NOT EXISTS idp (
id INTEGER PRIMARY KEY AUTOINCREMENT,
name TEXT NOT NULL,
type TEXT NOT NULL,
identifier_filter TEXT NOT NULL DEFAULT '',
config TEXT NOT NULL DEFAULT '{}'
);
CREATE TABLE IF NOT EXISTS inbox (
id INTEGER PRIMARY KEY AUTOINCREMENT,
created_ts BIGINT NOT NULL DEFAULT (strftime('%s', 'now')),
sender_id INTEGER NOT NULL,
receiver_id INTEGER NOT NULL,
status TEXT NOT NULL,
message TEXT NOT NULL DEFAULT '{}'
);
DELETE FROM sqlite_sequence;
CREATE INDEX idx_user_username ON user (username);
CREATE INDEX idx_memo_creator_id ON memo (creator_id);
CREATE INDEX idx_memo_content ON memo (content);
CREATE INDEX idx_memo_visibility ON memo (visibility);
CREATE INDEX idx_resource_creator_id ON resource (creator_id);
CREATE INDEX idx_resource_memo_id ON resource (memo_id);
COMMIT;

create table if not exists sessions
(
    id text primary key not null,
    data blob not null,
    expiry_date integer not null
);