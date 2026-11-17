#!/usr/bin/env bash
# PROTOTYPE, throwaway (branch prototype/cv): renders every CV layout variant in English and German, as PDF and as
# PNG pages, into tmp/cv/<source>/. Typst 0.15.1, the version the renderer will embed (issue #7), and the candidate
# fonts are fetched into tmp/ on the first run.
#
#   crates/render-typst/prototype/cv/render.sh            # the fixture vault
#   crates/render-typst/prototype/cv/render.sh real       # the vault in ~/.config/kariera/config.toml, private, tmp/ only
set -euo pipefail

source_name="${1:-fixture}"
repo="$(git rev-parse --show-toplevel)"
prototype="crates/render-typst/prototype/cv"
typst="$repo/tmp/typst-0.15.1/typst"
fonts="$repo/tmp/fonts"
out="$repo/tmp/cv/$source_name"
build_month="$(date +%Y-%m)"

case "$source_name" in
  fixture) vault="$repo/fixtures/vault" ;;
  real) vault="$(sed -n 's/^vault = "\(.*\)"$/\1/p' "$HOME/.config/kariera/config.toml")"; vault="${vault/#\~/$HOME}" ;;
  *) echo "usage: $0 [fixture|real]" >&2; exit 2 ;;
esac

fetch_typst() {
  [[ -x "$typst" ]] && return
  mkdir -p "$(dirname "$typst")"
  curl -fsSL "https://github.com/typst/typst/releases/download/v0.15.1/typst-aarch64-apple-darwin.tar.xz" \
    | tar -xJ -C "$(dirname "$typst")" --strip-components=1 typst-aarch64-apple-darwin/typst
}

fetch_fonts() {
  [[ -d "$fonts" ]] && return
  local download
  download="$(mktemp -d)"
  mkdir -p "$fonts"
  curl -fsSL -o "$download/inter.zip" https://github.com/rsms/inter/releases/download/v4.1/Inter-4.1.zip
  unzip -jqo "$download/inter.zip" 'extras/otf/Inter-Regular.otf' 'extras/otf/Inter-Italic.otf' \
    'extras/otf/Inter-Medium.otf' 'extras/otf/Inter-SemiBold.otf' 'extras/otf/Inter-Bold.otf' -d "$fonts"
  local plex="https://github.com/IBM/plex/releases/download"
  curl -fsSL -o "$download/plex-sans.zip" "$plex/%40ibm%2Fplex-sans%401.1.0/ibm-plex-sans.zip"
  curl -fsSL -o "$download/plex-serif.zip" "$plex/%40ibm%2Fplex-serif%402.0.0/ibm-plex-serif.zip"
  local weight
  for weight in Regular Italic Medium SemiBold Bold Light; do
    unzip -jqo "$download/plex-sans.zip" "ibm-plex-sans/fonts/complete/otf/IBMPlexSans-$weight.otf" -d "$fonts"
    unzip -jqo "$download/plex-serif.zip" "ibm-plex-serif/fonts/complete/otf/IBMPlexSerif-$weight.otf" -d "$fonts"
  done
  for weight in Regular Medium SemiBold Bold Light; do
    curl -fsSL -o "$fonts/IBMPlexSansJP-$weight.ttf" \
      "https://raw.githubusercontent.com/google/fonts/main/ofl/ibmplexsansjp/IBMPlexSansJP-$weight.ttf"
  done
  local noto="https://raw.githubusercontent.com/notofonts/noto-cjk/main"
  for weight in Regular Medium Bold; do
    curl -fsSL -o "$fonts/NotoSansJP-$weight.otf" "$noto/Sans/SubsetOTF/JP/NotoSansJP-$weight.otf"
    curl -fsSL -o "$fonts/NotoSerifJP-$weight.otf" "$noto/Serif/SubsetOTF/JP/NotoSerifJP-$weight.otf"
  done
  rm -rf "$download"
}

fetch_typst
fetch_fonts
mkdir -p "$out"
(cd "$repo" && cargo run --quiet -p kariera-cli --example prototype_cv_data -- "$vault" "$out/data" "$build_month")

for variant in a b c; do
  [[ -f "$repo/$prototype/variant-$variant.typ" ]] || continue
  for lang in en de; do
    pdf="$out/cv-$variant-$lang.pdf"
    "$typst" compile --root "$repo" --ignore-system-fonts --font-path "$fonts" \
      --input "data=/${out#"$repo"/}/data/cv-$lang.json" \
      --input "personal=/$prototype/personal-fixture.yaml" \
      "$repo/$prototype/variant-$variant.typ" "$pdf"
    rm -f "$out/cv-$variant-$lang"-*.png
    pdftoppm -png -r 110 "$pdf" "$out/cv-$variant-$lang"
  done
done
echo "$out"
