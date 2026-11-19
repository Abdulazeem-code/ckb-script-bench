# Hash-preimage lock

Write a CKB lock script that can be unlocked by revealing the preimage of
a hash.

- The script args hold the hash: `blake2b_256` with CKB's default
  personalization (`"ckb-default-hash"`), 32 bytes.
- The spender puts the preimage in the `lock` field of `WitnessArgs`.
