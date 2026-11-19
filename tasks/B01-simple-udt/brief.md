# Simple fungible token (UDT)

Write a CKB type script for a simple fungible token (UDT).

- The script args hold the owner's lock script hash, 32 bytes.
- A token cell's amount is the first 16 bytes of its cell data, as a
  `u128`, little-endian.
- The owner can mint new tokens. Everyone else can only transfer or burn
  tokens they hold.
