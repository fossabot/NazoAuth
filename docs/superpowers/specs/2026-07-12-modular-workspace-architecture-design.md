# NazoAuth Modular Workspace Architecture Design

**Date:** 2026-07-12  
**Status:** Approved for implementation planning  
**Branch:** `codex/modular-workspace-architecture`

## 1. Objective

Rebuild NazoAuth as a Cargo Workspace modular monolith that can extend OAuth,
OpenID Connect, FAPI, CIBA, and future standards without coupling protocol
policy to Actix, Diesel, Fred, database rows, or deployment configuration.

The change is one architectural cutover in one branch and one pull request.
There will be no long-lived compatibility facade, duplicate old/new runtime,
or staged production release. Internally, the work will use logically scoped
commits and verification gates so failures remain attributable.

The project has not been formally released. Existing Rust crate and module APIs
may therefore change. Runtime and data compatibility remain mandatory.

## 2. Verified Baseline

The design is based on commit `413e18f` on `main`.

- The repository is already a Cargo Workspace, but the root package still owns
  nearly all runtime, protocol, identity, persistence, and HTTP dependencies.
- The only separate protocol-related package is
  `nazo-fapi-http-signatures`, which is organized around one feature rather
  than a stable domain boundary.
- `support::prelude` re-exports Actix, Diesel, Fred, database rows, settings,
  schema modules, cryptography, serialization, and helper functions.
- `AppState` exposes the complete PostgreSQL pool, Fred client, full settings
  object, and key store to handlers.
- `Settings` contains roughly sixty fields spanning unrelated capabilities.
- Protocol decisions, persistence operations, and HTTP response construction
  coexist in large endpoint modules; several production modules exceed one
  thousand lines.
- The existing resource-server API contains Actix, Tower, and Tonic adapters
  in the same module as framework-independent verification.
- CI path filters omit `crates/**` in several workflows, allowing workspace
  package changes to bypass intended checks.
- CI and container builds pin Rust 1.96 while the verified current stable
  toolchain is Rust 1.97.0.
- `cargo audit` reports no known vulnerability. `cargo deny` passes but emits
  duplicate-version warnings.
- On Rust stable 1.97.0, `cargo fmt --check`, workspace check, Clippy with
  warnings denied, and 1,977 tests pass on the baseline. Windows produces an
  environment-specific MSVC linker-message warning that is not a Rust source
  warning.

## 3. Invariants

The refactor must not unintentionally change any of the following:

- HTTP methods, routes, endpoint URLs, or route enablement conditions;
- configuration keys, environment variables, defaults, precedence, validation,
  or rejection of unknown configuration;
- PostgreSQL schema, migration ordering, migration history, stored values, or
  data compatibility;
- Valkey key strings, serialized values, expiry behavior, Lua atomicity, or
  fail-closed behavior;
- access-token, ID-token, logout-token, JARM, introspection, and other claims;
- OAuth/OIDC error codes, HTTP status codes, headers, JSON bodies, redirects,
  or browser responses;
- discovery, authorization-server, protected-resource, and JWKS metadata;
- OIDC, FAPI, CIBA, device, DCR, SCIM, identity, session, and administrative
  behavior.

Tests must capture these contracts before their implementation moves. A public
behavior change is permitted only when the same change includes compatibility
handling where applicable, migrations, documentation, and tests.

## 4. Target Workspace

```text
Cargo.toml
crates/
  auth/
  identity/
  resource-server/
  postgres/
  valkey/
  http-actix/
  server/
migrations/
tests/
```

The root manifest becomes a virtual workspace using resolver version 3. Shared
package metadata, dependency versions, lint policy, and release profiles live
at workspace scope. `crates/server` is the default member so existing operator
commands remain straightforward. The primary binary remains
`nazo-oauth-server`; migration, key-control, and OIDF seed tools remain separate
binaries in the server package.

No crate may exist solely to make the directory tree look symmetrical. A crate
must own a domain, dependency boundary, or security/failure boundary and must
contain production behavior and tests when introduced.

