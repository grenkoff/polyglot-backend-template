# Cookbook

Step-by-step recipes for the typical tasks of `AGENTS.md`. The example
service `hello` and the example app `greetings` show every step in real
code; copy their shape, not their names.

Every recipe starts and ends the same way:

- **Start**: an Issue approved by the human (for a change of behaviour),
  a branch and a worktree from `develop`, the acceptance criteria as a list
  of tests and the human's "ok".
- **Each behaviour**: red (one failing test, its failure line in the
  chat), green (the least code), refactor; commits `test` → `feat|fix` →
  `refactor`.
- **End**: `make patterns` when an object names a pattern, `make check`
  green, the `pattern-reviewer` subagent, a pull request to `develop`, and
  the question who merges it.

Run `/patterns` before writing a new object: an existing one may already
do the job or only need a decorator.

## Add an HTTP endpoint to `core`

Example: `POST /api/v1/greetings/` in `server/apps/greetings/`.

1. Value objects in `logic/value_objects.py`: the request and the response
   as frozen `msgspec.Struct`, validated on construction (`GreetingRequest`,
   `GreetingText`).
2. Red: an API test in `tests/test_apps/test_<app>/test_api/` that posts
   through the Django test client and checks the status code and the body;
   then one test per error status.
3. A use case object in `logic/usecases/<name>.py` (`greet.Greet`): a
   frozen `attrs` class that receives its collaborators as protocols from
   `logic/` (`Greetings`) and raises domain errors
   (`GreetingsUnavailableError`).
4. Implementations of the protocols in `infra/` (`remote.RemoteGreetings`),
   with every network call bounded by a timeout from settings.
5. Bind them in `server/implemented.py`, in the function of the app:
   `container.register(Greetings, remote.RemoteGreetings)` and
   `container.register(inject(greet.Greet))`.
6. A `dmr` controller in `api/views.py`: `HasContainer` and
   `Controller[StrictMsgspecSerializer]`, the handler resolves the use case,
   `@modify(extra_responses=[...])` declares every error status, and
   `handle_error` maps each domain error to its status.
7. Route it in `api/urls.py` (`dmr.routing.Router`) and include the router
   in `server/urls.py` under `router_v1`.
8. `make core`: schemathesis and tracecov must cover the new operation and
   each declared response.

## Add a Django app

1. `server/apps/<app>/` with `__init__.py`, and only the layers it needs:
   `logic/` (value objects, protocols, use cases), `infra/`, `api/`,
   `models.py`, `admin.py`.
2. Add `'server.apps.<app>'` to `INSTALLED_APPS` in
   `server/settings/components/common.py`.
3. Include its router in `server/urls.py`.
4. The layer contract in `services/core/.importlinter` covers every app
   through `server.apps.*`; `lint-imports` (in `make core`) proves the new
   app imports downwards only and nothing from other apps.
5. Tests mirror the app in `tests/test_apps/test_<app>/`.

## Add a Rust service

Example: `services/hello/`.

1. Contract first: `proto/<name>/v1/<name>.proto`, package `<name>.v1`,
   service `<Name>Service`. Compile it in `contracts/build.rs` (add the
   file to `compile_protos`) and expose it in `contracts/src/lib.rs` as
   `pub mod <name> { pub mod v1 { tonic::include_proto!("<name>.v1"); } }`.
2. `services/<name>/Cargo.toml` like the one of `hello`: every
   dependency, `contracts` and dev-dependencies included, comes from
   `[workspace.dependencies]` (`.workspace = true`), and
   `[lints] workspace = true`. Add the crate to `members` in the root
   `Cargo.toml`.
3. Layers under `src/`: `domain/` (newtypes, traits, errors; no I/O, rule
   EO010), `infra/` (adapters to NATS, sqlx, gRPC clients, settings),
   `api/` (tonic and axum entrypoints), `lib.rs` exposing them, and
   `main.rs` as the composition root that reads settings from the
   environment and wires the objects.
4. Red/green per behaviour: unit tests next to the code
   (`#[cfg(test)] mod tests`), fakes of the domain traits (like
   `events::Memory`), integration tests in `tests/` with testcontainers.
5. A `Dockerfile` like `services/hello/Dockerfile`: build stage with
   `protobuf-compiler`, a non-root runtime user, `HEALTHCHECK` against the
   health route of axum.
