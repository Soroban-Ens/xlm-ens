#!/usr/bin/env bash
set -euo pipefail

RPC_URL="${SOROBAN_RPC_URL:-https://soroban-testnet.stellar.org}"

jsonrpc() {
  local method="$1"
  curl -fsS -H 'content-type: application/json' \
    --data "{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"${method}\"}" \
    "${RPC_URL}"
}

echo "Canary: RPC health check"
echo "  SOROBAN_RPC_URL=${RPC_URL}"

# Non-destructive endpoint checks to catch integration drift early.
jsonrpc getHealth >/dev/null
jsonrpc getNetwork >/dev/null

echo "OK"

