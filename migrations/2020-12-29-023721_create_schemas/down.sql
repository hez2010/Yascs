-- This file should undo anything in `up.sql`
DROP INDEX IF EXISTS "ix_users_id";
DROP INDEX IF EXISTS "ix_messages_id";
DROP INDEX IF EXISTS "ix_messages_quote_id";
DROP INDEX IF EXISTS "ix_messages_from_user";
DROP INDEX IF EXISTS "ix_messages_to_user";
DROP INDEX IF EXISTS "ix_friends_id";
DROP INDEX IF EXISTS "ix_friends_user_id";
DROP INDEX IF EXISTS "ix_friends_friend_user_id";
DROP TABLE IF EXISTS "messages";
DROP TABLE IF EXISTS "friends";
DROP TABLE IF EXISTS "users";
