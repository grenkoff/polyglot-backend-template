# Instructions for AI agents

## Purpose

This repository is a template for backend systems built by AI agents under
human direction. A new project starts as a copy of it
(`gh repo create <name> --template grenkoff/polyglot-backend-template`)
and keeps its stack, structure, rules and checks.

Roles:

- **The human** owns the product: sets tasks with a sentence in the chat,
  approves Issues, makes decisions, reviews pull requests, decides who
  merges each of them, releases to production.
- **The agent** implements tasks: turns a task into an Issue, shows its text
  to the human and creates it only after approval; writes tests and code,
  keeps every check green, opens pull requests, asks the human who merges
  each pull request, and asks the human whenever a decision is not covered
  by these rules.

These rules apply to every change in this repository and in every project
created from it. When a rule and a task conflict, stop and ask the human;
never break a rule silently.

## Getting started

Read before the first change, in this order:

1. this file;
2. `docs/architecture.md`: services and how they talk;
3. `docs/elegant-objects.md`: object design rules;
4. `docs/tdd.md`: how code is written;
5. `services/core/README.md` and `services/core/docs/`: the Django service
   from wemake-django-template.

First steps in a new project created from the template (the agent performs
all of them, including the GitHub settings):

1. Install the tools (see "Developer environment") and run `make setup`.
2. Create the branches: `develop` from `main`, then rename `main` to
   `production`; make `production` the default branch.
3. Configure the repository: rebase merge only, delete branches after
   merge, protect `develop` and `production` (pull requests only, every
   required CI job green).
4. `make env`, then `make check`: everything must be green before the first
   task.
5. Replace the project name and description in `README.md`; keep the
   example (`hello`, `greetings`) until the first real service exists.

## Repository map

| Path | What it is |
|------|------------|
| `proto/` | gRPC contracts, the single source of truth between services |
| `contracts/` | Rust crate with the code generated from `proto/`; every Rust service depends on it |
| `services/core/` | main Django service: public HTTP API, admin, its database |
| `services/<name>/` | Rust services (`hello` is the example) |
| `tools/eo-lint/` | Elegant Objects linter for Rust |
| `scripts/` | code generation, end-to-end check, checks and agent hooks |
| `docs/` | architecture, Elegant Objects, TDD, patterns registry, cookbook |
| `.github/` | CI workflow, Dependabot |
| `.claude/` | Claude Code hooks, commands and subagents (see "Agent tools") |
| `compose.yml` | the whole system for `make up` |
| `Makefile` | every command, see "Commands" |
| `Cargo.toml` | the Rust workspace and its shared lints |

Never edit by hand, regenerate instead:

- `services/core/server/contracts/`: `make proto`;
- `docs/patterns.md`: `make patterns`;
- `CHANGELOG.md`: `make changelog`;
- `Cargo.lock`, `services/core/poetry.lock`, `package-lock.json`: change
  dependencies with `cargo`, `poetry`, `npm`;
- migrations in `services/core/server/apps/*/migrations/`: create with
  `makemigrations`, never edit one that was already applied.

Never commit: `target/`, `node_modules/`, `mutants/`, `mutants.out/`,
coverage reports, `services/core/config/.env` (all are in `.gitignore`).

## Stack

Changing anything in this table needs the human's approval.

