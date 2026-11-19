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
    ckb_types::prelude::*,
    high_level::{load_script, load_witness_args},
};

/// Error codes.
const ERROR_INVALID_ARGS: i8 = 1;
const ERROR_INVALID_WITNESS: i8 = 2;
const ERROR_HASH_MISMATCH: i8 = 3;

/// Hash-preimage lock script.
///
/// Script arguments:
///   32-byte CKB-personalized Blake2b-256 hash.
///
/// Witness:
///   WitnessArgs.lock contains the preimage.
pub fn program_entry() -> i8 {
    // Load the currently executing script.
    let script = match load_script() {
        Ok(script) => script,
        Err(_) => return ERROR_INVALID_ARGS,
    };

    // Extract the expected hash from script arguments.
    let args: alloc::vec::Vec<u8> = script.args().raw_data().to_vec();

    // A valid CKB Blake2b-256 hash must be exactly 32 bytes.
    if args.len() != 32 {
        return ERROR_INVALID_ARGS;
    }

    // Load the first witness belonging to this lock-script group.
    let witness_args = match load_witness_args(0, Source::GroupInput) {
        Ok(witness) => witness,
        Err(_) => return ERROR_INVALID_WITNESS,
    };

    // Extract the lock field containing the preimage.
    let lock_field = match witness_args.lock().to_opt() {
        Some(lock) => lock,
        None => return ERROR_INVALID_WITNESS,
    };

    let preimage: alloc::vec::Vec<u8> = lock_field.raw_data().to_vec();

    // Calculate CKB's default personalized Blake2b-256 hash.
    let computed_hash = blake2b_256(&preimage);

    // Compare the computed hash against the expected hash.
    if computed_hash.as_ref() != args.as_slice() {
        return ERROR_HASH_MISMATCH;
    }

    // The preimage is valid.
    0
}