6. A service in `compose.yml` with its environment; services that call it
   wait for `condition: service_healthy`. It publishes no ports: only
   `core` is public.
7. Its client in `core`: `make proto`, then an adapter in `infra/` of the
   app that calls it (see "Add a gRPC method").
8. `make rust`, `make e2e`.

## Add a gRPC method

1. Add the method and its `<Method>Request` and `<Method>Response`
   messages to `proto/<service>/v<N>/<service>.proto`: an additive change
   stays in the same version, a breaking one goes to `v<N+1>`. Never reuse
   a field number; mark removed fields `reserved`.
2. `make proto` and commit the regenerated `services/core/server/contracts/`
   together with the `.proto` change.
3. Server side, red/green: a test through the tonic service in `api/`, the
   behaviour in `domain/`, the translation between protobuf and domain
   types in `api/` or `infra/`.
4. Client side in `core`, red/green: an adapter in `infra/` that maps gRPC
   status codes to domain errors and uses a timeout from settings
   (`RemoteGreetings`), tested against the in-process fake server in
   `tests/plugins/<app>/` (`hello_server.py`).

## Add an event

Example: `hello` publishes `hello.v1.greeted`, `core-events` records it.

1. A message named as a fact in the past tense in the `.proto` file of the
   publisher (`Greeted`); the subject is `<service>.v<N>.<event>` in snake
   case. `make proto`.
2. Publisher in Rust: the domain owns the event type and a trait
   (`domain::events::Events`), `infra/` implements it (`infra::nats::Nats`)
   and translates the event into protobuf; a fake (`events::Memory`)
   proves the use case publishes it.
3. Publish only after the data it reports is committed: after `commit()`
   in sqlx, inside `transaction.on_commit` in Django.
4. Consumer in `core`: protocols in `logic/` (`Inbox`, `Journal`), a use
   case that reads the inbox (`usecases.listen`), adapters in `infra/`, a
   management command as the entrypoint (`listen_greetings`) and a Compose
   service that runs it (`core-events`).
5. Make the consumer idempotent: JetStream may deliver an event again, so
   record the event id in the same transaction as the write and skip ids
   already seen.
6. `make e2e` proves the event travels through every service.

## Add a model and a migration

Django:

1. Change `models.py`, then
   `docker compose run --rm web python manage.py makemigrations <app>`;
   one logical change per migration.
2. Backward compatible only (expand → deploy → contract): a new table, a
   nullable column or one with a default, a backfill; never a `DROP`, a
   rename, `SET NOT NULL` or a new `UNIQUE` together with the code that
   stops using the old shape.
3. Reversible where feasible: `RunPython` with `reverse_code`.
4. A large backfill raises its own limits for its transaction:
   `migrations.RunSQL("SET LOCAL lock_timeout = '30s'")` before it.
5. Red/green for the code that uses the model; `make core` runs the
   migration linters, `makemigrations --check` and migration tests.

Rust (sqlx):

1. `sqlx migrate add <name>` in the crate of the service; plain SQL, the
   same rules as Django.
2. Start the migration with `SET LOCAL lock_timeout = '5s';` and
   `SET LOCAL statement_timeout = '15s';`.
3. Write a `down` file where the change can be undone.
4. Queries only with `sqlx::query!` and `sqlx::query_as!`, mapped into
   immutable domain objects in `infra/`.

## Remove the example

Once the first real service exists, in its own pull request:

1. `services/hello/`, `proto/hello/`, its lines in `contracts/build.rs`
   and `contracts/src/lib.rs`, its member in `Cargo.toml`.
2. `services/core/server/apps/greetings/`, its line in `INSTALLED_APPS`,
   its router in `server/urls.py`, `_inject_greetings` in
   `server/implemented.py`, its settings (`HELLO_GRPC_*`) in
   `server/settings/components/services.py` and `config/.env.template`.
3. Its tests: `tests/test_apps/test_greetings/`, `tests/plugins/greetings/`.
4. `hello` and `core-events` in `compose.yml`, the `HELLO_GRPC_ADDR` of
   `core`, and the request of `scripts/e2e.sh` (point it at the new
   service).
5. `make proto`, `make patterns`, `make check`.
