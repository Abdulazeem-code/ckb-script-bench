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

# A01: Hash-preimage lock

**Tier:** A (single condition, no cross-cell logic)
**Script kind:** lock script

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
