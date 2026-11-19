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

## Layout

```
tasks/<id>/spec.md        what the script must do (this is the prompt given to a model)
contracts/<name>/         reference solution
tests/src/<name>.rs       positive + attack tests (ckb-testtool)
```

## Running

Requirements: Rust with the `riscv64imac-unknown-none-elf` target and
Clang 18+. See [ckb-script-templates](https://github.com/cryptape/ckb-script-templates).

```bash
make build   # build reference solutions into build/release/
make test    # run all tests against them
```

### Testing another script (e.g. AI-generated)

Put the compiled binary in a folder, using the same file name as the
reference (for example `hash-preimage-lock`), and point `BENCH_BIN_DIR`
at it:

```bash
BENCH_BIN_DIR=/path/to/candidate cargo test -p tests hash_preimage_lock::
```

Failed `p*` tests mean the script doesn't work. Failed `a*` tests mean it
accepted an attack.

## Checking the tests themselves

Each attack test is checked against a deliberately broken copy of the
reference solution containing the bug it targets. For example, a timelock
that only checks the first input in the group must fail `a6` and nothing
else. All broken copies for A01 and A02 are caught by exactly the tests
that target their bug.

## Notes

- `scripts/find_clang` was changed to also look for Clang 18, the
  default on Ubuntu 24.04. The upstream template only searches 19+.

*This project was bootstrapped with [ckb-script-templates].*

[ckb-script-templates]: https://github.com/nervosnetwork/ckb-script-templates
