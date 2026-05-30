COMMENT ON COLUMN users.role IS NULL;
COMMENT ON COLUMN oauth_clients.token_endpoint_auth_method IS NULL;

DROP INDEX IF EXISTS ix_client_access_requests_created_at;
DROP INDEX IF EXISTS ix_client_access_requests_status_created_at;
DROP INDEX IF EXISTS ix_client_access_requests_user_created_at;

DROP INDEX IF EXISTS ix_user_client_grants_user_last_authorized;

DROP INDEX IF EXISTS ix_oauth_tokens_expires_at;
DROP INDEX IF EXISTS ix_oauth_tokens_user_client_active;
DROP INDEX IF EXISTS ix_oauth_tokens_family_active;

DROP INDEX IF EXISTS ix_oauth_clients_created_at_desc;

DROP INDEX IF EXISTS ix_users_created_at_desc;
DROP INDEX IF EXISTS ux_users_email_lower;

ALTER TABLE oauth_clients
    DROP CONSTRAINT IF EXISTS ck_oauth_clients_token_endpoint_auth_method_value;
