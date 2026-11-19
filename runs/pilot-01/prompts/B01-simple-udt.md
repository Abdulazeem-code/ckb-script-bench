Write a CKB on-chain script in Rust that implements the specification below.

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

# B01: Simple fungible token (UDT)

**Tier:** B (amounts across several cells, ownership rule)
**Script kind:** type script

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
