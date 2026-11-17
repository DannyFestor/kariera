#!/bin/sh
# Holds the end of the agent's turn until `make check` passes. A clean working tree has nothing new to check.
set -eu
. "$(dirname "$0")/lib.sh"

if [ -z "$(git status --porcelain)" ]; then
  exit 0
fi

make_or_block check || true
