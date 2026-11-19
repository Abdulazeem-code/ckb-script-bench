# A02: Absolute block-number timelock

**Tier:** A (single condition, no cross-cell logic)
**Script kind:** lock script
**Reference solution:** `contracts/absolute-timelock`
**Tests:** `tests/src/absolute_timelock.rs`

## What the script must do

Write a CKB lock script that keeps a cell unspendable until the chain
reaches a given block number. It uses the transaction input's `since`
field, which CKB consensus enforces against the real chain.

### Script args

- `args` must be **exactly 8 bytes**: an unsigned 64-bit integer `N`,
  little-endian. `N` is the earliest block number at which the cell may be
  spent.
- `N` must be at least 1. `N == 0` is invalid.
- Any other args length is invalid.

In all invalid cases the script must exit with a non-zero code.

### Unlocking

For **every** input in this script's group (`Source::GroupInput`), read its
`since` value and require all of the following:

1. The top byte (bits 56–63) is `0x00`. That means:
   - bit 63 = 0: absolute, not relative;
   - bits 61–62 = `00`: the metric is **block number** (not epoch, not
     timestamp);
   - bits 56–60 = 0: reserved bits are clear.
2. The remaining 56-bit value is `>= N`.

If every input in the group satisfies this, exit with code 0. Otherwise,
exit with a non-zero code.

No witness is needed.

### `since` encoding reference

```
bit 63      : 1 = relative, 0 = absolute
bits 61..62 : 00 = block number, 01 = epoch, 10 = timestamp
bits 56..60 : reserved, must be 0
bits 0..55  : value
```

See [RFC 0017](https://github.com/nervosnetwork/rfcs/blob/master/rfcs/0017-tx-valid-since/0017-tx-valid-since.md).

## Out of scope

- The script does not need to check the chain tip itself; consensus
  rejects a transaction whose `since` hasn't been reached yet.
- Outputs are not checked.

## Tests

`N = 1000` in all tests unless noted.

Positive (must pass):

| ID | Case |
|---|---|
| P1 | `since` = absolute block number `N` exactly |
| P2 | `since` = absolute block number `N + 500` |
| P3 | Two inputs with the same lock, both with valid `since` |

Attack (must be rejected):

| ID | Attack | Catches scripts that... |
|---|---|---|
| A1 | `since = 0` (no time lock at all) | skip the check when `since` is 0 |
| A2 | Absolute block number `N - 1` | use `>` / `<` the wrong way round, or are off by one |
| A3 | **Relative** block number `N` (bit 63 set) | ignore the relative flag and only compare the value |
| A4 | Absolute **timestamp** with a large value | ignore the metric bits |
| A5 | Absolute **epoch** with a large value | ignore the metric bits |
| A6 | Two inputs: first valid, second `since = 0` | only check the first input in the group |
| A7 | Args are 9 bytes (`N` plus one extra byte) | read the first 8 bytes and ignore the rest |
