#!/bin/sh
# Judges a tool call before it runs: denies what breaks a hard rule of AGENTS.md and asks the human before an edit
# to the checks themselves. The command rules match text, so they catch the usual spellings, not every one.
set -eu
. "$(dirname "$0")/lib.sh"

ALLOW_ATTRIBUTE='#!?\[allow\('

judge_edit() {
  path=$(edited_path)
  if inserted_text | grep -Eq -e "$ALLOW_ATTRIBUTE"; then
    deny "#[allow] is denied. Fix the code, or use #[expect(lint, reason = \"…\")] with a real reason (AGENTS.md)."
    return
  fi
  judge_edited_path "$path"
}

judge_edited_path() {
  case $1 in
    /*) ;;
    Cargo.lock)
      deny "Cargo.lock is written by cargo. Change a Cargo.toml and let cargo update the lock file." ;;
    Cargo.toml | */Cargo.toml)
      ask "$1 holds dependencies and lints. A new dependency needs a deny.toml entry with its reason (AGENTS.md)." ;;
    Makefile | lefthook.yml | deny.toml | crate-boundaries.toml | clippy.toml | rustfmt.toml | rust-toolchain.toml | \
      AGENTS.md | .claude/settings.json | scripts/* | tools/* | docs/adr/*)
      ask "$1 is a guardrail file: it configures the checks the agent works under." ;;
    *) ;;
  esac
}

SKIPS_HOOKS='--no-verify|LEFTHOOK=0|LEFTHOOK_EXCLUDE|core\.hooksPath|git commit.*[[:space:]]-[a-zA-Z]*n[a-zA-Z]*([[:space:]]|$)'
CHANGES_DEPENDENCIES='cargo (add|remove)'
DISCARDS_WORK='git (reset --hard|clean -|checkout -- |stash (drop|clear)|branch -D|push (-f|--force))'

command_matches() {
  shell_command | grep -Eq -e "$1"
}

judge_command() {
  if command_matches "$SKIPS_HOOKS"; then
    deny "The git hooks are this project's checks and are never skipped."
  elif command_matches "$CHANGES_DEPENDENCIES"; then
    ask "This changes the dependencies. A new one needs a deny.toml entry with its reason (AGENTS.md)."
  elif command_matches "$DISCARDS_WORK"; then
    ask "This command discards work or rewrites history."
  fi
}

case $(input_field .tool_name) in
  Bash) judge_command ;;
  Edit | Write | NotebookEdit) judge_edit ;;
  *) ;;
esac
