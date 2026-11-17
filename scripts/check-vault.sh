#!/bin/sh
# Validates the real vault on this machine with `kariera validate`. Local only: no hook runs it, because the vault
# and its personal data exist on the owner's machines alone.
set -eu

fail() {
  echo "check-vault: $*" >&2
  exit 1
}

# kariera reads the one config under $HOME, so a config elsewhere would validate a different vault than it names.
main() {
  [ $# -eq 1 ] || fail "usage: $0 <config-file>"
  config=$1
  [ -f "$config" ] || fail "no config at $config. Create it, naming the vault path, before running this."
  [ "$config" = "$HOME/.config/kariera/config.toml" ] ||
    fail "kariera reads $HOME/.config/kariera/config.toml only, not $config"
  exec cargo run --quiet --locked --package kariera-cli -- validate
}

main "$@"
