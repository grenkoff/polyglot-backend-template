#!/usr/bin/env bash

set -o errexit
set -o nounset
set -o pipefail

name="e2e-$RANDOM"
url="${CORE_URL:-http://localhost:8000}/api/v1/greetings/"

echo "POST $url"
response="$(curl -fsS -X POST "$url" \
  -H 'Content-Type: application/json' \
  -d "{\"name\": \"$name\", \"polite\": true}")"
echo "$response"

expected="Hello, $name! Nice to meet you."
if [[ "$response" != *"$expected"* ]]; then
  echo "FAIL: core did not return the greeting from hello"
  exit 1
fi

for _ in $(seq 1 20); do
  if docker compose logs core-events 2>/dev/null | grep -q "$name"; then
    echo "OK: the greeting went through core -> hello -> nats -> core-events"
    exit 0
  fi
  sleep 1
done

echo "FAIL: core-events did not receive the hello.v1.greeted event"
docker compose logs core-events | tail -20
exit 1
