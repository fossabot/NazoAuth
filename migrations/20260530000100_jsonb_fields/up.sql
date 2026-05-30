ALTER TABLE oauth_clients
    ALTER COLUMN redirect_uris TYPE JSONB USING redirect_uris::jsonb,
    ALTER COLUMN scopes TYPE JSONB USING scopes::jsonb,
    ALTER COLUMN grant_types TYPE JSONB USING grant_types::jsonb,
    ALTER COLUMN allowed_audiences DROP DEFAULT,
    ALTER COLUMN allowed_audiences TYPE JSONB USING allowed_audiences::jsonb,
    ALTER COLUMN allowed_audiences SET DEFAULT '["resource://default"]'::jsonb;

ALTER TABLE oauth_tokens
    ALTER COLUMN scopes TYPE JSONB USING scopes::jsonb;

ALTER TABLE user_client_grants
    ALTER COLUMN last_scopes TYPE JSONB USING last_scopes::jsonb;

ALTER TABLE oauth_clients
    DROP CONSTRAINT IF EXISTS ck_oauth_clients_redirect_uris_array,
    DROP CONSTRAINT IF EXISTS ck_oauth_clients_scopes_array,
    DROP CONSTRAINT IF EXISTS ck_oauth_clients_grant_types_array,
    DROP CONSTRAINT IF EXISTS ck_oauth_clients_allowed_audiences_array;

ALTER TABLE oauth_tokens
    DROP CONSTRAINT IF EXISTS ck_oauth_tokens_scopes_array;

ALTER TABLE user_client_grants
    DROP CONSTRAINT IF EXISTS ck_user_client_grants_last_scopes_array;

ALTER TABLE oauth_clients
    ADD CONSTRAINT ck_oauth_clients_redirect_uris_array CHECK (jsonb_typeof(redirect_uris) = 'array'),
    ADD CONSTRAINT ck_oauth_clients_scopes_array CHECK (jsonb_typeof(scopes) = 'array'),
    ADD CONSTRAINT ck_oauth_clients_grant_types_array CHECK (jsonb_typeof(grant_types) = 'array'),
    ADD CONSTRAINT ck_oauth_clients_allowed_audiences_array CHECK (jsonb_typeof(allowed_audiences) = 'array');

ALTER TABLE oauth_tokens
    ADD CONSTRAINT ck_oauth_tokens_scopes_array CHECK (jsonb_typeof(scopes) = 'array');

ALTER TABLE user_client_grants
    ADD CONSTRAINT ck_user_client_grants_last_scopes_array CHECK (jsonb_typeof(last_scopes) = 'array');
