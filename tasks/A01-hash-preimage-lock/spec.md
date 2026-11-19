# A01: Hash-preimage lock

**Tier:** A (single condition, no cross-cell logic)
**Script kind:** lock script
**Reference solution:** `contracts/hash-preimage-lock`
**Tests:** `tests/src/hash_preimage_lock.rs`

## What the script must do

Write a CKB lock script that lets a cell be spent by anyone who reveals a
secret value (the *preimage*) whose hash matches a hash stored in the
script's args.

### Script args

- `args` must be **exactly 32 bytes**: the hash `H`.
- `H = blake2b_256(preimage)`, using CKB's default blake2b
  (32-byte output, personalization `"ckb-default-hash"`).
- Any other args length is invalid and the script must fail.

### Unlocking

- The preimage is supplied in the `lock` field of a `WitnessArgs`
  structure (molecule-encoded).
- The witness used is the one at the **same index as the first input in
  this script's group** (load it with `Source::GroupInput`, index 0).
  Witnesses at other indexes must be ignored.
- The script succeeds (exit code 0) only if:
  1. the witness exists and parses as `WitnessArgs`,
  2. its `lock` field is present, and
  3. `blake2b_256(lock) == H`.
- In every other case it must exit with a non-zero code.

One valid witness unlocks every input in the group (all inputs that carry
this exact lock script).

## Out of scope

- Values in `input_type` / `output_type` are ignored.
- Outputs are not checked.

## Known limitation (by design)

Once the preimage is revealed in a pending transaction, anyone who sees it
can build their own transaction spending the same cell. That makes this
lock unsafe for real funds. It is used here because it is the simplest
lock that still has several ways to get wrong.

## Tests

Positive (must pass):

| ID | Case |
|---|---|
| P1 | One input, correct preimage in `WitnessArgs.lock` |
| P2 | Two inputs with the same lock, one witness at the group's first index |
| P3 | Lock input at index 1 (after an unrelated input), witness at index 1 |

Attack (must be rejected):

| ID | Attack | Catches scripts that... |
|---|---|---|
| A1 | Wrong preimage | don't actually compare the hash |
| A2 | No witnesses at all | treat a missing witness as success |
| A3 | Preimage in `input_type`, `lock` empty | read any `WitnessArgs` field |
| A4 | Witness `lock` holds `H` itself, not the preimage | compare `lock == args` instead of hashing |
| A5 | Args are `H` plus one extra byte (33 bytes) | only compare the first 32 bytes of args |
| A6 | Valid preimage at witness 0, lock input at index 1 with an empty witness | read witness by absolute index (`Source::Input`) instead of group index |
