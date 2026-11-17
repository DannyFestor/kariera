#!/usr/bin/env bash
# PROTOTYPE, throwaway: renders every layout variant to tmp/prototype/<name>/ as PDF and PNG.
#
#   crates/render-typst/prototype/shokumu-keirekisho/render.sh               # the fictional fixture vault
#   crates/render-typst/prototype/shokumu-keirekisho/render.sh <vault> real  # a private preview, never committed
#
# Needs the Typst CLI at the version the renderer will embed, installed into tmp/ (see README.md).
set -euo pipefail

repo=$(git rev-parse --show-toplevel)
here=crates/render-typst/prototype/shokumu-keirekisho
vault=${1:-fixtures/vault}
name=${2:-fixture}
out=tmp/prototype/$name
typst=$repo/tmp/typst-cli/bin/typst

cd "$repo"
mkdir -p "$out"
cargo run --quiet -p kariera-cli --example prototype_shokumu_keirekisho_data -- "$vault" "$out/data.json"

for template in "$here"/variant-*.typ "$here"/rirekisho-*.typ; do
  [[ -e $template ]] || continue
  variant=$(basename "$template" .typ)
  for era in ${ERAS:-seireki}; do
    target=$out/$variant-$era
    "$typst" compile --root "$repo" --ignore-system-fonts --font-path tmp/fonts \
      --input "data=/$out/data.json" --input "today=$(date +%F)" --input "era=$era" \
      "$template" "$target.pdf"
    rm -f "$target"-*.png
    "$typst" compile --root "$repo" --ignore-system-fonts --font-path tmp/fonts \
      --input "data=/$out/data.json" --input "today=$(date +%F)" --input "era=$era" \
      --format png --ppi 110 "$template" "$target-{p}.png"
    echo "$repo/$target.pdf"
  done
done
