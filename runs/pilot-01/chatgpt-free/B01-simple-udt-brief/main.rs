#![cfg_attr(not(any(feature = "library", test)), no_std)]
#![cfg_attr(not(test), no_main)]

#[cfg(any(feature = "library", test))]
extern crate alloc;

#[cfg(not(any(feature = "library", test)))]
ckb_std::entry!(program_entry);
#[cfg(not(any(feature = "library", test)))]
ckb_std::default_alloc!(16384, 1258306, 64);

use alloc::vec::Vec;
use core::convert::TryInto;

use ckb_std::{
    ckb_constants::Source,
    error::SysError,
    high_level::{
        load_cell_data,
        load_cell_lock,
        load_cell_type,
        load_script,
    },
};

/// Read the token amount from the first 16 bytes of cell data.
///
/// Amounts are encoded as little-endian u128 values.
fn parse_amount(data: &[u8]) -> Result<u128, ()> {
    if data.len() < 16 {
        return Err(());
    }

    let bytes: [u8; 16] = data[..16].try_into().map_err(|_| ())?;

    Ok(u128::from_le_bytes(bytes))
}

/// Calculate the total token amount in cells carrying this type script.
///
/// Only cells whose type script hash matches the current script hash
/// are included in the calculation.
fn sum_token_amounts(
    source: Source,
    type_hash: &ckb_std::ckb_types::packed::Byte32,
) -> Result<u128, ()> {
    let mut total: u128 = 0;
    let mut index: usize = 0;

    loop {
        let cell_type = match load_cell_type(index, source) {
            Ok(cell_type) => cell_type,
            Err(SysError::IndexOutOfBound) => break,
            Err(_) => return Err(()),
        };

        if let Some(cell_type) = cell_type {
            if cell_type.hash() == *type_hash {
                let data = load_cell_data(index, source)
                    .map_err(|_| ())?;

                let amount = parse_amount(&data)?;

                total = total.checked_add(amount).ok_or(())?;
            }
        }

        index += 1;
    }

    Ok(total)
}

/// Determine whether the token owner authorized the transaction.
///
/// The owner's lock script hash is stored in the type script arguments.
/// If any transaction input is locked by that script, minting is allowed.
fn is_owner_transaction(owner_hash: &[u8]) -> Result<bool, ()> {
    let mut index: usize = 0;

    loop {
        let lock = match load_cell_lock(index, Source::Input) {
            Ok(lock) => lock,
            Err(SysError::IndexOutOfBound) => break,
            Err(_) => return Err(()),
        };

        let lock_hash = lock.hash();

        if lock_hash.as_slice() == owner_hash {
            return Ok(true);
        }

        index += 1;
    }

    Ok(false)
}

/// Validate the simple fungible token transaction.
fn validate() -> Result<(), ()> {
    // Load the currently executing type script.
    let script = load_script().map_err(|_| ())?;

    // The script arguments must contain exactly 32 bytes:
    // the owner's lock script hash.
    let args = script.args().raw_data();

    if args.len() != 32 {
        return Err(());
    }

    let owner_hash = args.as_ref();

    // Calculate the current type script hash.
    let type_hash = script.hash();

    // Calculate the total token supply represented by input cells.
    let input_amount = sum_token_amounts(
        Source::Input,
        &type_hash,
    )?;

    // Calculate the total token supply represented by output cells.
    let output_amount = sum_token_amounts(
        Source::Output,
        &type_hash,
    )?;

    // Check whether the owner authorized this transaction.
    let owner_authorized = is_owner_transaction(owner_hash)?;

    if owner_authorized {
        // The owner may mint, transfer, or burn tokens.
        // No restriction is placed on the output amount.
        Ok(())
    } else {
        // Non-owners cannot create additional tokens.
        //
        // They may:
        // - Transfer tokens.
        // - Burn tokens.
        // - Keep the token supply unchanged.
        //
        // The total output amount must never exceed
        // the total input amount.
        if output_amount <= input_amount {
            Ok(())
        } else {
            Err(())
        }
    }
}

/// CKB script entry point.
///
/// Returns:
/// - 0  : Transaction is accepted.
/// - -1 : Transaction is rejected.
pub fn program_entry() -> i8 {
    match validate() {
        Ok(()) => 0,
        Err(()) => -1,
    }
}
