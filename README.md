# ckb-script-bench

A benchmark that measures whether AI-written CKB scripts are **safe**, not
just whether they work. Each task has a plain-English spec, a hand-written
reference solution, and two kinds of tests:

- **Positive tests:** valid transactions the script must accept.
- **Attack tests:** malicious transactions the script must reject.

A script that passes every positive test but accepts an attack is
"working but unsafe", which is the failure this bench exists to find.

## Tasks

| ID | Task | Positive | Attack | Status |
|---|---|---|---|---|
| A01 | [Hash-preimage lock](tasks/A01-hash-preimage-lock/spec.md) | 3 | 6 | Done |
| A02 | [Absolute block-number timelock](tasks/A02-absolute-timelock/spec.md) | 3 | 7 | Done |
| B01 | [Simple fungible token (UDT)](tasks/B01-simple-udt/spec.md) | 4 | 7 | Done |

Tier A: one condition, one cell type. Tier B: amounts across several
cells and ownership rules.

## Layout

```
tasks/<id>/spec.md        full spec: names every rule, including the safety checks
tasks/<id>/brief.md       interface only: the way a developer would actually ask
contracts/<name>/         reference solution
tests/src/<name>.rs       positive + attack tests (ckb-testtool)
prompts/                  fixed prompt templates given to models
runs/<run>/               model answers, prompts used, and results
```

Each task is given to models at two levels: the **full spec** tests
whether a model can follow explicit rules; the **brief** tests whether
it knows which safety checks matter without being told.

## Running

Requirements: Rust with the `riscv64imac-unknown-none-elf` target and
Clang 18+. See [ckb-script-templates](https://github.com/cryptape/ckb-script-templates).

```bash
make build   # build reference solutions into build/release/
make test    # run all tests against them
```

### Testing a model's answer

Render the prompt, give it to a model, and save its answer as `main.rs`
(plus its `[dependencies]` section as `deps.toml`) in a folder. Then:

```bash
scripts/render_prompt.sh tasks/B01-simple-udt prompts/baseline-v2.md brief
scripts/run_candidate.sh simple-udt runs/<run>/<model>/<task>
```

The runner compiles the answer in a throwaway copy of the repo, runs the
task's tests, and writes `summary.txt` with one of three verdicts:

- **CORRECT:** all positive tests pass, all attacks rejected
- **WORKS BUT UNSAFE:** all positive tests pass, at least one attack accepted
- **BROKEN:** a valid transaction was rejected

To test an already-compiled binary directly, point `BENCH_BIN_DIR` at its
folder:

```bash
BENCH_BIN_DIR=/path/to/bin cargo test -p tests simple_udt::
```

## Checking the tests themselves

Each attack test is checked against a deliberately broken copy of the
reference solution containing the bug it targets. For example, a timelock
that only checks the first input in the group must fail `a6` and nothing
else. All 12 broken copies across A01, A02 and B01 are caught by exactly
the tests that target their bug.

## Results so far

[Pilot run 01](runs/pilot-01/README.md), 6 answers from ChatGPT and
Gemini:

- **Tier A** (A01, A02): all correct and safe, from both the full spec
  and the brief.
- **Tier B** (B01 UDT, brief): neither answer compiled against ckb-std
  1.1. After the smallest compile fix, both were correct and safe.

No unsafe scripts so far. Every failure was outdated or invented API
usage.

[Run 02](runs/run-02-agents/README.md): the same B01 brief prompt with
CKB's official `AGENTS.md` guide added. ChatGPT's answer now compiles
and is correct; Gemini Pro repeats the same compile error (a `SysError`
match written for older ckb-std). The guide fixes "which API" mistakes
but not "which version" mistakes.

## Notes

- `scripts/find_clang` was changed to also look for Clang 18, the
  default on Ubuntu 24.04. The upstream template only searches 19+.

*This project was bootstrapped with [ckb-script-templates].*

[ckb-script-templates]: https://github.com/nervosnetwork/ckb-script-templates
