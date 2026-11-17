#!/bin/sh
# Smoke-tests kariera against the real vault on this machine. Local only: no hook runs it, because the vault and its
# personal data exist on the owner's machines alone.
set -eu

fail() {
  echo "check-vault: $*" >&2
  exit 1
}

main() {
  [ $# -eq 1 ] || fail "usage: $0 <config-file>"
  config=$1
  [ -f "$config" ] || fail "no config at $config. Create it, naming the vault path, before running this."
  echo "check-vault: found $config. kariera has no validate command yet, so there is nothing more to run."
}

main "$@"