## 5. Crate Responsibilities

### 5.1 `nazo-auth`

Owns protocol-domain and application policy for OAuth, OIDC, FAPI, CIBA, and
closely related extensions. It contains typed requests, outcomes, errors,
claims, client policy, grant processing, authorization details, sender
constraints, metadata construction, security profiles, and signing policy.

The current FAPI HTTP Message Signatures implementation moves into
`auth::http_signatures`. The old `nazo-fapi-http-signatures` package is removed.

This crate must not depend on Actix, Diesel, diesel-async, diesel-migrations,
Fred, PostgreSQL schema modules, or persistence row structs. It may depend on
framework-neutral data, serialization, URL, cryptographic, and time libraries.
It defines storage and external-service ports in terms of domain types.

### 5.2 `nazo-identity`

Owns users, tenants, organizations, sessions, login policy, email verification,
MFA, passkeys, external identity links, and federation-domain behavior. It
defines identity repository, session store, email delivery, and federation
ports without depending on Actix, Diesel, or Fred.

Identity types do not reuse database rows. Protocol code receives the minimum
principal and authentication-context values it needs instead of an identity
record or persistence model.

### 5.3 `nazo-resource-server`

Owns framework-independent access-token, audience, scope, confirmation claim,
DPoP, and sender-constraint verification. It consumes framework-neutral request
evidence and returns typed authorization results or errors.

Actix adapters move to `nazo-http-actix`. Unused Tower and Tonic convenience
adapters are deleted unless repository usage proves that they are required by
an executable or test. The generic `http` crate may be used only if it improves
interoperability without importing a runtime or web framework.

### 5.4 `nazo-postgres`

Owns Diesel schema declarations, database row projections, pool management,
migrations, SQL queries, repository implementations, and PostgreSQL transaction
boundaries. Row structs never leave this crate. Explicit conversions map rows
to auth or identity domain types.

The existing `migrations/` directory and every migration file remain in place.
The migration binary invokes this crate. Multi-write domain operations use one
database transaction where atomicity is required.

### 5.5 `nazo-valkey`

Owns Fred, Valkey connection management, key construction, serialized storage
records, Lua scripts, replay protection, sessions, short-lived protocol state,
rate-limit counters, and atomic compare/set/delete operations.

Existing key formats, payload formats, TTL calculations, and script semantics
are locked by contract tests. The crate implements auth and identity store ports
and converts infrastructure failures into typed availability or consistency
errors; it does not construct HTTP responses.

### 5.6 `nazo-http-actix`

Owns Actix routes, request extraction, form/query/header parsing, cookies, CORS,
proxy-derived request context, HTTP response presentation, and endpoint-specific
dependency bundles. It contains no Diesel query, Fred command, token-claim
construction, protocol policy, or identity persistence logic.

Instead of one `AppState`, endpoints receive focused immutable services such as
`AuthorizationEndpoint`, `TokenEndpoint`, `MetadataEndpoint`,
`IdentityEndpoints`, `AdminEndpoints`, and `ResourceEndpoint`. Each service
exposes only the ports and configuration required by that capability.

### 5.7 `nazo-server`

Is the composition root. It owns configuration loading, grouped runtime
settings, observability, process lifecycle, dependency construction, background
tasks, and binaries. It is the only package allowed to depend on every concrete
adapter package.

The current configuration namespace remains unchanged. `ConfigSource` parses
the same keys and constructs focused immutable settings values for auth,
identity, HTTP, PostgreSQL, Valkey, key management, email, federation, rate
limits, and observability. No consumer receives the complete configuration.

## 6. Dependency Direction

Cargo dependency edges are exact and point from consumer to dependency:

```text
nazo-resource-server -> nazo-auth
nazo-postgres        -> nazo-auth, nazo-identity
nazo-valkey          -> nazo-auth, nazo-identity
nazo-http-actix      -> nazo-auth, nazo-identity, nazo-resource-server
nazo-server          -> nazo-auth, nazo-identity, nazo-resource-server,
                        nazo-postgres, nazo-valkey, nazo-http-actix
```

