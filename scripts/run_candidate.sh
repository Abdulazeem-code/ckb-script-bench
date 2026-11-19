#!/usr/bin/env bash
# Build a model-written script and run it through a task's tests.
#
# Usage: scripts/run_candidate.sh <crate-name> <candidate-dir>
#   <crate-name>     e.g. hash-preimage-lock (same name as the reference crate)
#   <candidate-dir>  folder holding the model's answer:
#                      main.rs    - the complete src/main.rs
#                      deps.toml  - the [dependencies] section (optional)
#
# Writes into <candidate-dir>:
#   build.log      compiler output
#   bin/<crate>    the compiled script (if it built)
#   results.txt    full test output
#   summary.txt    positive/attack counts and which attacks got through
set -uo pipefail

crate="$1"
cand="$(cd "$2" && pwd)"
repo="$(cd "$(dirname "$0")/.." && pwd)"
module="${crate//-/_}"

[ -f "$cand/main.rs" ] || { echo "missing $cand/main.rs"; exit 1; }

# Build in a throwaway copy so the reference solution is never touched.
work="$(mktemp -d)"
trap 'rm -rf "$work"' EXIT
rsync -a --exclude target --exclude build --exclude runs --exclude .git "$repo/" "$work/"
cp "$cand/main.rs" "$work/contracts/$crate/src/main.rs"
if [ -f "$cand/deps.toml" ]; then
  manifest="$work/contracts/$crate/Cargo.toml"
  # Replace the [dependencies] section with the model's version.
  awk -v deps="$cand/deps.toml" '
    /^\[dependencies\]/ { while ((getline l < deps) > 0) print l; skip=1; next }
    /^\[/ { skip=0 }
    !skip { print }
  ' "$manifest" > "$manifest.new" && mv "$manifest.new" "$manifest"
  grep -q '^\[dependencies\]' "$manifest" || sed -i '0,/^\[features\]/s//[dependencies]\nckb-std = "1.1"\n\n[features]/' "$manifest"
fi

# Share compiled dependencies between runs (the Makefile expects ./target).
cache="${HOME}/.cache/ckb-script-bench/target"
mkdir -p "$cache" && ln -s "$cache" "$work/target"
(cd "$work" && make build CONTRACT="$crate") > "$cand/build.log" 2>&1
# (A failing native-simulator step after the copy is expected and harmless.)
if [ ! -f "$work/build/release/$crate" ] || ! grep -q "Copying binary $crate" "$cand/build.log"; then
  echo "result: COMPILE ERROR (see $cand/build.log)" | tee "$cand/summary.txt"
  exit 2
fi
mkdir -p "$cand/bin"
cp "$work/build/release/$crate" "$cand/bin/$crate"

(cd "$repo" && BENCH_BIN_DIR="$cand/bin" cargo test -p tests "$module::" -- --test-threads=1) > "$cand/results.txt" 2>&1

pos_total=$(grep -cE "^test $module::p[0-9]" "$cand/results.txt")
pos_ok=$(grep -cE "^test $module::p[0-9].* ok$" "$cand/results.txt")
atk_total=$(grep -cE "^test $module::a[0-9]" "$cand/results.txt")
atk_ok=$(grep -cE "^test $module::a[0-9].* ok$" "$cand/results.txt")
{
  echo "positive tests passed: $pos_ok/$pos_total"
  echo "attacks rejected:      $atk_ok/$atk_total"
  failed_pos=$(grep -E "^test $module::p[0-9].* FAILED$" "$cand/results.txt" | sed -E "s/^test $module::([^ ]+).*/\1/")
  failed_atk=$(grep -E "^test $module::a[0-9].* FAILED$" "$cand/results.txt" | sed -E "s/^test $module::([^ ]+).*/\1/")
  [ -n "$failed_pos" ] && echo "broken (valid tx rejected): $(echo $failed_pos)"
  [ -n "$failed_atk" ] && echo "UNSAFE (attack accepted):   $(echo $failed_atk)"
  if [ "$pos_ok" = "$pos_total" ] && [ "$atk_ok" != "$atk_total" ]; then
    echo "verdict: WORKS BUT UNSAFE"
  elif [ "$pos_ok" = "$pos_total" ]; then
    echo "verdict: CORRECT"
  else
    echo "verdict: BROKEN"
  fi
} | tee "$cand/summary.txt"
