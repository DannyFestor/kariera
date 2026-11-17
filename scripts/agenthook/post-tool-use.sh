#!/bin/sh
# Formats and lints right after an edit, so the agent sees a failure while the edit is fresh. lint-wip tolerates
# dead code, because the caller of code just written often does not exist yet; `make check` at the end does not.
set -eu
. "$(dirname "$0")/lib.sh"

path=$(edited_path)

case $path in
  /*) ;;
  *.rs) make_or_block fmt-rust FILES="$path" && make_or_block lint-wip ;;
  Cargo.toml | */Cargo.toml | clippy.toml | rust-toolchain.toml) make_or_block lint-wip ;;
  *.typ) make_or_block fmt-typst FILES="$path" ;;
  *.sh) make_or_block lint-shell FILES="$path" ;;
  *) ;;
esac || true