`nazo-auth` and `nazo-identity` do not depend on each other. They exchange only
minimal values at the composition/application boundary so identity storage
models cannot become protocol models. Domain crates do not depend on
infrastructure or HTTP crates. Adapters depend on the domain ports they
implement. HTTP depends on domain/application services, not concrete storage
clients. Server constructs concrete adapters and injects them into services.
Circular Cargo dependencies, cross-crate glob re-exports, and workspace-wide
preludes are forbidden.

## 7. Stable Extension Points

`nazo-auth` provides immutable registries built during startup:

- `GrantHandler`: validates and executes one token grant without parsing Actix
  requests or issuing an HTTP response;
- `ClientAuthenticator`: evaluates normalized credentials and produces an
  authenticated client context;
- `AuthorizationDetailsHandler`: validates, normalizes, canonicalizes, and
  evaluates one authorization-details type;
- `SenderConstraint`: validates proof material and produces confirmation claims
  and token-binding requirements;
- `MetadataContributor`: contributes typed metadata from enabled capabilities;
- `SecurityProfile`: selects allowed grants, client authentication, sender
  constraints, PAR/JAR/JARM requirements, algorithms, TTL bounds, and metadata.

Registry construction rejects duplicate identifiers. Metadata construction
rejects conflicting contributions instead of using last-write-wins behavior.
Registries are frozen before the HTTP server starts. Built-in implementations
use the same interfaces as future extensions, preventing extension-specific
branches from accumulating in token and authorization handlers.

Asynchronous ports return explicit futures or use stable async trait syntax
where object safety is unnecessary. Dynamic dispatch is introduced only for
registries that require runtime selection; static services otherwise use
generics or concrete aggregates to preserve clarity and performance.

## 8. Request and Error Flow

The canonical call chain is:

```text
Actix route
  -> strict transport parsing
  -> normalized protocol/identity request
  -> application service and security profile
  -> repository/store/external-service ports
  -> typed outcome or typed domain/infrastructure error
  -> Actix presenter
  -> existing HTTP response contract
```

Transport errors, OAuth protocol errors, identity policy errors, storage
availability errors, consistency conflicts, and internal defects remain
distinct. The Actix presenter owns exact response mapping. Domain and adapter
crates never return `HttpResponse`.

Errors retain sources internally for logs and tracing while public responses
remain non-sensitive. Authentication, replay, sender-constraint, and backend
failures preserve existing fail-closed behavior.

## 9. Transactions, Concurrency, and Failure Semantics

PostgreSQL atomic operations remain PostgreSQL transactions. Valkey atomic
operations remain single commands or reviewed Lua scripts. The architecture
does not pretend these systems share a distributed transaction.

Cross-system workflows must make their ordering and compensation explicit.
Existing authorization-code, refresh-token, session-rotation, replay, logout
outbox, and issuance invariants receive targeted concurrency and failure tests
before being moved. A newly found partial-success window must be fixed in the
same change with a reproducing test rather than documented as debt.

Shared registries and configuration are immutable after startup. Mutable key
state uses a narrowly scoped concurrency-safe store. Blocking password hashing,
filesystem access, and external signing remain outside async executor threads
or use bounded blocking execution.

## 10. Compatibility Verification

Before moving implementations, tests will record:

- the complete method-and-route inventory and conditional route registration;
- the canonical configuration key set, defaults, parsing, and invalid-input
  behavior;
- migration filenames, ordering, and schema results on empty and upgraded
  databases;
- every Valkey key builder and representative serialized state;
- token and authorization response claim fixtures;
- OAuth/OIDC error status, code, header, body, and redirect fixtures;
- discovery, authorization-server, protected-resource, and JWKS metadata;
- CIBA, device, PAR, DPoP, mTLS, introspection, userinfo, logout, DCR, and SCIM
  behavior exercised by existing integration and conformance tooling.

