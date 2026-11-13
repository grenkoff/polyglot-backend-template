#!/usr/bin/env bash

set -o nounset
set -o pipefail

root="$(cd "$(dirname "$0")/.." && pwd)"
tools=(
  docker make rustup cargo-llvm-cov cargo-mutants cargo-deny cargo-machete
  protoc uv node npm buf gitleaks
)
missing=0

report() {
  if [ "$2" = ok ]; then
    printf 'ok       %s\n' "$1"
  else
    printf 'missing  %s (%s)\n' "$1" "$2"
    missing=$((missing + 1))
  fi
}

for tool in "${tools[@]}"; do
  if command -v "$tool" >/dev/null 2>&1; then
    report "$tool" ok
  else
    report "$tool" 'not on PATH'
  fi
done

if docker compose version >/dev/null 2>&1; then
  report 'docker compose v2' ok
else
  report 'docker compose v2' 'docker compose version fails'
fi

wanted="$(cat "$root/.nvmrc")"
actual="$(node --version 2>/dev/null || true)"
if [[ "$actual" == "v$wanted."* ]]; then
  report "node $wanted from .nvmrc" ok
else
  report "node $wanted from .nvmrc" "found ${actual:-none}, run nvm install"
fi

if [ "$missing" -gt 0 ]; then
  printf '\n%s tool(s) missing, see "Developer environment" in AGENTS.md\n' "$missing"
  exit 1
fi
printf '\nall tools are installed\n'
