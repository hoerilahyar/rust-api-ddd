# rust-api-ddd

Clean Architecture / DDD backend in Rust: Axum + SQLx (PostgreSQL) + Redis,
JWT auth (access + refresh), Argon2 password hashing, and RBAC (roles ↔
permissions), with a full set of admin modules (users, roles, permissions,
menus, settings, files, master data, and audit/activity logging).

## Stack

- **Web**: Axum 0.7 (Tokio, Tower / tower-http: CORS, tracing, catch-panic, timeouts)
- **DB**: PostgreSQL via SQLx 0.9 (migrations embedded at compile time from
  `databases/postgresql/migrations`)
- **Cache**: Redis, via `redis::aio::ConnectionManager` (cache-aside for
  several read-heavy entities, fixed-window rate limiter, refresh-token /
  password-reset-token lookup)
- **Auth**: JWT access/refresh tokens signed with **separate secrets**
  (`jwt.access_secret` / `jwt.refresh_secret`), Argon2id password hashing,
  refresh-token rotation + revocation on logout/reset
- **Validation**: `validator` crate via a custom `ValidatedJson<T>` extractor
  (returns field-level errors as JSON, HTTP 422)
- **Errors**: a single `AppError` enum maps every layer's failures to a
  consistent `{ success, message, ... }` JSON envelope and the right status
  code (see `shared::errors::AppError`)

## Project layout

```
src/
  bootstrap/     # config loading, Postgres pool, Redis connection manager,
                 # migrations, AppState wiring, top-level router assembly
  shared/        # cross-cutting: errors, response envelope, cache, request
                 # context, pagination, middleware, cross-module contracts
  routes.rs      # merges every module's router under bootstrap::router
  modules/
    auth/            # login, refresh, logout, forgot/reset password
    user/            # user CRUD, role assignment, /me, change password
    user_profile/    # per-user profile, /me/profile + admin read
    user_setting/     # per-user key/value preferences, /me/settings
    role/            # role CRUD + permission assignment
    permission/      # permission catalog CRUD
    menu/            # navigation menu tree, permission-filtered /me/menu
    setting/         # system-wide key/value settings (admin only)
    file/            # multipart upload, download (streamed), local storage
    masters/         # generic "master group -> master items" lookup data
    activity_log/    # generic per-request activity trail (all modules)
    audit_auth_log/  # authentication-specific audit trail (login attempts)
    audit_trail_log/ # entity-change audit trail (old/new values per mutation)
```

Every module follows `domain -> application -> infrastructure -> presentation`.
`shared::contracts` (`UserReader`, `AuditAuthRecorder`, `AuditTrailRecorder`,
`ActivityRecorder`, ...) is what lets one module read/write another module's
data (e.g. `auth` reading user records, or any module writing an audit trail
entry) without depending on that module's persistence types directly.

Two logging modules exist side by side on purpose:
- **`activity_log`** — one row per HTTP request that passed
  `activity_log_middleware` (method, path, status, actor, description),
  regardless of which module handled it.
- **`audit_trail_log`** — one row per *mutation*, with the actual
  before/after JSON values, written explicitly by each service (`create`,
  `update`, `delete`) via `spawn_audit_log`.
- **`audit_auth_log`** — one row per login *attempt* (success or failure),
  written explicitly by `AuthServiceImpl`, independent of both of the above.

## Setup

1. **Postgres + Redis** (adjust to taste):
   ```bash
   docker run -d --name pg -e POSTGRES_PASSWORD=postgres -e POSTGRES_DB=rust_clean_ddd -p 5432:5432 postgres:16
   docker run -d --name redis -p 6379:6379 redis:7
   ```

2. **Config**: layered, later wins — `config/config.yaml` (committed
   defaults) → `config/config.local.yaml` (gitignored local overrides) →
   environment variables prefixed `APP__` with `__` as the section
   separator (see `.env.example`). At minimum, set real values for
   `database.url`, `redis.url`, `jwt.access_secret`, `jwt.refresh_secret`
   before running anywhere near production — the committed
   `config/config.yaml` ships with placeholder JWT secrets.
   ```bash
   cp .env.example .env
   # edit .env with real secrets
   ```

3. **Migrate**:
   ```bash
   cargo install sqlx-cli --no-default-features --features postgres
   sqlx migrate run --source databases/postgresql/migrations
   ```
   (or set `database.run_migrations_on_boot: true` in config to run them
   automatically on `cargo run`.)

