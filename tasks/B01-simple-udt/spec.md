# B01: Simple fungible token (UDT)

**Tier:** B (amounts across several cells, ownership rule)
**Script kind:** type script
**Reference solution:** `contracts/simple-udt`
**Tests:** `tests/src/simple_udt.rs`

## What the script must do

Write a CKB type script for a simple fungible token. The token's issuer
(the *owner*) can mint and burn freely. Everyone else can only move or
burn tokens, never create them.

### Script args

- `args` must be **exactly 32 bytes**: the owner's lock script hash.
- Any other args length is invalid and the script must fail.

### Token amount

- A token cell is any cell whose type script is this exact script.
- Its amount is the **first 16 bytes** of the cell data, read as a `u128`,
  little-endian. Extra bytes after the first 16 are allowed and ignored.
- A token cell whose data is **shorter than 16 bytes** is invalid and the
  script must fail.

### Rules

1. **Owner mode.** If any **input** of the transaction (any input, not
   only token cells) has a lock script hash equal to `args`, the script
   succeeds. Only inputs count: the owner's lock appearing in an output
   or a cell dep proves nothing.
2. **Otherwise**, sum the amounts of token cells in the inputs
   (`Source::GroupInput`) and in the outputs (`Source::GroupOutput`). The
   script succeeds only if `output_sum <= input_sum`.
   - Only cells with this exact type script count. Cells of other tokens
     (same code, different args) must not be included.
   - Sums must not overflow. If adding amounts overflows a `u128`, the
     script must fail (never wrap around).

Exit with code 0 on success and a non-zero code otherwise.

## Out of scope

- Capacity and fees are not checked.
- Witnesses are not used.

## Tests

`owner` is a lock whose hash is the token's args; `alice` is any other
lock.

Positive (must pass):

| ID | Case |
|---|---|
| P1 | Transfer: alice input 100 → outputs 60 + 40 |
| P2 | Burn: input 100 → output 30 |
| P3 | Owner mint: owner-locked input (no tokens) → output 1000 |
| P4 | Merge 50 + 50 → 100, with a cell of a different token passing through unchanged |

Attack (must be rejected):

| ID | Attack | Catches scripts that... |
|---|---|---|
| A1 | Mint 100 with no owner input | don't enforce `output_sum <= input_sum` when there are no token inputs |
| A2 | Input 100 → output 101 | get the comparison wrong |
| A3 | Input 1 → outputs `u128::MAX` and 2 (wraps to 1) | add amounts without overflow checks |
| A4 | Input of a *different* token worth 1000 → output 1000 of this token | sum `Source::Input`/`Output` instead of the script group |
| A5 | Owner's lock only on an **output** → mint 1000 | look for the owner lock in outputs |
| A6 | Input 100 → outputs 100 + a token cell with 8-byte data | treat short data as amount 0 |
| A7 | Args are owner hash plus one byte, owner input present | only compare the first 32 bytes of args |
