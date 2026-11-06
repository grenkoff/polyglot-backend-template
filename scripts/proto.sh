#!/usr/bin/env bash

set -o errexit
set -o nounset
set -o pipefail

root="$(cd "$(dirname "$0")/.." && pwd)"
out="$root/services/core/server/contracts"

rm -rf "$out"
mkdir -p "$out"

uvx --from 'grpcio-tools==1.85.*' --with 'mypy-protobuf==5.*' \
  python -m grpc_tools.protoc \
  --proto_path="$root/proto" \
  --python_out="$out" \
  --mypy_out="$out" \
  --grpc_python_out="$out" \
  --mypy_grpc_out="$out" \
  $(cd "$root/proto" && find . -name '*.proto' -printf '%P\n' | sort)

find "$out" -name '*.py' -o -name '*.pyi' | while read -r file; do
  sed -i -E 's/^from ([a-z_]+)\.(v[0-9]+) import/from server.contracts.\1.\2 import/; s/^import ([a-z_]+)\.(v[0-9]+)\.([a-z_0-9]+)$/import server.contracts.\1.\2.\3/' "$file"
done

find "$out" -type d -exec sh -c 'touch "$1/__init__.py"' _ {} \;

echo "generated: $(find "$out" -name '*_pb2*.py*' | wc -l) files in ${out#"$root"/}"
