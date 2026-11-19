# Block-number timelock

Write a CKB lock script that keeps a cell locked until the chain reaches a
given block number.

- The script args hold the unlock block number as a `u64`, 8 bytes,
  little-endian.
- Use the transaction input's `since` field.
