#![cfg_attr(not(any(feature = "library", test)), no_std)]
#![cfg_attr(not(test), no_main)]

#[cfg(any(feature = "library", test))]
extern crate alloc;

#[cfg(not(any(feature = "library", test)))]
ckb_std::entry!(program_entry);
#[cfg(not(any(feature = "library", test)))]
ckb_std::default_alloc!(16384, 1258306, 64);

use ckb_hash::blake2b_256;
use ckb_std::{
    ckb_constants::Source,
    ckb_types::{
        bytes::Bytes,
        packed::WitnessArgs,
        prelude::*,
    },
    high_level::{load_script, load_witness},
};

// Error codes
const ERROR_INVALID_ARGS: i8 = 1;
const ERROR_LOAD_SCRIPT: i8 = 2;
const ERROR_LOAD_WITNESS: i8 = 3;
const ERROR_INVALID_WITNESS: i8 = 4;
const ERROR_MISSING_LOCK: i8 = 5;
const ERROR_HASH_MISMATCH: i8 = 6;

/// Entry point of the hash-preimage lock script.
///
/// Returns:
///     0  - Transaction accepted.
///     Non-zero - Transaction rejected.
pub fn program_entry() -> i8 {
    match verify() {
        Ok(()) => 0,
        Err(code) => code,
    }
}

/// Performs the hash-preimage verification.
fn verify() -> Result<(), i8> {
    // --------------------------------------------------
    // 1. Load the current script.
    // --------------------------------------------------

    let script = load_script().map_err(|_| ERROR_LOAD_SCRIPT)?;

    // --------------------------------------------------
    // 2. Extract and validate script arguments.
    //
    // The arguments must contain exactly 32 bytes:
    // the expected CKB Blake2b-256 hash.
    // --------------------------------------------------

    let args: Bytes = script.args().unpack();

    if args.len() != 32 {
        return Err(ERROR_INVALID_ARGS);
    }

    // --------------------------------------------------
    // 3. Load the witness at index 0 of the current
    //    script group.
    //
    // GroupInput ensures that the witness corresponds
    // to the first input carrying this exact lock script.
    // --------------------------------------------------

    let witness_data = load_witness(0, Source::GroupInput)
        .map_err(|_| ERROR_LOAD_WITNESS)?;

    // --------------------------------------------------
    // 4. Parse the witness as a Molecule WitnessArgs.
    //
    // Reject malformed or incorrectly encoded witnesses.
    // --------------------------------------------------

    let witness_args = WitnessArgs::from_slice(&witness_data)
        .map_err(|_| ERROR_INVALID_WITNESS)?;

    // --------------------------------------------------
    // 5. Extract the lock field.
    //
    // input_type and output_type are intentionally ignored.
    // --------------------------------------------------

    let lock_field = witness_args
        .lock()
        .to_opt()
        .ok_or(ERROR_MISSING_LOCK)?;

    // The lock field contains the supplied preimage.
    let preimage = lock_field.raw_data();

    // --------------------------------------------------
    // 6. Compute the CKB default Blake2b-256 hash.
    //
    // ckb_hash::blake2b_256 uses:
    //   Output size:     32 bytes
    //   Personalization: "ckb-default-hash"
    // --------------------------------------------------

    let computed_hash = blake2b_256(&preimage);

    // --------------------------------------------------
    // 7. Compare the computed hash with script args.
    //
    // Accept only when both 32-byte hashes match.
    // --------------------------------------------------

    if computed_hash.as_ref() != args.as_ref() {
        return Err(ERROR_HASH_MISMATCH);
    }

    // --------------------------------------------------
    // 8. Verification successful.
    //
    // One valid witness authorizes every input in this
    // script group.
    // --------------------------------------------------

    Ok(())
}
