# OAuth Browser-Based Applications Draft-26 Audit

Date: 2026-07-11

## Conclusion and claim boundary

NazoAuth was audited against `draft-ietf-oauth-browser-based-apps-26`, which
was still in the RFC Editor publication queue without an RFC number on the
review date. This is a dated security audit, not a final RFC conformance or
certification claim.

The audit confirms two intentionally different architectures:

- NazoAuthWeb is a same-origin first-party application using server-managed
  sessions and CSRF-protected `/auth/*` APIs. It does not receive or persist
  OAuth access tokens, refresh tokens, ID Tokens, client secrets, private keys,
  OIDF private configuration, or PKCE verifiers.
- Third-party browser applications are public OAuth clients. They use
  Authorization Code with S256 PKCE and exact redirect URIs; they cannot become
  confidential by embedding a static secret in JavaScript.

This change tightens `/token` and `/revoke` CORS to POST-only and removes the
session-only `X-CSRF-Token` header from public OAuth CORS. `/userinfo` retains
GET/POST bearer or DPoP access. None of these public OAuth routes authorize
browser credentials.

Primary source:

- <https://datatracker.ietf.org/doc/draft-ietf-oauth-browser-based-apps/26/>

## Requirement and evidence matrix

| Draft-26 area | NazoAuth role | Current control | Evidence | Outcome |
| --- | --- | --- | --- | --- |
| BFF architecture | First-party web/session backend | NazoAuthWeb receives an opaque secure session, not OAuth tokens; unsafe session writes require CSRF. | `src/http/profile/session.rs`, `src/http/auth/csrf.rs`, session/CSRF tests, coordinated NazoAuthWeb persistence gate | Covered for the first-party application. |
| Authorization endpoint | Authorization server | `/authorize` is a navigation endpoint and has no CORS middleware. | `authorization_endpoint_is_not_cors_enabled` | Covered. |
| Public browser client | Authorization server | Code-only flow, S256 PKCE for public clients, exact redirect binding, one-time authorization code. | `authorization_request_requires_pkce_for_public_client`, authorization PKCE and authorization-code tests | Covered. |
| Token endpoint CORS | Authorization server | Exact configured origins, POST only, no credentials, no CSRF header, explicit DPoP/content-type support. | `browser_token_management_cors_allows_post_dpop_without_credentials`, `production_token_route_rejects_get_csrf_and_unknown_origins` | Tightened in this change. |
| Revocation CORS | Authorization server | Shares the POST-only non-credentialed token-management policy. | `production_browser_oauth_routes_expose_only_required_cors` | Covered. |
| UserInfo CORS | Authorization server/resource endpoint | Exact origins, GET/POST, Authorization/DPoP, no browser credentials. | `browser_userinfo_cors_allows_get_and_post_bearer_or_dpop`, production-route test | Covered. |
| Redirect attacks | Authorization server | Redirect URI is registered and matched exactly at authorization and code exchange. | authorization request tests and `client_or_redirect_uri_mismatch` token test | Covered. |
| Authorization-code theft/replay | Authorization server | Code is short-lived, PKCE-bound, atomically consumed, and replay affects the associated refresh family. | authorization-code consumption/replay tests | Covered. |
| Refresh tokens | Authorization server | Issuance is client/policy gated; rotation, family binding, reuse detection, and fail-closed persistence are tested. | `refresh_grant_marks_family_reuse_and_revokes_active_family_tokens` and refresh failure tests | Covered. |
| Browser token storage | First-party app / third-party responsibility | NazoAuthWeb stores no OAuth credentials; arbitrary third-party SPA storage cannot be enforced by the AS. | Coordinated NazoAuthWeb `check-browser-security.mjs` gate | Bounded claim; third-party storage remains application responsibility. |
| Malicious JavaScript | BFF/public SPA | First-party tokens remain server-side; session/CSRF and response headers reduce impact. The AS cannot make arbitrary third-party JavaScript trustworthy. | session, CSRF, CORS, response-header, dependency/build gates | Covered for NazoAuth-controlled surfaces with stated residual risk. |
| Final RFC delta | Governance | The reviewed document has no RFC number. | IETF Datatracker status on 2026-07-11 | Re-audit required immediately after publication. |

## Threat review

### Malicious JavaScript and single token theft

The first-party application uses the BFF/session pattern, so its JavaScript has
no OAuth bearer token to exfiltrate. JavaScript can still act through the
current browser session; CSP, dependency integrity, output escaping, CSRF, and
short session lifetime therefore remain material controls. A compromised
third-party SPA can access any token available to that SPA; NazoAuth does not
claim to eliminate that application-local risk.

### Persistent token theft

NazoAuthWeb permits durable browser storage only for locale and a boolean
session hint. The hint is non-authoritative: the backend always checks the real
session. A coordinated source/build gate rejects new durable OAuth credential
storage. Third-party clients remain responsible for selecting a BFF,
token-mediating backend, or browser-only architecture appropriate to their risk.

### New-flow token acquisition and client hijacking

Authorization requests use registered exact redirects, one-time `state` at the
client, S256 PKCE, short-lived codes, and atomic code consumption. NazoAuth
does not accept an embedded browser secret as proof of confidentiality. OIDC
clients must use nonce to bind an ID Token to their authorization request.

### CSRF, CORS, and session confusion

The authorization endpoint is navigation-only and intentionally has no CORS.
Public OAuth protocol APIs use exact-origin, non-credentialed CORS and do not
accept the first-party CSRF header. Credentialed `/auth/me/*` operations are a
separate server-session surface and require the configured origin and CSRF on
unsafe requests. CORS does not replace OAuth client, token, redirect, issuer,
or audience validation.

### Refresh-token compromise

Refresh tokens are issued only under current client/scope policy. Rotation and
family reuse detection are server-side and fail closed when the reuse marker
cannot be persisted. Sender constraints remain available where selected by the
client/profile. This audit does not promise that arbitrary browser-only clients
can safely retain long-lived bearer refresh tokens.

## Architecture choices

| Pattern | NazoAuth position |
| --- | --- |
| BFF / same-origin session | Required architecture for NazoAuthWeb; recommended for first-party applications handling sensitive sessions. |
| Token-mediating backend | Compatible third-party architecture, but not implemented by NazoAuthWeb and not advertised as a distinct AS profile. |
| Browser-only public client | Supported at the AS boundary through code + S256 PKCE and minimal non-credentialed CORS; token storage and application compromise remain the client's responsibility. |

## Local verification

The focused audit commands are:

```powershell
cargo test --locked cors --lib
cargo test --locked authorization_pkce --lib
cargo test --locked redirect_uri --lib
cargo test --locked refresh --lib
cargo test --locked session --lib
cargo test --locked csrf --lib
cargo test --locked well_known --lib
```

The coordinated NazoAuthWeb change adds source and built-artifact checks to its
normal `npm test` gate. OIDC/FAPI local and official 19+2 matrices remain
regression evidence; the inspected OIDF suite has no dedicated Browser-Based
Applications OP plan.

## Publication re-entry trigger

After the RFC Editor assigns an RFC number:

1. compare the published RFC to draft-26 requirement by requirement;
2. update this matrix for normative or architectural differences;
3. implement and negatively test every concrete new server or first-party Web
   requirement;
4. re-check the official conformance suite for applicable plans; and
5. update public claims only after the delta audit and regression evidence pass.
