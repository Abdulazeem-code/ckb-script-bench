#!/usr/bin/env bash
# Render the prompt given to a model for one task.
# The spec is inserted without its "## Tests" section and without the
# lines pointing at the reference solution and tests, so the model never
# sees the attack list.
#
# Usage: scripts/render_prompt.sh <task-dir> [template] [spec|brief]
#   spec  (default) - the full spec, which names the pitfalls to avoid
#   brief           - interface only (args/witness format); the model must
#                     work out the safety checks itself
set -euo pipefail

task_dir="$1"
template="${2:-prompts/baseline-v1.md}"
level="${3:-spec}"

spec=$(sed -e '/^## Tests/,$d' \
           -e '/^\*\*Reference solution:\*\*/d' \
           -e '/^\*\*Tests:\*\*/d' \
           "$task_dir/$level.md")

while IFS= read -r line; do
  if [[ "$line" == "{{SPEC}}" ]]; then
    printf '%s\n' "$spec"
  else
    printf '%s\n' "$line"
  fi
done < "$template"