| Area | Choice |
|------|--------|
| Main service | Python 3.14, Django 6.1 from [wemake-django-template](https://github.com/wemake-services/wemake-django-template) |
| HTTP API | [django-modern-rest](https://github.com/wemake-services/django-modern-rest) (`dmr`) with `msgspec`; not DRF, not Django Ninja |
| Python dependencies | Poetry (as wemake-django-template does) |
| Python checks | ruff, [wemake-python-styleguide](https://github.com/wemake-services/wemake-python-styleguide), mypy strict, ty (second, non-blocking), import-linter |
| Python tests | pytest, hypothesis, schemathesis + tracecov, mutmut |
| Other services | Rust (stable), one Cargo workspace |
| Rust libraries | tokio, axum (HTTP), tonic + prost (gRPC), async-nats, thiserror, tracing |
| Rust database access | [sqlx](https://github.com/launchbadge/sqlx), see below |
| Rust checks | rustfmt, clippy pedantic, `eo-lint`, cargo-llvm-cov, cargo-mutants |
| Contracts | protobuf, checked by buf |
| Messaging | NATS with JetStream |
| Database | PostgreSQL, one database per service that needs one |
| Runtime | Docker Compose |
| CI and updates | GitHub Actions, Dependabot |
| Agent tooling | Probity (TDD hook), commitlint (Node.js, tooling only) |

Why sqlx for Rust services:

- plain SQL instead of an ORM: no Active Record, so database rows map to
  immutable objects and queries live in their own objects, as Elegant
  Objects requires;
- queries are checked against the real schema at compile time (`query!`,
  `query_as!`), so a typo or a type mismatch fails the build, not
  production;
- async on tokio, the runtime the services already use;
- migrations as plain SQL files with `sqlx migrate`, one tool for queries
  and schema.

## Architecture

| Service | Language | Role |
|---------|----------|------|
| `core` | Python, Django | the only public service: HTTP API, auth, admin |
| `core-events` | same image as `core` | consumes events from NATS |
| `<name>` | Rust | internal services; `hello` is the example |
| `nats` | — | message broker |
| `db` | — | PostgreSQL of `core` |

Rules:

1. **Only `core` is public.** Internal services are reachable only inside
   the Docker network; clients never call them directly.
2. **Requests go over gRPC, facts go over NATS.** A caller that needs an
   answer uses gRPC; a service that reports something that happened
   publishes an event and does not wait for anyone.
3. **Each service owns its data.** No service reads or writes another
   service's database; it asks over gRPC or listens to events.
4. **Services are stateless.** State lives in databases and NATS streams,
   so any container can be restarted or replicated.
5. **Configuration only from environment variables** (12-factor); defaults
   suit local development, secrets have no defaults.
6. **Every service has a health check** used by Docker Compose and CI.
7. **Logs are structured** (structlog in Python, tracing in Rust) and go to
   stdout.

## Contracts

Contracts in `proto/` are the single source of truth between services.
A change that crosses a service boundary starts with the contract.

Layout and naming:

- `proto/<service>/v<N>/<service>.proto`, package `<service>.v<N>`
  (`proto/hello/v1/hello.proto`, package `hello.v1`);
- gRPC: service `<Name>Service`, messages `<Method>Request` and
  `<Method>Response`;
- events: a message named as a fact in the past tense (`Greeted`),
  published to the NATS subject `<service>.v<N>.<event>` in snake case
  (`hello.v1.greeted`); the payload is that protobuf message.

Evolution:

1. Never change or reuse a field number; mark removed fields `reserved`.
2. Additive changes (new field, new method, new event) stay in the same
   version.
3. A breaking change goes to a new package `v<N+1>`; the old version keeps
   working until every client has moved, then it is removed in a separate
   pull request.
4. `buf lint`, `buf format` and `buf breaking` (against `develop`) run in
   CI; a breaking change outside a new version fails the build.

Generated code:

- Python: `make proto` writes `services/core/server/contracts/`; commit it
  together with the `.proto` change, CI checks it is up to date.
- Rust: the `contracts` crate generates it at build time with
  `contracts/build.rs`; it is never committed and is the only place where
  generated Rust code lives, so lints of the workspace do not apply to it.
- Generated code is never edited by hand.

## Design principles

Judgement rules, applied to every change:

- **KISS**: the simplest thing that works; boring over clever.
- **YAGNI**: build only what the current task needs; no speculative
  abstraction, configuration or extension points.
- **Separation of concerns**: business logic, I/O and entrypoints live in
  different layers and depend in one direction only (see the layers below).
- **Fail fast**: invalid input or state raises (Python) or returns an
  error (Rust) at once; never continue with bad data.
- **Least astonishment**: code does what its name says and nothing more;
  no hidden side effects.
- **Boy Scout rule**: leave touched code cleaner than you found it, in the
  same pull request, without changing behaviour that is not covered by the
  task.
- **SOLID**: Elegant Objects make these natural: one reason to change per
  object, extension by wrapping objects (decorators) instead of editing
  them, any implementation usable wherever its interface is expected,
  narrow interfaces, dependencies injected through constructors.

Layers:

| Service | Layers, outer to inner | Enforced by |
|---------|------------------------|-------------|
| `core` (Python) | `api` → `infra` → `models` → `logic`; apps independent of each other | import-linter (`services/core/.importlinter`) |
| Rust services | `api` (tonic, axum) → `infra` (sqlx, async-nats, gRPC clients) → `domain`; `main.rs` wires them | `eo-lint` rule EO010 |

EO010: modules under `src/domain/` import no I/O crate (`tokio`, `tonic`,
`axum`, `sqlx`, `async_nats`, `prost`) and nothing from `api` or `infra`.
The domain defines traits; `infra` implements them.

Size and complexity: wemake-python-styleguide in Python; in Rust clippy
`too_many_lines` (40), `too_many_arguments` (4), `cognitive_complexity`
(10) from `clippy.toml`.

A function that trips a limit does too much: split it. Never raise a limit
or silence it.

## Elegant Objects

We follow [Elegant Objects](https://www.elegantobjects.org/) by Yegor
Bugayenko. Details and examples: `docs/elegant-objects.md`.

- **Rust: every rule, always.** `eo-lint` and clippy check them in CI.
- **Python: every rule where Django and Pythonic style allow.** The
  framework wins only where it dictates the shape of the code (models,
  managers, `Meta`, settings, management commands); our own code follows
  the rules.

The rules in short:

1. Objects are named after what they are, not what they do: no `-er`/`-or`
   names (`Manager`, `Handler`, `Parser`, `Helper`, `Util`).
2. No getters and setters (`get_*`, `set_*`).
3. Objects are immutable (no `&mut self`; frozen `attrs`/`msgspec` in
   Python).
4. Every public method implements an interface (trait, `typing.Protocol`).
5. No procedures, utility classes or static methods.
6. No public fields; transfer objects at the API boundary are the only
   exception in Python.
7. No global state and no singletons; dependencies come through
   constructors.
8. Constructors only assign fields.
9. No reflection or type checks in business logic.
10. No `NULL`: absence is explicit (`Option`/`Result`, exceptions, empty
    collections).
11. Composition over inheritance; decorators over boolean flags.
12. Small objects: few fields, short methods.

There are no exemptions and no syntax to write one. When a framework or a
crate seems to leave no way to follow a rule, stop and ask the human with
options, reasoning and a recommendation (for example, wrap the foreign type
in an adapter object, or change the rule in `eo-lint` for everyone).

## Design patterns

Patterns from the Gang of Four catalogue are shared vocabulary and a
toolbox, never a target. The `pattern-reviewer` subagent (see "Agent
tools") gives advice and never blocks.

- **A pattern must earn its place**: it removes real duplication, decouples
  a real seam, or tames conditional logic that keeps growing.
  KISS and YAGNI break every tie.
- **Rule of three**: do not abstract before the third concrete case.
- **No speculative patterns**: a strategy with one implementation, a
  factory that makes one thing, an interface with one implementation and
  no test fake, indirection with one caller: delete it.
- **Prefer the lighter form** while it scales: a closure, a `match`/dict
  dispatch, a small value object.

Smell → pattern:

| Smell in the change | Pattern |
|---------------------|---------|
| a growing `if`/`match` on a type or mode choosing behaviour | Strategy: one object per behaviour behind a trait/protocol |
| repeated conditional construction | Factory as an object implementing a trait (never a static method) |
| an external API that does not fit our interfaces | Adapter |
| behaviour added around an object (retry, cache, logging, validation, politeness) | Decorator: wrap, do not edit |
| callers reach into a subsystem's internals | Facade |
| one fact must trigger several unrelated reactions | Observer: a NATS event |
| behaviour depends on a lifecycle status with transitions | State: one immutable object per state |
| operations that must be queued, replayed or undone | Command |

Forbidden by Elegant Objects, use instead:

- Singleton → one instance created in the composition root and injected;
- Template Method (inheritance) → Decorator or Strategy (composition);
- static factory methods → a factory object.

Where patterns are recorded:

- An object that implements a pattern starts its one-line docstring
  (`///` in Rust) with the pattern name:
  `/// Decorator: adds a polite ending to another greeting.`
- Allowed names: Decorator, Strategy, Adapter, Facade, Factory, Observer,
  State, Command, Composition root.
- `make patterns` generates `docs/patterns.md` (pattern → object → file)
  from the code. Never edit it by hand; commit it with the change.
- CI fails when `docs/patterns.md` is out of date or a docstring uses a
  name outside the list.
- Read `docs/patterns.md` before adding a pattern: reuse what exists.

## Language idioms

Write idiomatic code. Lintable recipes are checked in CI: ruff rule groups
in Python (`C4`, `PERF`, `PTH`, `RET`, `TRY`, `LOG`, `G`, `SIM`, `FURB`
and others in `services/core/pyproject.toml`) and clippy pedantic in Rust.
The rest is judgement; the `pattern-reviewer` subagent (see "Agent tools")
gives advice and never blocks.

Python and Django:

- **Queries**: no N+1 (`select_related`, `prefetch_related`); fetch only
  what is used (`values_list`, `only`); `exists()` and `count()` instead of
  loading rows; bulk operations for many rows.
- **Transfer objects**: frozen `msgspec.Struct` at the API boundary,
  frozen `attrs` classes inside.
- **Logging**: `structlog` with keyword fields
  (`log.info('greeting received', name=name)`), never f-strings in log
  messages and never `print`; `log.exception` inside `except`.
- **Collections**: comprehensions instead of loop-and-append; a plain loop
  when the body has side effects or does not read well as a comprehension.
- **Dispatch**: `match` for several branches on an enum, a `Literal` or a
  structure; `if` for two branches or ranges.
- **Paths**: `pathlib`, never `os.path`.

Rust:

- **Errors**: `thiserror` types per module, `?` to propagate; `anyhow`
  only in `main.rs`.
- **Iterators** instead of index loops; `collect` into the target type.
- **Exhaustive `match`**: no `_ =>` arm that hides a new variant of our own
  enums.
- **Borrowing**: take `&str`/`&[T]` in methods, `impl Into<String>` in
  constructors; clone only when ownership is really needed.
- **Newtypes** for domain values (an `Email`, not a `String`).
- **Async**: never block the runtime; CPU-heavy or blocking work goes to
  `spawn_blocking`.
- **Logging**: `tracing` with fields (`tracing::info!(%name, "greeted")`).
- **Database**: `sqlx::query!`/`query_as!` macros only, so every query is
  checked at compile time.

## General code rules

- **English only**: everything in the repository (code, identifiers,
  strings, docstrings, logs, error messages, file names, documentation,
  commits, pull requests, Issues).
- **Line length**: 88 in Python (the [Black default](https://black.readthedocs.io/en/stable/the_black_code_style/current_style.html#line-length),
  found to give noticeably shorter files than 79 or 80, after Raymond
  Hettinger's ["Beyond PEP 8"](https://youtu.be/wf-BqAjZb8M?t=260)),
  100 in Rust (the [Rust style guide](https://doc.rust-lang.org/style-guide/)
  default). Formatters enforce it.
- **SQL in code** (raw Django SQL, `sqlx` macros) follows the same limit:
  break lines at `SELECT` columns, `FROM`, `JOIN`, `ON`, `WHERE`, `AND`,
  `VALUES`; never split a quoted literal.
- **Names**: full words, no abbreviations except well-known ones (`id`,
  `url`, `http`); booleans read as statements (`polite`, `expired`).
- **Errors**: specific types (exception classes in Python, `thiserror`
  enums in Rust); never a bare `except`, never swallow an error; every
  error either is handled or reaches the edge of the service, where it
  becomes an HTTP or gRPC status.
- **Timeouts**: every network call (gRPC, HTTP, NATS request, database)
  has an explicit timeout taken from settings.
- **Logs**: what happened and with which identifiers; never secrets,
  tokens, passwords or personal data.
- **No magic values**: numbers and strings with meaning become named
  constants or settings.

## DRY and dead code

- **DRY**: one piece of knowledge lives in one place; extract a shared
  object or constant instead of repeating it.
- **No false DRY**: code that only looks alike but changes for different
  reasons stays separate.
- **Delete dead code as you go**: unused functions, objects, imports,
  dependencies, settings, files. Version control remembers it.

Machine-enforced (`make check` and CI):

| What | Python | Rust |
|------|--------|------|
| unused imports and variables | ruff | rustc `unused` (denied) |
| unused functions, classes, methods | vulture | rustc `dead_code` (denied) |
| unused dependencies | deptry | cargo-machete |
| duplicated code | jscpd (both languages, from 8 lines) | jscpd |

Framework entrypoints that look unused to vulture (Django views, admin
classes, management commands, settings) go to
`services/core/whitelist_vulture.py`. The agent asks the human before
adding an entry there; never keep real dead code alive through it.

## Data integrity

Each database is the source of truth for its service. Other services and
NATS are outside any transaction.

- **Atomicity**: a change that writes more than one row or table as one
  logical unit runs in one transaction (`transaction.atomic()` in Django,
  `pool.begin()` … `commit()` in sqlx). Never leave a half-applied state.
- **Consistency**: invariants live in the database (constraints, foreign
  keys, `CHECK`) and in value objects; a violation fails fast.
- **Isolation**: read-modify-write under possible concurrency re-reads
  the row with a lock inside the transaction
  (`select_for_update()` in Django, `SELECT … FOR UPDATE` in sqlx).
  Keep transactions short.
- **No I/O inside a transaction**: gRPC calls, HTTP requests and NATS
  publishing happen before the transaction or after it commits, never
  while it holds locks.
- **Events after commit**: publish an event only once the data it reports
  is committed (`transaction.on_commit` in Django, after `commit()` in
  sqlx). When losing an event is not acceptable, ask the human about a
  transactional outbox.
- **Idempotent consumers**: NATS JetStream may deliver an event again;
  handling the same event twice must not change the result (check by the
  event id in the same transaction as the write).

Machine-enforced:

- Python: `scripts/check_transactions.py` (in `make check` and CI) rejects
  gRPC, HTTP and NATS calls inside `transaction.atomic()`;
- Rust: `eo-lint` rule EO011 rejects an `.await` on anything but the
  transaction itself between `begin()` and `commit()`.

## Comments, docstrings and directives

**Comments are forbidden.** Nothing that the interpreter or compiler
ignores: no `#` comments in Python, no `//` and `/* */` in Rust, no
comments in configs, scripts, Dockerfiles, Compose files, workflows,
`.proto` files. If code needs a comment to be understood, rewrite it:
rename, extract an object, split the method, let the test describe the
behaviour.

**Docstrings are mandatory and one line long.** Every module, class,
function and method, private ones included, starts with exactly one line
that answers "what does it do?":

- Python: `"""Builds greetings with the hello service over gRPC."""`;
- Rust: `//!` for a module, `///` for an item
  (`/// Decorator: adds a polite ending to another greeting.`);
  these are documentation attributes, not comments.
- A pattern name goes first when the object implements one
  (see "Design patterns").
- Not required: test functions (their names describe the behaviour),
  empty `__init__.py`, migrations, generated code.

**Tool directives are forbidden**: `# noqa`, `# type: ignore`,
`# pragma: no cover`, `#[allow]`, `#[expect]`, `#[rustfmt::skip]` and the
like. Fix the cause instead. When there is truly no other way, stop and
ask the human with options, reasoning and a recommendation.

Existing comments and directives from wemake-django-template stay as they
are. When you change a line that carries one, delete it together with the
change; do not rewrite or shorten it.

Machine-enforced: ruff `D` rules and a one-line check for Python docstrings,
`missing_docs` (denied) and `eo-lint` for Rust doc lines, a comment and
directive scan over the diff in CI.

## Test-driven development

All code is written test-first. Details: `docs/tdd.md`.

Before the first test of a task, show the acceptance criteria as a list of
tests (3–8 lines) and wait for the human's "ok".

Then, for every unit of behaviour:

1. **Red**: write one test, run it, and show the real failure line in the
   chat. It must fail on an assertion or a status code, not on a missing
   import or type: create an empty skeleton first (a class or a function
   that returns a wrong value).
2. **Green**: write the minimum code that makes the test pass. Nothing
   "while I'm here": write it down and do it in a later cycle.
3. **Refactor**: only on green. Clean names, duplication and Elegant
   Objects rules; all checks of `make check` stay green.

Each cycle is three commits: `test(<scope>): …` → `feat|fix(<scope>): …`
→ `refactor(<scope>): …` (skip the last one when nothing needed
refactoring). Their order in the branch history is the proof of TDD; do
not squash them inside the branch. Pull requests are merged by rebase, so
the cycles stay in `develop` too.

Special cases:

- **A bug**: the first test reproduces it.
- **Refactoring without a change in behaviour**: first characterization
  tests that pin the current behaviour, then refactor commits.
- **Not test-first by nature**: migrations, configuration, generated code,
  documentation. Say so explicitly in the chat when skipping the cycle.

The Probity hook (`probity.config.ts`) blocks an implementation write that
has no failing test behind it; if it blocks, write the test.

## Tests

| Level | Python (`core`) | Rust | Runs in |
|-------|-----------------|------|---------|
| Unit | `services/core/tests/`, mirrors `server/` | `#[cfg(test)] mod tests` next to the code | `make core`, `make rust` |
| Integration | real fakes: an in-process gRPC server, PostgreSQL from pytest-django | `tests/` of the crate; NATS and PostgreSQL in Docker (testcontainers) | same |
| API contract | schemathesis + tracecov against the OpenAPI schema | — | `make core` |
| End-to-end | `scripts/e2e.sh` through every service | same | `make e2e` |
| Mutation | mutmut | cargo-mutants | `make mutants` |

Rules:

- **One behaviour per test**; the test name says what the object does
  (`refuses_blank_names`, `test_greeting_hello_is_down`).
- **Through the public interface**: traits, protocols, HTTP and gRPC APIs,
  never private details; refactoring must not break tests.
- **Fakes, not mocks**: a small real implementation of the same interface
  (`events::Memory`, the fake gRPC server). No `unittest.mock`, no
  `mockall`.
- **Deterministic**: no `sleep` to wait for something (wait for a signal or
  use a timeout), time and randomness come through injected objects, tests
  run in random order (pytest-randomly), network only to local fakes and
  containers.
- **Fast**: a unit test finishes in milliseconds; every test has a
  timeout (pytest-timeout).
- **Test file names** say what is tested, never an Issue number
  (`test_remote.py`, not `test_remote_42.py`).

Gates (CI fails below them; never lower a gate):

| Code | Coverage |
|------|----------|
| Domain logic: `server/apps/*/logic/` in Python, `src/domain/` in Rust | 100% of lines (and branches in Python) |
| Everything else | at least 80% overall |

Most defects live in a small share of the code: the domain rules. That
share is covered completely; glue and infrastructure get the 80% floor and
are also exercised by `make e2e`. Test-first code reaches far above 80%
anyway: the floor catches code written without tests, it is not a target.

- Mutation testing: zero surviving mutants in covered code; it is what
  proves the tests check behaviour, not just run lines.
- Composition roots (`main.rs`, `build.rs`) are excluded in the tool
  configuration, never with a directive.

## Typical tasks

Every task starts and ends the same way:

- **Start**: Issue approved by the human → branch from `develop` (see
  "Issues, branches, worktrees") → acceptance criteria as a test list,
  human says "ok".
- **End**: `make check` green → pull request to `develop` → ask the human
  who merges it.

Step-by-step recipes live in `docs/cookbook.md` (`/cookbook`); in short:

**Add an HTTP endpoint to `core`**
1. Value objects for the request and the response in `logic/value_objects.py`
   (frozen `msgspec.Struct`).
2. Red/green in `tests/test_apps/<app>/test_api/`: status codes, body,
   errors.
3. A use case object in `logic/usecases/`, collaborators as protocols in
   `logic/`, implementations in `infra/`, bound in `server/implemented.py`.
4. A `dmr` controller in `api/views.py` with every error response declared
   (`modify(extra_responses=...)`), routed in `api/urls.py`.
5. schemathesis must pass for the new operation.

**Add a Django app**
1. `server/apps/<app>/` with `logic/`, `infra/`, `api/`, `models.py` as
   needed; add it to `INSTALLED_APPS`.
2. Include its router in `server/urls.py`.
3. Add the app to the layer contract in `.importlinter`.

**Add a Rust service**
1. Contract first: `proto/<name>/v1/<name>.proto`, compiled in
   `contracts/build.rs` and exposed as a module in `contracts/src/lib.rs`.
2. `services/<name>/` with `src/domain/`, `src/infra/`, `src/api/`,
   `src/main.rs`; add it to the workspace `members`.
3. A `Dockerfile` like `services/hello/Dockerfile`; a service with a health
   check in `compose.yml`.
4. Its client in `core`: `make proto`, an adapter in `infra/`.

**Add a gRPC method or an event**
1. Change `proto/` (an additive change, or a new version for a breaking
   one), `make proto`.
2. Red/green on the server side, then on the client side.
3. An event: the publisher after commit, the consumer idempotent
   (see "Data integrity").

**Add a model and a migration**
1. Django: change `models.py`, `makemigrations`, one logical change per
   migration, backward compatible (see "Migrations").
2. Rust: `sqlx migrate add <name>`, plain SQL, same rules.

**Fix a bug**
1. A test that reproduces the bug (red), then the fix (green).

**Remove the example** (`hello`, `greetings`), once the first real service
exists, in its own pull request: services, contracts, generated code,
tests, Compose entries, `docs/patterns.md`.

## Migrations

Migrations run before the new code starts, while the old code still serves
traffic (`core-migrate` in Compose, the same on deploy). So every migration
must work with the code that is currently in production.

1. **Backward compatible only: expand → deploy → contract.**

   | Step | Pull request | Contents |
   |------|--------------|----------|
   | Expand | PR 1 | new table or column, backfill, code switched to the new one; the old object untouched |
   | Deploy | — | PR 1 reaches production |
   | Contract | PR 2 | only `DROP`, rename, `SET NOT NULL`, new `UNIQUE`; no code that depends on them |

   PR 2 is opened as a draft and becomes ready only after PR 1 is in
   production. Destructive changes never ship together with the code that
   stops using the old object.
2. **One logical change per migration**, one migration per pull request
   where possible.
3. **Never edit a migration that was applied anywhere** (including another
   developer's database or CI); write a new one.
4. **Reversible** where feasible (Django `reverse_code`, sqlx `down`
   files); an irreversible migration is a decision to ask the human about.
5. **Short locks**: every migration runs with `lock_timeout = 5s` and a
   statement timeout; a large backfill raises its own `lock_timeout` for
   its transaction. A migration that fails on `lock_timeout` is the right
   outcome: retry the deploy instead of blocking production. Before
   dropping a table, drop its foreign keys to busy tables first.
6. **One head**: two migrations with the same parent break the deploy.
   `makemigrations --check` in CI catches it; resolve with
   `makemigrations --merge` before merging into `develop`.
7. **In a release pull request**, list every migration as additive or
   contract (with the expand pull request it depends on). A contract
   migration whose expand step is not in production blocks the release.

Machine-enforced: `django-migration-linter` (`lintmigrations`) and
`django-safe-migrations` (`check_migrations`) from wemake-django-template,
`makemigrations --check`, migration tests (`django-test-migrations`).

## Security and supply chain

- **Secrets never enter the repository.** Keys, tokens and passwords come
  from environment variables; `config/.env` is ignored, only
  `config/.env.template` with placeholders is committed. gitleaks scans
  every pull request and the history in CI; a finding blocks the merge.
  If a secret ever lands in git, rotate it at once: deleting the commit
  does not help.
- **Validate at the boundary.** Every input from outside (HTTP, gRPC,
  events) is parsed into a strict value object first; nothing unvalidated
  goes deeper (`StrictMsgspecSerializer`, prost types, value objects).
- **Only `core` is exposed**; internal services listen inside the Docker
  network only.
- **Security lint stays on**: ruff `S` (bandit) in Python, clippy in Rust;
  never disable a rule to pass.
- **Vulnerable dependencies block the merge**: pip-audit for Python,
  cargo-deny (advisories, licenses, duplicate versions) for Rust,
  `npm audit` for tooling. Fix by updating; when no fixed version exists,
  ask the human.
- **Containers**: run as a non-root user, base images pinned to a version
  tag and updated by Dependabot.

## Dependencies

Adding a dependency:

- **Every new dependency needs the human's approval.** The agent proposes
  it with alternatives (including "write it ourselves" and "use what we
  already have"), the reasoning and a recommendation: maintenance, size,
  license, how much of it we need.
- Python: `poetry add` (`--group dev` for tooling), never `pip install`
  into the project.
- Rust: versions live once in the root `[workspace.dependencies]`; crates
  refer to them with `.workspace = true`.
- Node.js: tooling only, exact versions (`npm install --save-dev
  --save-exact`).
- Lock files (`poetry.lock`, `Cargo.lock`, `package-lock.json`) are always
  committed with the change that touches dependencies.

Updating:

- Dependabot opens update pull requests weekly. Never merge them on
  GitHub and never push an update straight to `develop`.
- Each update goes the usual way: a branch `chore/deps-<slug>` from
  `develop`, the change applied locally, one commit with the bot's pull
  request title and number as the only line
  (`ci(deps): bump actions/checkout from 5 to 7 (#1)`), `make check`,
  our own pull request to `develop`, ask the human who merges it. The bot
  closes its pull request once the change is in `develop`.
- A major version is a task like any other: read the changelog, run the
  full checks, fix what changed (a breaking API change may compile and
  still change behaviour; only tests catch it).
- Python itself: only patch versions arrive from Dependabot; change the
  Docker image, `requires-python` in `pyproject.toml`, `.python-version`
  and `poetry.lock` together. A minor or major Python version is a
  separate task.

Removing: unused dependencies fail CI (deptry, cargo-machete); remove a
dependency in the same change that stops using it.

## Issues, branches, worktrees

**No Issue, no branch.** Every change of behaviour (`feat`, `fix`,
`refactor`, `perf`) starts with an Issue, however small. Order before the
first line of code:

1. The human states the task in the chat.
2. The agent may investigate first (read code, run things): that is
   diagnosis, not code.
3. The agent drafts the Issue in English (see "Pull requests and Issues"),
   shows it, and creates it with `gh issue create` only after the human
   approves.
4. Then the branch and the worktree.

Changes that do not alter product behaviour (configuration, documentation,
CI, tooling, dependency updates) need no Issue; they still get their own
branch and pull request.

Branches:

- `<type>/<N>-<slug>` with an Issue (`feat/12-polite-greetings`,
  `fix/15-hello-timeout`); `<type>/<slug>` without one
  (`chore/deps-syn-3`, `docs/agents-rules`, `ci/gitleaks`).
- `<type>` is a Conventional Commits type; `<slug>` is short kebab-case
  English.
- Always from a fresh `develop`:
  `git fetch origin && git worktree add ../<project>-<N> -b <branch> origin/develop`.
- Never commit to `develop` or `production` directly; they change only
  through pull requests.
- Keep the branch rebased on `develop`; force-push only your own feature
  branch, with `--force-with-lease`.

Worktrees: one worktree per branch next to the main checkout, so several
tasks never share a working tree; remove it after the pull request is
merged (`git worktree remove`).

## Commits

- **[Conventional Commits 1.0.0](https://www.conventionalcommits.org/en/v1.0.0/)**:
  `<type>(<scope>): <description>`.
  - types: `feat`, `fix`, `refactor`, `perf`, `test`, `docs`, `build`,
    `ci`, `chore`, `style`, `revert`;
  - scope: the service, app or area: `core`, `hello`, `greetings`,
    `eo-lint`, `proto`, `deps`, `ci`;
  - description: imperative mood, lowercase, no trailing period, the whole
    line within 100 characters;
  - breaking change: `!` after the scope (`feat(proto)!: remove greet v1`).
- **One line only**: no body, no footer.
- **No AI attribution anywhere**: no `Co-Authored-By` of any AI, no
  "Generated with …", no mention of AI tools. The human who runs the agent
  is the only author.
- **One logical change per commit**; a TDD cycle is the commits
  `test` → `feat|fix` → `refactor` (see "Test-driven development").
- **Every commit builds and passes linters**; tests may fail only in the
  `test(...)` commit of a red step. The last commit of a branch passes the
  whole `make check`.
- Stage files by name (`git add <paths>`), never `git add .` or `-A`, so
  nothing unrelated slips in.

commitlint checks every commit in CI (`commitlint.config.mjs`,
`make commitlint`).

## Pull requests and Issues

Both are written in English, for a human who must understand the point in
a minute. Up to about 15 lines; when it does not fit, cut, do not shrink
the font. Headings `###` or smaller only, never `#` or `##`.

- Write each paragraph and each list item on one line: GitHub shows every
  line break of an Issue or pull request body as a break, so wrapped text
  renders as a narrow column.
- Assign the human to every Issue and pull request the agent creates
  (`--assignee @me`).

**Issue** (drafted by the agent, created after the human approves):

- title: a plain sentence about the outcome
  (`Greetings fail with 500 when hello is down`);
- body: what does not work or is missing, and how it should behave;
  the cause in one line if it is already known;
- `### Acceptance criteria`: a checklist; each item becomes a test.

**Pull request**:

- target branch `develop` (release pull requests: see "Releases");
- title in Conventional Commits form, like a commit
  (`fix(greetings): return 503 when hello is down`);
- first line of the body: `Issue #N`, as a plain mention; no `Closes`
  keyword (see "Releases");
- then: what was broken or missing and what changed in behaviour;
  decisions and limits that the diff does not show (why this contract,
  this format, what was deliberately left out).

Never write in a pull request or an Issue: checks that were run (tests,
linters, coverage), how the cause was found, the history of the branch,
anything the diff already shows (renames, signatures, a list of files),
any mention of AI tools.

After each new commit in a branch with an open pull request, compare the
description with the diff and update it when the commit changed the
essence: behaviour, a contract, a limit, something deliberately left out.
Renames, formatting and review fixes that do not change the task are no
reason to touch it.

## Releases

Branches: `develop` collects finished work; `production` is what runs for
users. `production` is the default branch of the repository, so Issues
close when a release reaches it.

Versions are calendar tags: `vYYYY.MM.DD`; a second release on the same day
is `vYYYY.MM.DD.2`, a third `vYYYY.MM.DD.3` (never a hyphen: tools read it
as a pre-release).

A release:

1. Changelog: a branch `chore/release-<version>` from `develop`,
   `make changelog` (git-cliff with `cliff.toml`, built from Conventional
   Commits), a pull request to `develop`.
2. A pull request from `develop` to `production`, titled
   `chore(release): <version>`.
3. Its body lists every Issue finished since the previous release as
   `Closes #N`, one per line, in plain text (GitHub ignores keywords in
   backticks); collect them from the `Issue #N` lines of the feature pull
   requests merged into `develop` since the last release tag.
4. A `### Migrations` section classifies each migration as additive or
   contract (see "Migrations"); a contract migration whose expand step is
   not in production blocks the release.
5. CI green, the human approves.
6. Merge by fast-forward, never by rebase or squash (they rewrite commits
   and the two branches would diverge):
   `git push origin develop:production`; GitHub marks the pull request
   merged.
7. Tag and publish: `git tag <version>` on `production`, push it, and
   create a GitHub Release with the notes of this version
   (`git cliff --latest`).
8. Check that every listed Issue is closed (`gh issue view <N> --json
   state`); close the rest by hand.

Feature pull requests target `develop` explicitly (`gh pr create --base
develop`); Dependabot targets `develop` (`target-branch` in
`.github/dependabot.yml`).

## Definition of done

A task is done only when all of these are true. Never call it done before.

1. Every acceptance criterion of the Issue has a test, and every test
   passes.
2. The branch history shows the TDD cycles (`test` → `feat|fix` →
   `refactor`).
3. `make check` is green locally on the last commit.
4. Contracts and generated code match (`make proto`, `make patterns`
   produce no diff).
5. Documentation follows the change: `README.md`, `docs/`, this file when
   a rule or a command changed, `config/.env.template` for every new
   setting.
6. Migrations follow "Migrations"; a contract step is a separate pull
   request.
7. The pull request is open against `develop` with a description per
   "Pull requests and Issues", and CI on GitHub is green.
8. The human has been asked who merges it.

After the merge: remove the worktree and the local branch.

## CI

GitHub Actions (`.github/workflows/ci.yml`) runs on every pull request and
on every push to `develop` and `production`. Both branches are protected:
a pull request merges only with every required job green.

| Job | Checks |
|-----|--------|
| commits | commit messages and the pull request title (commitlint) |
| secrets | gitleaks over the changes and the history |
| contracts | `buf lint`, `buf format`, `buf breaking`; generated code and `docs/patterns.md` up to date |
| rust | rustfmt, clippy, tests, coverage (80% overall, 100% in `domain/`), `eo-lint`, cargo-machete, cargo-deny |
| rust-mutants | cargo-mutants: zero survivors |
| core | the wemake-django-template pipeline (ruff, wemake-python-styleguide, mypy, import-linter, pytest, schemathesis, Django checks, migration linters, deptry), pip-audit, vulture, transaction check, coverage (80% overall, 100% in `logic/`), mutmut |
| core-ty | ty (non-blocking: reported, never fails the pull request) |
| tooling | `npm audit`, jscpd (duplication in Python and Rust) |
| e2e | `make up` and a request through every service |

When CI is red:

1. Read the failed step's log; reproduce it locally with the same `make`
   target.
2. Fix the code, never the check: do not skip, disable, lower or retry
   blindly.
3. A test that fails only sometimes is a bug: find the cause (time, order,
   a shared resource) and fix it.
4. A change to CI itself goes through a pull request like any code; a
   change that weakens a check needs the human's approval.

## Agent tools

Claude Code configuration for the project lives in `.claude/` and is
committed. It only automates the rules of this file: an agent without
these tools follows the same rules by hand.

| Tool | Where | What it does |
|------|-------|--------------|
| Probity hook | `.claude/settings.json`, `probity.config.ts` | blocks a write to implementation code that has no failing test behind it |
| Format hook | `.claude/settings.json`, `scripts/format-hook.sh` | after every edit formats the file: `ruff format` and `ruff check --fix` for Python, `rustfmt` for Rust |
| `/patterns` | `.claude/commands/patterns.md` | runs `make patterns` and shows `docs/patterns.md`, so an existing object is reused before a new one is written |
| `/cookbook` | `.claude/commands/cookbook.md`, `docs/cookbook.md` | step-by-step recipes: a new Rust service, a Django app, an endpoint, a gRPC method, an event, a migration |
| `pattern-reviewer` | `.claude/agents/pattern-reviewer.md` | a subagent that reviews the branch diff against Elegant Objects, the pattern registry and this file before a pull request is opened |

Phrases and what they mean:

- "start the server", "run the system" → `make up`;
- "stop it" → `make down`.

Rules:

1. Never disable, skip or edit a hook to get past it. When a hook blocks,
   do what it says (for Probity: write the failing test first).
2. Start long-running processes in the background and read their logs.
   Stop them when the task is done, unless the human asked to keep them.
3. A new command, hook or subagent is added through a pull request, and
   `docs/cookbook.md` or this file describes it.

## Developer environment

Supported systems: Linux, macOS, Windows 11 with WSL2. On Windows the
repository lives inside the WSL file system (`~/projects/...`), never
under `/mnt/c`: file watching, permissions and speed break there.

Required tools (`make doctor` checks them and prints what is missing):

| Tool | Used for |
|------|----------|
| Docker with Compose v2 | `make up`, `make core`, `make e2e`, testcontainers |
| GNU Make | every command |
| rustup (toolchain from `rust-toolchain.toml`) | Rust services and `eo-lint` |
| cargo-llvm-cov, cargo-mutants, cargo-deny, cargo-machete | Rust checks |
| uv | installs Python and Poetry for the editor environment |
| Node.js LTS and npm | Probity, commitlint, jscpd |
| buf | contracts |
| gitleaks | secret scanning |

Setup from a fresh clone:

1. `make env`: creates `services/core/config/.env` from the template.
2. `make setup`: installs the Node tooling, the Rust components and the
   Poetry environment of `core` for the editor.
3. `make up`: starts the system; `make e2e` proves it works.

Editor: VS Code settings and recommended extensions are committed in
`.vscode/`. Any other editor works if it uses the same formatters
(ruff, rustfmt) and the line lengths from "General code rules".

Versions are pinned in the repository (`rust-toolchain.toml`,
`.python-version`, `.nvmrc`, `package-lock.json`, `poetry.lock`,
`Cargo.lock`, image tags in `compose.yml`). The agent never upgrades a
tool on the machine to make a check pass; a version change goes through
"Dependencies".

## Commands

Every check and every routine action is a `make` target. `make help`
lists them. The agent uses these targets, not the tools behind them,
so local runs match CI.

| Command | What it does |
|---------|--------------|
| `make help` | lists all targets |
| `make doctor` | checks that the required tools are installed |
| `make env` | creates `services/core/config/.env` from the template |
| `make setup` | installs Node tooling, Rust components and the editor environment of `core` |
| `make proto` | checks the contracts with `buf` and regenerates the Python code |
| `make patterns` | regenerates `docs/patterns.md` |
| `make rust` | rustfmt, clippy, tests, coverage, `eo-lint`, cargo-machete, cargo-deny |
| `make core` | the wemake-django-template pipeline for `core` inside Docker |
| `make ty` | ty for `core` (non-blocking) |
| `make mutants` | cargo-mutants and mutmut: zero survivors |
| `make security` | gitleaks, pip-audit, cargo-deny advisories, `npm audit` |
| `make duplicates` | jscpd over Python and Rust |
| `make up` / `make down` | starts / stops the whole system |
| `make logs` | follows the logs of the running system |
| `make e2e` | starts the system and sends a request through every service |
| `make commitlint` | checks the commits of the branch against Conventional Commits |
| `make changelog` | updates `CHANGELOG.md` with git-cliff for the next release |
| `make check` | everything CI runs, in the same order |

A new routine action gets a target here and a line in this table in the
same pull request.

## Do not

Stack and architecture:

- Do not add another web framework, ORM, broker, database, package
  manager or language. Root `package.json` holds developer tooling only.
- Do not share a database between services; a service owns its schema.
- Do not expose internal services: only `core` is public. Services talk
  through gRPC contracts and NATS events from `proto/`, never through
  each other's database or internal HTTP.
- Do not import across layers the wrong way (`domain` never imports
  `api` or `infra`).

Code:

- Do not edit generated code (`services/core/server/contracts/`, the
  `contracts` crate); change `proto/` and run `make proto`.
- Do not write comments or tool directives; do not use mocks or `sleep`
  in tests.
- Do not leave dead code, commented-out code, `TODO` or debug output.

Checks:

- Do not weaken a linter, a rule, a coverage or mutation threshold, a
  timeout or a CI job to make a check pass.
- Do not disable or bypass a hook, and do not use `--no-verify`.

Git and GitHub:

- Do not push to `develop` or `production` directly; everything lands
  through a pull request (the release fast-forward is the only exception).
- Do not force-push, rewrite published history or delete a branch that
  is not yours without the human's approval.
- Do not merge a pull request without asking the human who merges it.
- Do not merge bot pull requests on GitHub.
- Do not add AI attribution anywhere.

Secrets and data:

- Do not commit secrets, `.env` files, keys, tokens or dumps; use
  `config/.env.template` for new settings.
- Do not put real personal data in tests, fixtures or logs.
- Do not run destructive commands against shared environments
  (dropping databases, deleting volumes, `docker system prune`) without
  the human's approval.

## When to ask the human

The agent decides alone everything these rules already decide. It stops
and asks when a decision is not in this file or changes it.

Always ask before:

1. Creating an Issue (show its text first).
2. Starting work on acceptance criteria (show the test list first).
3. Adding, removing or replacing a dependency (give alternatives).
4. Adding a tool directive, a vulture whitelist entry or any other
   exception to a rule.
5. Changing a rule, a threshold, a CI job or this file.
6. Changing a public contract in a breaking way (`proto/`, the HTTP API,
   a database schema in a contract step).
7. Merging a pull request (ask who merges it) and releasing.
8. Force-pushing, rewriting history, deleting branches, data, volumes or
   anything else that cannot be undone.
9. Doing anything outside the repository: GitHub settings, external
   services, messages to people. The settings listed in "Getting started"
   are the exception: the agent applies them itself.

Also ask when:

- the task is ambiguous or two rules conflict;
- the same check fails after two honest attempts to fix the cause;
- the work grows beyond the Issue.

How to ask: one question at a time, with the options, the reasoning for
each and a recommendation. Then wait. Silence is not approval, and an
approval covers only the action it was given for.

When the agent writes or changes a rule (this file, `docs/`, linter or CI
configuration), it shows the human in the chat the exact text it wrote,
in English as committed, together with a translation into the language
of the chat.
