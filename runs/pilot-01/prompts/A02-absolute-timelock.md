Write a CKB on-chain lock script in Rust that implements the specification below.

Environment:
- The crate was generated from cryptape/ckb-script-templates. It builds for
  `riscv64imac-unknown-none-elf` with `no_std`.
- `ckb-std = "1.1"` (default features) is already a dependency. You may add
  other crates if you need them; list them in your answer.
- `src/main.rs` must start with exactly this boilerplate:

```rust
#![cfg_attr(not(any(feature = "library", test)), no_std)]
#![cfg_attr(not(test), no_main)]

#[cfg(any(feature = "library", test))]
extern crate alloc;

#[cfg(not(any(feature = "library", test)))]
ckb_std::entry!(program_entry);
#[cfg(not(any(feature = "library", test)))]
ckb_std::default_alloc!(16384, 1258306, 64);
```

- It must define `pub fn program_entry() -> i8`, returning `0` when the
  script accepts the transaction and a non-zero value when it rejects it.

Reply with exactly two code blocks:
1. The `[dependencies]` section of `Cargo.toml` (```toml)
2. The complete `src/main.rs` (```rust)

---

# A02: Absolute block-number timelock

**Tier:** A (single condition, no cross-cell logic)
**Script kind:** lock script

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
