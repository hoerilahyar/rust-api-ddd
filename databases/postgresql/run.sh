#!/usr/bin/env bash
# Simple runner for environments without golang-migrate / node-pg-migrate installed.
# Usage:
#   ./run.sh migrate   # apply all *.up.sql in order
#   ./run.sh rollback   # apply all *.down.sql in reverse order (danger: drops data)
#   ./run.sh seed        # apply all seeders in order
#
# Requires: psql on PATH, and DATABASE_URL env var set, e.g.:
#   export DATABASE_URL="postgres://user:password@localhost:5432/mydb"

set -euo pipefail

DATABASE_URL="postgres://postgres:postgres@localhost:5432/mydb"

DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

migrate() {
  for f in "$DIR"/migrations/*.up.sql; do
    echo ">> applying $(basename "$f")"
    psql "$DATABASE_URL" -v ON_ERROR_STOP=1 -f "$f"
  done
}

rollback() {
  for f in $(ls "$DIR"/migrations/*.down.sql | sort -r); do
    echo ">> rolling back $(basename "$f")"
    psql "$DATABASE_URL" -v ON_ERROR_STOP=1 -f "$f"
  done
}

seed() {
  for f in "$DIR"/seeders/*.sql; do
    echo ">> seeding $(basename "$f")"
    psql "$DATABASE_URL" -v ON_ERROR_STOP=1 -f "$f"
  done
}

case "${1:-}" in
  migrate) migrate ;;
  rollback) rollback ;;
  seed) seed ;;
  *) echo "Usage: $0 {migrate|rollback|seed}"; exit 1 ;;
esac