4. **Seed** (idempotent, safe to re-run — every statement is
   `ON CONFLICT DO NOTHING`):
   ```bash
   for f in databases/postgresql/seeders/*.sql; do psql "$DATABASE_URL" -f "$f"; done
   ```
   ⚠️ `002_seed_admin_user.sql` inserts a **placeholder** password hash in
   bcrypt-shaped text. This app verifies passwords with **Argon2**, not
   bcrypt, so that seeded hash will never successfully log in. Generate a
   real Argon2 hash with the dev helper already wired up in `Cargo.toml`:
   ```bash
   cargo run --example hash_password -- "YourP@ssw0rd"
   ```
   then either edit the hash in the seeder before first running it, or
   update the row directly:
   ```sql
   UPDATE users SET password_hash = '<hash from the command above>'
   WHERE email = 'admin@mail.local';
   ```
   The seeded admin account is `admin@mail.local` / username `admin`, with
   the `admin` role, which is granted every permission that exists in the
   `permissions` table at seed time.

5. **Run**:
   ```bash
   cargo run
   ```
   Server listens on `server.host:server.port` from config (default
   `0.0.0.0:8080`). `GET /health` reports build metadata (git commit/branch,
   build time) baked in by `build.rs`.

## API

All routes are mounted under `/api/v1`, plus an unprefixed `GET /health`.
Every response is a JSON envelope: `{ "success": bool, "message": string,
"data"?: ..., "errors"?: [...] }` (paginated list endpoints also include
`page`, `limit`, `total`). Every `✔` route below additionally requires the
listed permission via `ensure_permission`, checked *inside* the handler —
`require_auth` alone is not sufficient for those.

| Method | Path | Auth | Notes |
|---|---|---|---|
| POST | `/auth/login` | – | `{ identifier, password }` → access + refresh token pair |
| POST | `/auth/refresh` | – | rotates the refresh token |
| POST | `/auth/logout` | – | revokes the given refresh token |
| POST | `/auth/forgot-password` | – | always 200, never reveals whether the email exists |
| POST | `/auth/reset-password` | – | `{ token, password }`; revokes all of the user's refresh tokens |
| GET | `/me` | ✔ | current user profile |
| PUT | `/me/password` | ✔ | change own password |
| GET | `/me/menu` | ✔ | nav tree filtered to the caller's own permissions |
| GET/PUT/DELETE | `/me/profile` | ✔ | own profile |
| GET/PUT/DELETE | `/me/settings`, `/me/settings/:key` | ✔ | own key/value preferences |
| GET | `/users` | ✔ `user.manage` | paginated, `?page&limit&search` |
| POST | `/users` | ✔ `user.manage` | create user |
| GET/PUT/DELETE | `/users/:id` | ✔ `user.manage` | read / update / soft-delete |
| GET | `/users/:id/profile` | ✔ `user.manage` | admin read of another user's profile |
| POST | `/users/:id/roles` | ✔ `user.manage` | `{ role: "admin" }` |
| DELETE | `/users/:id/roles/:role` | ✔ `user.manage` | revoke a role |
| GET/POST | `/roles`, `/roles/:id`, `PUT`/`DELETE` | ✔ `role.manage` | role CRUD |
| POST/DELETE | `/roles/:id/permission[/:permission]` | ✔ `role.manage` | assign/revoke a permission |
| GET/POST | `/permissions`, `/permissions/:id`, `PUT`/`DELETE` | ✔ `permission.manage` | permission catalog CRUD |
| GET | `/menus/tree` | ✔ `menu.manage` | full nav tree, unfiltered |
| GET/POST | `/menus`, `/menus/:id`, `PUT`/`DELETE` | ✔ `menu.manage` | menu CRUD (cycle-checked reparenting) |
| POST/DELETE | `/menus/:id/permission[/:permission]` | ✔ `menu.manage` | gate a menu behind a permission |
| GET | `/settings`, `/settings/:key` | ✔ `settings.manage` | system settings |
| PUT/DELETE | `/settings/:key` | ✔ `settings.manage` | upsert / delete a setting |
| GET | `/files` | ✔ `file.read` | paginated |
| POST | `/files` | ✔ `file.upload` | multipart, capped by `storage.max_upload_bytes` |
| GET | `/files/:uuid` | ✔ `file.read` | metadata |
| GET | `/files/:uuid/download` | ✔ `file.read` | streamed from disk |
| DELETE | `/files/:uuid` | ✔ `file.delete` | soft-delete metadata, best-effort disk cleanup |
| GET/POST | `/masters`, `/masters/:id`, `PUT`/`DELETE` | ✔ `masters.manage` | master groups |
| POST | `/masters/:id/items` | ✔ `masters.manage` | create an item under group `:id` |
| GET | `/master-items`, `/master-items/:id` | ✔ `masters.manage` | master items |
| PUT/DELETE | `/master-items/:id` | ✔ `masters.manage` | update / delete an item |
| GET | `/activity-logs`, `/activity-logs/:id` | ✔ `activity_log.read` | generic request activity trail |
| GET | `/audit-trail/logs`, `/audit-trail/logs/:id` | ✔ `audit_trail.read` | entity change (old/new value) trail |
| GET | `/audit-auth/logs`, `/audit-auth/logs/:id` | ✔ `audit_auth.read` | login-attempt trail |
| GET | `/health` | – | not under `/api/v1`; liveness + build metadata |

