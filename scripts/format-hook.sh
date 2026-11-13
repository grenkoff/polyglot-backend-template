#!/usr/bin/env bash

set -o nounset
set -o pipefail

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
ruff="$root/services/core/.venv/bin/ruff"

case "$(uname -s)" in
  MINGW* | MSYS* | CYGWIN*)
    if [[ "$root" == //wsl* ]]; then
      exit 0
    fi
    ;;
esac

file="$(node -e '
  let input = "";
  process.stdin.on("data", (chunk) => { input += chunk; });
  process.stdin.on("end", () => {
    console.log(JSON.parse(input).tool_input?.file_path ?? "");
  });
' 2>/dev/null)"

case "$file" in
  *.py)
    if [[ -x "$ruff" ]]; then
      "$ruff" format --quiet --force-exclude "$file"
      "$ruff" check --fix --quiet --exit-zero --force-exclude "$file"
    fi
    ;;
  *.rs)
    rustfmt --quiet "$file" 2>/dev/null
    ;;
esac
exit 0
