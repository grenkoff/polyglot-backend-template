# polyglot-backend-template

A starting point for backend projects built by AI agents and humans:
microservices with a **Django** main service and **Rust** services around it,
written in [Elegant Objects](https://www.elegantobjects.org/) style and
checked by strict CI.

| | |
|---|---|
| Main service | Django from [wemake-django-template](https://github.com/wemake-services/wemake-django-template), API on [django-modern-rest](https://github.com/wemake-services/django-modern-rest) |
| Python style | [wemake-python-styleguide](https://github.com/wemake-services/wemake-python-styleguide), ruff, mypy strict, coverage (100% of domain logic), schemathesis |
| Other services | Rust: [axum](https://github.com/tokio-rs/axum), [tonic](https://github.com/hyperium/tonic), [async-nats](https://github.com/nats-io/nats.rs) |
| Rust style | Elegant Objects on 100%: clippy pedantic + [`eo-lint`](tools/eo-lint) |
| Communication | gRPC contracts in [`proto/`](proto) ([buf](https://buf.build)), events over [NATS](https://nats.io) |
| Runtime | Docker Compose, PostgreSQL |

## Start a new project

```bash
gh repo create my-backend --template grenkoff/polyglot-backend-template --private --clone
cd my-backend
make doctor   # lists the tools that are missing
make setup    # installs Node tooling, the Rust toolchain and the core venv
make env      # creates services/core/config/.env
make up       # starts everything on http://localhost:8000
make e2e      # checks that a request goes through all services
```

Then tell your AI agent what to build. It reads [`AGENTS.md`](AGENTS.md)
(Claude Code reads it through [`CLAUDE.md`](CLAUDE.md)).

## What is inside

```
proto/               gRPC contracts, the single source of truth
services/core/       Django main service (public HTTP API, admin, database)
services/hello/      example Rust service (gRPC + health check + NATS events)
tools/eo-lint/       Elegant Objects linter for Rust
scripts/             code generation and end-to-end check
docs/                architecture and coding rules
compose.yml          the whole system
```

The example request flow:

```
POST /api/v1/greetings/ ─▶ core ─gRPC─▶ hello ─▶ NATS hello.v1.greeted ─▶ core-events
```

Keep `hello` and the `greetings` app as a reference until your own services
exist, then delete them.

## Checks

`make check` runs everything that CI runs:

| Job | What it checks |
|-----|----------------|
| secrets | gitleaks over the whole history |
| commits | commit messages and pull request titles follow [Conventional Commits](https://www.conventionalcommits.org/en/v1.0.0/), one line |
| contracts | `buf lint`, `buf format`, breaking changes on pull requests, generated Python code is up to date |
| rust | `cargo fmt`, clippy pedantic (warnings are errors), tests with line coverage of 80% overall and 100% in `src/domain/` (cargo-llvm-cov), `eo-lint`, cargo-machete, cargo-deny |
| rust-mutants | mutation testing with cargo-mutants: no mutant may survive |
| core | the full wemake-django-template pipeline: ruff, wemake-python-styleguide, import-linter, mypy, pytest with coverage of 80% overall and 100% in `logic/`, schemathesis + tracecov, Django deploy checks, migration linters, deptry, codespell, typos, yamllint; mutation testing with mutmut; plus [ty](https://github.com/astral-sh/ty) as a second, non-blocking type checker |
| tooling | `npm audit`, jscpd (duplicated Python and Rust code) |
| e2e | `docker compose up` and a request through every service |

## Test-driven development

Code is written test-first, see [docs/tdd.md](docs/tdd.md). For AI agents
it is enforced by the [Probity](https://github.com/nizos/probity) hook
(`make setup` once, then run the agent where the code lives); for
everyone by coverage gates and mutation testing in CI.

## Requirements

Linux, macOS or WSL2 with Docker, Rust (`rustup`), `protoc`,
[uv](https://docs.astral.sh/uv/), Node.js from [`.nvmrc`](.nvmrc), buf,
gitleaks and the cargo tools listed in [`AGENTS.md`](AGENTS.md#developer-environment).
`make doctor` says what is missing.

## Documentation

- [Architecture](docs/architecture.md)
- [Elegant Objects rules](docs/elegant-objects.md)
- [Rules for AI agents](AGENTS.md)
- [Django service](services/core/README.md)

## License

[MIT](LICENSE)