## Security notes worth knowing before deploying

- Access and refresh tokens are signed with **different secrets** on
  purpose, so a leaked refresh secret alone can't mint access tokens (see
  `jwt_service.rs`); each `decode_*_token` also checks the token's
  `token_type` claim, so a refresh token can never be replayed as an access
  token even if the two secrets were ever misconfigured to be equal.
- The rate limiter only trusts `X-Forwarded-For` when `rate_limit.trust_proxy`
  is `true` — leave it `false` unless the app genuinely sits behind a proxy
  that overwrites that header, or a client can spoof it to bypass rate
  limiting entirely.
- Uploaded files are stored under a server-generated UUID name; the
  client-supplied filename is sanitized to its final path component and
  used only for display (`Content-Disposition`), never to build a
  filesystem path.
- `forgot_password` always returns `200` and never reveals whether an email
  is registered.

## Known gaps / TODOs left on purpose

- `forgot_password` only generates and logs a reset token server-side;
  wiring an actual email/notification provider is left as an integration
  point (see the comment in `AuthServiceImpl::forgot_password`).
- `databases/mysql/` is an empty skeleton (no migrations were ever added
  there) — only the PostgreSQL migrations are implemented against, and
  `sqlx`'s `postgres` feature is the only DB driver enabled in `Cargo.toml`.
- Seeders are plain idempotent SQL, not wired into `sqlx migrate` — you run
  them yourself (step 4 above); `migration::run_seeders` exists but is left
  commented out in `main.rs` since seeding automatically on every boot
  isn't something you generally want outside a fresh dev environment.

## Fixes applied in this pass

Found while auditing the codebase; all three are fixed in the accompanying
patch:

1. **Router failed to start at all.** `masters/presentation/routes.rs`
   registered `/masters/:id` and `/masters/:group_id/items` — two routes
   sharing the same dynamic path segment position but with *different*
   parameter names. Axum's router (built on `matchit`) requires one
   consistent parameter name per segment position across every route on a
   `Router`; mixing names there makes `Router` construction panic
   immediately on startup with `insertion failed due to conflict with
   previously registered route: /masters/:id` (the exact failure mode
   reported upstream in `tokio-rs/axum#1498`). Fixed by renaming the nested
   route to `/masters/:id/items` — the handler is unaffected, since axum's
   single-value `Path<i64>` extractor binds positionally, not by name.
2. **`role.manage` and `permission.manage` were never seeded.** Every
   handler in `role::presentation::handler` and
   `permission::presentation::handler` is gated behind those two
   permissions, but no seeder ever inserted them into the `permissions`
   table — meaning nobody, not even the seeded admin, could ever be
   granted them, so the entire `/roles` and `/permissions` APIs were
   unreachable (`403`) out of the box. Added
   `013_seed_role_and_permission_management.sql`, following the same
   pattern as every other module's permission seeder.
3. **`audit_auth.read` was never seeded.** Same class of bug as above, for
   `audit_auth_log::presentation::handler`, which checks
   `"audit_auth.read"` — but only the now-unused `audit.read` (predating
   the module being split into `audit_auth_log`/`audit_trail_log`) existed
   in `001_seed_roles_and_permissions.sql`. Added
   `012_seed_audit_auth_log.sql`, mirroring
   `010_seed_audit_trail_log.sql`.

If you're applying the patch to a database that was already seeded before
this fix, just re-run the seeders (step 4 above) — every statement is
`ON CONFLICT DO NOTHING`, so it's safe to re-run in full.
