SHELL := /bin/bash
.DEFAULT_GOAL := help
CORE := services/core
COMPOSE := env UID=$(shell id -u) GID=$(shell id -g) docker compose
PYTHON_VERSION := $(shell cat $(CORE)/.python-version)
POETRY_VERSION := $(shell sed -n "s/.*POETRY_VERSION=\([0-9.]*\).*/\1/p" $(CORE)/docker/django/Dockerfile)
RUST_COVERAGE_IGNORE := '(^|/)(main|build)\.rs$$'

.PHONY: help
help:
	@printf '%s\n' \
	  'make doctor        check that the required tools are installed' \
	  'make setup         install Node tooling, the Rust toolchain and the core venv' \
	  'make env           create services/core/config/.env from the template' \
	  'make proto         regenerate Python gRPC code from proto/' \
	  'make patterns      regenerate docs/patterns.md from the code' \
	  'make rust          fmt, clippy, tests, coverage (80%, 100% in domain), eo-lint, cargo-deny' \
	  'make core          wemake-django-template CI for core in Docker' \
	  'make ty            second type checker for core (non-blocking)' \
	  'make mutants       mutation testing for Rust and core' \
	  'make security      gitleaks, cargo-deny advisories, npm audit, pip-audit' \
	  'make duplicates    jscpd over Python and Rust' \
	  'make up            start the whole system' \
	  'make down          stop the whole system' \
	  'make e2e           check a request through every service' \
	  'make commitlint    check commit messages (Conventional Commits)' \
	  'make changelog     write CHANGELOG.md for VERSION=vYYYY.MM.DD (git-cliff)' \
	  'make release-notes print the notes of the latest release' \
	  'make check         everything CI runs'

.PHONY: doctor
doctor:
	./scripts/doctor.sh

.PHONY: setup
setup: env
	npm ci --ignore-scripts
	rustup show active-toolchain || rustup toolchain install
	cd $(CORE) && POETRY_VIRTUALENVS_IN_PROJECT=true POETRY_VIRTUALENVS_USE_POETRY_PYTHON=true \
	  uvx --python $(PYTHON_VERSION) poetry==$(POETRY_VERSION) sync

.PHONY: env
env:
	@test -f $(CORE)/config/.env || ( \
	  cp $(CORE)/config/.env.template $(CORE)/config/.env && \
	  sed -i "s/__CHANGEME__/$$(head -c 48 /dev/urandom | base64 | tr -d '/+=')/" $(CORE)/config/.env && \
	  echo "created $(CORE)/config/.env" )

.PHONY: proto
proto:
	./scripts/proto.sh

.PHONY: patterns
patterns:
	cargo run -q -p eo-lint --bin patterns --locked -- services tools > docs/patterns.md

.PHONY: rust
rust:
	cargo fmt --all --check
	cargo clippy --workspace --all-targets --locked
	cargo llvm-cov --workspace --locked \
	  --ignore-filename-regex $(RUST_COVERAGE_IGNORE) --fail-under-lines 80
	./scripts/domain-coverage.sh
	cargo run -q -p eo-lint -- services tools
	cargo machete
	cargo deny --locked check

.PHONY: security
security: env
	gitleaks git --no-banner --redact .
	cargo deny --locked check advisories
	npm audit
	cd $(CORE) && $(COMPOSE) build web && \
	  $(COMPOSE) run --rm --no-deps web pip-audit --desc

.PHONY: duplicates
duplicates:
	npx --no-install jscpd --min-lines 8 --format python,rust --exit-code 1 --no-tips \
	  --ignore '**/server/contracts/**,**/migrations/**' services tools contracts

.PHONY: mutants
mutants: mutants-rust mutants-core

.PHONY: mutants-rust
mutants-rust:
	cargo mutants --workspace

.PHONY: mutants-core
mutants-core: env
	cd $(CORE) && $(COMPOSE) build web && \
	  $(COMPOSE) run --rm web ./docker/django/mutants.sh

.PHONY: core
core: env
	cd $(CORE) && $(COMPOSE) build web && \
	  $(COMPOSE) run --rm web ./docker/django/ci.sh

.PHONY: ty
ty: env
	cd $(CORE) && $(COMPOSE) build web && \
	  $(COMPOSE) run --rm --no-deps web ty check

.PHONY: up
up: env
	$(COMPOSE) up --build --detach --wait --wait-timeout 300

.PHONY: down
down:
	$(COMPOSE) down --volumes

.PHONY: e2e
e2e: up
	./scripts/e2e.sh

.PHONY: commitlint
commitlint:
	npx --no-install commitlint --last --verbose

.PHONY: changelog
changelog:
	@test -n "$(VERSION)" || { echo 'usage: make changelog VERSION=vYYYY.MM.DD'; exit 1; }
	npx --no-install git-cliff --tag $(VERSION) --output CHANGELOG.md

.PHONY: release-notes
release-notes:
	@npx --no-install git-cliff --latest --strip header

.PHONY: check
check: commitlint security duplicates rust core mutants e2e
	$(MAKE) down
