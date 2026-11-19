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

# Hash-preimage lock

Write a CKB lock script that can be unlocked by revealing the preimage of
a hash.

- The script args hold the hash: `blake2b_256` with CKB's default
  personalization (`"ckb-default-hash"`), 32 bytes.
- The spender puts the preimage in the `lock` field of `WitnessArgs`.
