# rust-api-ddd

Clean Architecture / DDD backend in Rust: Axum + SQLx (PostgreSQL) + Redis, JWT auth (access + refresh), Argon2 password hashing, and RBAC (roles ↔ permissions).

## Stack

- **Web**: Axum 0.7
- **DB**: PostgreSQL via SQLx (migrations under `databases/postgresql/migrations`)
- **Cache**: Redis (cache-aside for `User`, fixed-window rate limiter)
- **Auth**: JWT access/refresh tokens (separate signing secrets), Argon2id password hashing
- **Validation**: `validator` crate via a custom `ValidatedJson<T>` extractor (Laravel-style field errors)

## Project layout

```
src/
  bootstrap/     # config, db pool, redis pool, migrations, router/AppState wiring
  shared/        # cross-cutting: errors, response envelope, cache, middleware, contracts
  modules/
    auth/        # login, refresh, logout, forgot/reset password
    user/        # user CRUD, role assignment, /me, change password
```

Each module follows `domain -> application -> infrastructure -> presentation`.
`shared::contracts` (`UserReader`, `AuditRecorder`) is what lets `auth` read user
data and write login logs without depending on `user`'s persistence types directly.

> **Naming note**: `user_repository_mysql.rs` and `auth_repository_mysql.rs` are
> actually PostgreSQL (`sqlx::PgPool`) implementations — the filenames just match
> the module skeleton that was already in the repo (`pub mod user_repository_mysql;`).
> Rename them to `..._postgres.rs` if you want the naming to match reality; just
> update the corresponding `pub mod` line in each `persistence/mod.rs`.

## Setup

1. **Postgres + Redis** (adjust to taste):
   ```bash
   docker run -d --name pg -e POSTGRES_PASSWORD=postgres -e POSTGRES_DB=rust_clean_ddd -p 5432:5432 postgres:16
   docker run -d --name redis -p 6379:6379 redis:7
   ```

2. **Config**: copy `config/config.local.yaml` (already gitignored) or `.env.example` -> `.env`
   and set real values for `database.url`, `redis.url`, `jwt.access_secret`, `jwt.refresh_secret`.

3. **Migrate**:
   ```bash
   cargo install sqlx-cli --no-default-features --features postgres
   sqlx migrate run --source databases/postgresql/migrations
   ```
   (or set `database.run_migrations_on_boot: true` in config to run them automatically on `cargo run`.)

4. **Seed** (idempotent, safe to re-run):
   ```bash
   for f in databases/postgresql/seeders/*.sql; do psql "$DATABASE_URL" -f "$f"; done
   ```
   ⚠️ `002_seed_admin_user.sql` inserts a **placeholder** password hash in bcrypt
   format. This app verifies passwords with **Argon2**, not bcrypt, so that seeded
   hash will never successfully log in. Generate a real Argon2 hash and replace it,
   e.g. with a tiny throwaway Rust snippet using the `argon2` crate already in
   `Cargo.toml`, or update the row directly:
   ```sql
   UPDATE users SET password_hash = '<argon2-hash>' WHERE email = 'admin@mail.local';
   ```

5. **Run**:
   ```bash
   cargo run
   ```
   Server listens on `server.host:server.port` from config (default `0.0.0.0:8080`).

## API

All routes are mounted under `/api/v1`, plus an unprefixed `GET /health`.

| Method | Path                          | Auth | Notes                          |
|--------|-------------------------------|------|---------------------------------|
| POST   | `/auth/login`                 | –    | `{ identifier, password }`     |
| POST   | `/auth/refresh`                | –    | rotates the refresh token       |
| POST   | `/auth/logout`                 | –    | revokes the given refresh token |
| POST   | `/auth/forgot-password`        | –    | always 200, doesn't leak emails |
| POST   | `/auth/reset-password`         | –    | `{ token, password }`          |
| GET    | `/me`                          | ✔    | current user profile            |
| PUT    | `/me/password`                 | ✔    | change own password             |
| GET    | `/users`                       | ✔ + `user.manage` | paginated, `?page&limit&search` |
| POST   | `/users`                       | ✔ + `user.manage` | create user                     |
| GET/PUT/DELETE | `/users/:id`           | ✔ + `user.manage` | read/update/soft-delete         |
| POST   | `/users/:id/roles`             | ✔ + `user.manage` | `{ role: "admin" }`             |
| DELETE | `/users/:id/roles/:role`       | ✔ + `user.manage` | revoke a role                   |

`user.manage` is exactly the permission name seeded for the `admin` role in
`001_seed_roles_and_permissions.sql`, so it lines up with the RBAC data as-is.

## Known gaps / TODOs left on purpose

- `forgot_password` only generates and logs a reset token; wiring an actual
  email/notification provider is left as an integration point.
- Only `auth` and `user` modules were scaffolded in `src/modules` (matching what
  was already in the repo). The `system_settings`, `menus`, and
  `menu_permissions` tables from the migrations don't have a module yet.
- `databases/mysql/` is an empty skeleton (no migrations were added there) —
  only the PostgreSQL migrations you added are implemented against.

## A note on this pass

This sandbox's Rust toolchain (`rustc`/`cargo` 1.75 via `apt`) is too old to
resolve/build several current crate versions (many now require the `edition2024`
Cargo manifest feature), so I could not run a full `cargo build` here to
guarantee zero compile errors. Everything was written and manually re-checked
for type/trait/borrow correctness, brace/paren balance, and schema alignment
against every migration file, but please run `cargo check` on your machine
(with a recent stable toolchain, e.g. via `rustup`) as the first step and send
me any errors it reports — they should be quick to fix.
