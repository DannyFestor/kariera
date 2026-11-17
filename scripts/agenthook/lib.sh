# shellcheck shell=sh
# Shared by the Claude Code hooks. Each reads the hook's JSON from stdin once and answers with JSON on stdout.
# Exit status 2 would make Claude Code block on its own, so the hooks always exit 0 and a non-zero status only
# means a hook itself broke.

JQ=${JQ:-jq}
hook_input=$(cat)

input_field() {
  printf '%s' "$hook_input" | "$JQ" --raw-output "$1 // empty"
}

# Relative to the checkout root, which make -C made the working directory. Paths outside it stay absolute.
edited_path() {
  absolute_path=$(input_field '.tool_input.file_path // .tool_input.notebook_path')
  printf '%s' "${absolute_path#"$PWD"/}"
}

inserted_text() {
  input_field '.tool_input.new_string // .tool_input.content // .tool_input.new_source'
}

shell_command() {
  input_field '.tool_input.command'
}

# shellcheck disable=SC2016 # jq, not the shell, expands $decision and $reason.
permission() {
  "$JQ" --null-input --arg decision "$1" --arg reason "$2" \
    '{hookSpecificOutput: {hookEventName: "PreToolUse", permissionDecision: $decision, permissionDecisionReason: $reason}}'
}

deny() {
  permission deny "$1"
}

ask() {
  permission ask "$1"
}

# shellcheck disable=SC2016 # jq, not the shell, expands $reason.
block() {
  "$JQ" --null-input --arg reason "$1" '{decision: "block", reason: $reason}'
}

# Runs one Make target and blocks with its output when it fails. The hook itself runs under make, whose flags would
# otherwise leak into the target.
make_or_block() {
  if output=$(env -u MAKEFLAGS -u MFLAGS -u MAKELEVEL make -s --no-print-directory "$@" 2>&1); then
    return 0
  fi
  block "\`make $*\` failed. Fix every error below before going on.

$(printf '%s\n' "$output" | tail -n 200)"
  return 1
}
