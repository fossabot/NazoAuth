ALTER TABLE user_client_grants
    DROP CONSTRAINT IF EXISTS ck_user_client_grants_last_scopes_array;

ALTER TABLE oauth_tokens
    DROP CONSTRAINT IF EXISTS ck_oauth_tokens_scopes_array;

ALTER TABLE oauth_clients
    DROP CONSTRAINT IF EXISTS ck_oauth_clients_allowed_audiences_array,
    DROP CONSTRAINT IF EXISTS ck_oauth_clients_grant_types_array,
    DROP CONSTRAINT IF EXISTS ck_oauth_clients_scopes_array,
    DROP CONSTRAINT IF EXISTS ck_oauth_clients_redirect_uris_array;

ALTER TABLE user_client_grants
    ALTER COLUMN last_scopes TYPE JSON USING last_scopes::json;

ALTER TABLE oauth_tokens
    ALTER COLUMN scopes TYPE JSON USING scopes::json;

ALTER TABLE oauth_clients
    ALTER COLUMN allowed_audiences DROP DEFAULT,
    ALTER COLUMN allowed_audiences TYPE JSON USING allowed_audiences::json,
    ALTER COLUMN allowed_audiences SET DEFAULT '["resource://default"]'::json,
    ALTER COLUMN grant_types TYPE JSON USING grant_types::json,
    ALTER COLUMN scopes TYPE JSON USING scopes::json,
    ALTER COLUMN redirect_uris TYPE JSON USING redirect_uris::json;
