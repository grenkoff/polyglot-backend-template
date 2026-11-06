SHELL := /bin/bash
.DEFAULT_GOAL := help
CORE := services/core
COMPOSE := env UID=$(shell id -u) GID=$(shell id -g) docker compose
RUST_COVERAGE_IGNORE := '(^|/)(main|build)\.rs$$'

.PHONY: help
help:
	@printf '%s\n' \
	  'make env           create services/core/config/.env from the template' \
	  'make proto         regenerate Python gRPC code from proto/' \
	  'make rust          fmt, clippy, tests with 100% coverage, eo-lint' \
	  'make core          wemake-django-template CI for core in Docker' \
	  'make ty            second type checker for core (non-blocking)' \
	  'make mutants       mutation testing for Rust and core' \
	  'make up            start the whole system' \
	  'make down          stop the whole system' \
	  'make e2e           check a request through every service' \
	  'make commitlint    check commit messages (Conventional Commits)' \
	  'make check         everything CI runs'

.PHONY: env
env:
	@test -f $(CORE)/config/.env || ( \
	  cp $(CORE)/config/.env.template $(CORE)/config/.env && \
	  sed -i "s/__CHANGEME__/$$(head -c 48 /dev/urandom | base64 | tr -d '/+=')/" $(CORE)/config/.env && \
	  echo "created $(CORE)/config/.env" )

.PHONY: proto
proto:
	./scripts/proto.sh

.PHONY: rust
rust:
	cargo fmt --all --check
	cargo clippy --workspace --all-targets --locked
	cargo llvm-cov --workspace --locked \
	  --ignore-filename-regex $(RUST_COVERAGE_IGNORE) --fail-under-lines 100
	cargo run -q -p eo-lint -- services tools

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
	  $(COMPOSE) run --user=root --rm web ./docker/django/ci.sh

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

.PHONY: check
check: commitlint rust core mutants e2e
	$(MAKE) down
