#!/bin/sh
# Writes the `sha256sum` lines of a GitHub release's assets from the digests GitHub records, so releases with and
# without a checksum file of their own are pinned the same way. JQ names the jq binary to use.
set -eu

fail() {
  echo "pin-release-assets: $*" >&2
  exit 1
}

release_assets() {
  gh api "repos/$1/releases/tags/$2"
}

# shellcheck disable=SC2016 # jq, not the shell, expands $pattern.
# The pattern has to match the whole asset name, so archives never pull in their .sha256 or .sig siblings.
checksum_lines() {
  "${JQ:-jq}" --raw-output --arg pattern "$1" '
    .assets[]
    | select(.name | test("^(" + $pattern + ")$"))
    | "\(.digest | ltrimstr("sha256:"))  \(.name)"'
}

main() {
  [ $# -eq 3 ] || fail "usage: $0 <owner/repo> <tag> <asset-name-regex>"
  lines=$(release_assets "$1" "$2" | checksum_lines "$3")
  [ -n "$lines" ] || fail "no asset of $1 $2 matches $3"
  echo "$lines"
}

main "$@"
