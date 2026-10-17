#!/bin/sh
# Stop hook: when `statute check` reports stale or missing links, asks the agent to run the `change`
# skill before it ends the turn. It asks once per turn, never reviews on its own, and stays quiet
# outside a Statute repository or without the CLI.

input=$(cat)
# The agent is already continuing because of this hook: let it stop.
case "$input" in
  *'"stop_hook_active":true'* | *'"stop_hook_active": true'*) exit 0 ;;
esac

command -v statute >/dev/null 2>&1 || exit 0
root=$(git rev-parse --show-toplevel 2>/dev/null) || exit 0
[ -f "$root/statute.yml" ] || exit 0

report=$(cd "$root" && statute check --json 2>/dev/null)
printf '%s' "$report" | grep -Eq '"code": *"link\.(stale|missing)"' || exit 0

printf '%s\n' '{"decision": "block", "reason": "statute check reports stale or missing links. Run the Statute `change` skill: review each affected rule and record the verdict with `statute review` before ending the turn."}'