Tests compare externally observable values, not internal module paths. Existing
test coverage is moved to the owning crate rather than discarded or weakened.

## 11. One-Cutover Implementation Strategy

All work lands in one branch and one PR. There is no intermediate production
deployment and no final old/new compatibility layer.

Internal commit sequence:

1. add compatibility-contract tests and workspace lint/toolchain policy;
2. create non-empty auth and resource-server boundaries and move pure behavior;
3. add extension registries and migrate authorization/token/metadata policy;
4. extract identity services and domain types;
5. extract PostgreSQL rows, repositories, migrations, and transactions;
6. extract Valkey keys, scripts, state, and failure mapping;
7. rebuild Actix handlers as transport adapters with focused endpoint services;
8. move configuration/bootstrap/binaries to the server composition root;
9. delete old root modules, preludes, glob re-exports, duplicate helpers, unused
   adapters, dead code, and obsolete tests;
10. update dependencies, CI, containers, deployment files, and documentation;
11. run all local verification and create the Draft PR;
12. deploy the completed architecture and execute both OIDF matrices and PR
    checks before marking the PR ready for review.

Each commit must compile or be paired with the immediately following migration
commit when a Cargo graph cutover cannot be represented independently. No
commit is pushed until the complete local verification gate passes.

## 12. Dependency and Toolchain Policy

Rust is pinned through `rust-toolchain.toml` to the current stable release used
by CI and containers. Workspace dependencies use one canonical version and
feature declaration. Direct dependencies are scoped to the crates that use
them; the server package must not become another dependency dump.

Compatible stable dependency upgrades are applied with their changelogs and
official documentation checked for behavioral changes. `Cargo.lock` remains
committed. Dependabot includes Cargo, GitHub Actions, containers, and Python
locks. CI runs `cargo audit`, `cargo deny`, dependency review, CodeQL, SBOM,
container vulnerability scanning, and all workspace quality gates. Workflow
path filters include `crates/**` and workspace configuration.

Duplicate dependency versions are reduced where upstream compatibility permits;
unavoidable duplicates are understood and explicitly governed rather than
silently ignored.

## 13. Required Verification and Release Flow

Local completion requires fresh successful runs of:

```text
cargo fmt --check
cargo check --workspace --all-targets --all-features --locked
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo test --workspace --all-features --locked
```

The repository's executable unit, integration, real HTTP E2E, database
migration, security, concurrency/load, fault-injection, container-build, and
local-consistency suites also run. Failures are root-caused, fixed, and rerun.

After local verification:

1. push the branch and open a Draft PR;
2. inspect the current `hostinger` deployment and retain a verified rollback
   image/version;
3. run required migrations and deploy the completed service;
4. verify process state, logs, PostgreSQL, Valkey, TLS, health, discovery, JWKS,
   authorization, token, PAR, CIBA, userinfo, and introspection;
5. on deployment failure, repair or roll back before any conformance run;
6. run the host-local complete OIDF matrix and resolve every unexpected
   failure, warning, condition failure, or skip;
7. run the official complete OIDF matrix against
   `https://auth.nazo.run` and apply the same acceptance rules;
8. monitor and repair every PR check;
9. update the Draft PR description with evidence from actual runs;
10. mark Ready for Review only when every required gate passes; do not merge.

## 14. Acceptance Criteria

The architecture is complete only when:

- all target crates have their declared responsibility and enforced dependency
  direction;
- auth core has no Actix, Diesel, Fred, or database-row dependency;
- the old root monolith, large prelude, glob re-exports, giant state/settings,
  and miscellaneous support layer are gone;
- extension registries are active in real protocol paths, not decorative traits;
- obsolete and duplicate code is deleted;
- runtime and data compatibility tests pass;
- all local quality, integration, security, failure, migration, and container
  gates pass;
- the public deployment is stable and its critical endpoints pass;
- host-local and official OIDF full matrices meet the repository's established
  acceptance standard;
- all PR checks pass and the PR description matches observed evidence.
