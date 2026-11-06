# Instructions for AI agents

This repository is a template for backend projects. When you are asked to
build a backend on top of it, keep its structure, tools and rules.

## Read first

1. `docs/architecture.md`: services, how they talk, how to add new ones.
2. `docs/elegant-objects.md`: the coding rules. **Mandatory.**
3. `docs/tdd.md`: how we write code: test first. **Mandatory.**
4. `services/core/README.md` and `services/core/docs/`: the Django service
   generated from wemake-django-template.

## Stack (do not replace)

- Main service: **Django** in `services/core`, generated from
  [wemake-django-template](https://github.com/wemake-services/wemake-django-template).
  HTTP API with [django-modern-rest](https://github.com/wemake-services/django-modern-rest)
  (`dmr`), not DRF and not Django Ninja.
- Python style: [wemake-python-styleguide](https://github.com/wemake-services/wemake-python-styleguide)
  + ruff + mypy (strict). Dependencies via **Poetry** (as the template does).
- Other services: **Rust** in `services/<name>`, one Cargo workspace,
  **axum** for HTTP, **tonic** for gRPC, **async-nats** for events.
- Contracts: protobuf in `proto/`, checked by `buf`.
- Broker: **NATS**. Database: **PostgreSQL**. Runtime: **Docker Compose**.

## Elegant Objects

- **Rust: follow every rule, no exceptions without
  `#[cfg_attr(eo, allow(EO00N, reason = "..."))]`.**
  `eo-lint` and clippy pedantic run in CI.
- **Python: follow the rules unless Django or Pythonic style requires
  otherwise.** Frozen `attrs`/`msgspec` objects, `typing.Protocol` for
  collaborators, `@final` classes, dependencies through constructors (punq),
  no getters/setters, no `-er` names for our own classes.
- Prefer decorators (objects wrapping objects) to boolean flags.

## Test-driven development

Every change follows red → green → refactor (`docs/tdd.md`):

1. Write one failing test. **Run it and show that it fails** for the right
   reason before writing any implementation.
2. Write the minimum code to make it pass. Run the tests.
3. Refactor with the tests green.

Bugs are fixed the same way: a test that reproduces the bug comes first.
The Probity hook (`probity.config.ts`) blocks implementation writes that
have no failing test behind them; if it blocks you, write the test.

## Workflow

1. Change contracts in `proto/` first, then run `make proto`.
2. Work in TDD cycles (above). Coverage must stay at **100%** in Python and
   Rust, and mutation testing must find no surviving mutants
   (`make mutants`). `core` also checks OpenAPI coverage
   (schemathesis + tracecov).
3. Before you say the work is done, run **`make check`** and make it pass.
   It runs the same checks as CI. Never weaken a linter, a coverage threshold
   or a rule to make it pass; fix the code instead.
4. Keep the example (`hello` service, `greetings` app) until the real
   services exist, then delete it in a separate commit.

## English only

Everything in the repository is written in English: code, identifiers,
strings, documentation, commit messages, pull requests, issues.

## No comments in code

Code we write has **no comments**: no `//`, `///`, `//!` in Rust, no `#`
comments and no docstrings in Python, no comments in configs, scripts,
Dockerfiles, Compose files, workflows or `.proto` files.

If code needs a comment to be understood, it is bad code: rename things,
extract an object, split the method, make the test describe the behaviour.

Allowed:

- comments that came from wemake-django-template in `services/core`;
- tool directives that are not prose: `# noqa: <code>`,
  `# type: ignore[<code>]`, `# pragma: no cover`, `#!` shebangs,
  `# syntax=` and `# hadolint ignore=` in Dockerfiles;
- Elegant Objects exemptions as attributes:
  `#[cfg_attr(eo, allow(EO006, reason = "..."))]`.

## Commits and pull requests

- **[Conventional Commits 1.0.0](https://www.conventionalcommits.org/en/v1.0.0/)**:
  `<type>(<optional scope>): <description>`, for example
  `feat(greetings): add polite greetings`, `fix(hello): stop on broker errors`,
  `chore(deps): bump tonic`. Types: `feat`, `fix`, `refactor`, `test`,
  `docs`, `build`, `ci`, `chore`, `perf`, `style`, `revert`; breaking
  changes use `!` (`feat(api)!: rename greetings endpoint`).
- **A commit message is a single line**: no body, no footer.
- A pull request title follows the same rules: it becomes the commit
  message when the pull request is squash-merged.
- **No AI attribution anywhere**: no `Co-Authored-By: Claude ...` (or any
  other AI) trailers, no "Generated with ..." lines, no mentions of AI
  tools in commit messages, pull request titles or descriptions.
  The human who runs the agent is the only author.
- One logical change per commit; `make check` passes before every commit.
- commitlint checks all of this in CI (`commitlint.config.mjs`,
  `make commitlint`).

## Commands

| Command | What it does |
|---------|--------------|
| `make env` | creates `services/core/config/.env` from the template |
| `make proto` | regenerates Python gRPC code from `proto/` |
| `make rust` | fmt, clippy, tests with 100% line coverage and `eo-lint` for all Rust crates |
| `make core` | full wemake-django-template CI for `core` inside Docker |
| `make ty` | second type checker for `core` ([ty](https://github.com/astral-sh/ty), beta); does not block CI, but fix what it finds unless it is a known Django gap listed in `[tool.ty.overrides]` |
| `make mutants` | mutation testing for Rust (cargo-mutants) and `core` (mutmut); every surviving mutant is a missing test |
| `make up` / `make down` | starts / stops the whole system |
| `make e2e` | starts the system and checks a request goes through every service |
| `make commitlint` | checks the last commit message against Conventional Commits |
| `make check` | everything above, like CI |

## Do not

- Do not add another web framework, ORM, broker or package manager.
  (`package.json` in the root holds developer tooling only: Probity.)
- Do not share databases between services.
- Do not call internal services from outside: only `core` is public.
- Do not edit generated code in `services/core/server/contracts/`.
- Do not commit secrets: `config/.env` is ignored, use `config/.env.template`.
