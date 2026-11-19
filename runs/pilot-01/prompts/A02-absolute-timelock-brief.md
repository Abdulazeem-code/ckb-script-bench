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

# Block-number timelock

Write a CKB lock script that keeps a cell locked until the chain reaches a
given block number.

- The script args hold the unlock block number as a `u64`, 8 bytes,
  little-endian.
- Use the transaction input's `since` field.
