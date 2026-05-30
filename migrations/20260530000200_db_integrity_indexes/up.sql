DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1
        FROM pg_constraint
        WHERE conname = 'ck_oauth_clients_token_endpoint_auth_method_value'
    ) THEN
        ALTER TABLE oauth_clients
            ADD CONSTRAINT ck_oauth_clients_token_endpoint_auth_method_value
            CHECK (token_endpoint_auth_method IN ('none', 'client_secret_basic', 'client_secret_post'));
    END IF;
END $$;

CREATE UNIQUE INDEX IF NOT EXISTS ux_users_email_lower ON users (lower(email));
CREATE INDEX IF NOT EXISTS ix_users_created_at_desc ON users (created_at DESC);

CREATE INDEX IF NOT EXISTS ix_oauth_clients_created_at_desc ON oauth_clients (created_at DESC);

CREATE INDEX IF NOT EXISTS ix_oauth_tokens_family_active
    ON oauth_tokens (token_family_id)
    WHERE revoked_at IS NULL;
CREATE INDEX IF NOT EXISTS ix_oauth_tokens_user_client_active
    ON oauth_tokens (user_id, client_id)
    WHERE revoked_at IS NULL;
CREATE INDEX IF NOT EXISTS ix_oauth_tokens_expires_at ON oauth_tokens (expires_at);

CREATE INDEX IF NOT EXISTS ix_user_client_grants_user_last_authorized
    ON user_client_grants (user_id, last_authorized_at DESC);

CREATE INDEX IF NOT EXISTS ix_client_access_requests_user_created_at
    ON client_access_requests (user_id, created_at DESC);
CREATE INDEX IF NOT EXISTS ix_client_access_requests_status_created_at
    ON client_access_requests (status, created_at DESC);
CREATE INDEX IF NOT EXISTS ix_client_access_requests_created_at
    ON client_access_requests (created_at DESC);

COMMENT ON COLUMN oauth_clients.token_endpoint_auth_method IS
    'none=public client, client_secret_basic=HTTP Basic client authentication, client_secret_post=form body client authentication';
COMMENT ON COLUMN users.role IS 'user=normal account, admin=administrator';
