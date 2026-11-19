#![cfg_attr(not(any(feature = "library", test)), no_std)]
#![cfg_attr(not(test), no_main)]

#[cfg(any(feature = "library", test))]
extern crate alloc;

#[cfg(not(any(feature = "library", test)))]
ckb_std::entry!(program_entry);
#[cfg(not(any(feature = "library", test)))]
ckb_std::default_alloc!(16384, 1258306, 64);

use ckb_std::ckb_constants::Source;
use ckb_std::high_level::{
    load_cell_data,
    load_cell_lock_hash,
    load_cell_type_hash,
    load_script,
    load_script_hash,
    QueryIter,
};

/// Error code returned when the transaction violates the UDT rules.
const ERROR_INVALID_TRANSACTION: i8 = 1;

/// Read the token amount from the first 16 bytes of cell data.
///
/// Amounts are encoded as little-endian u128 values.
/// Additional bytes are permitted.
fn parse_amount(data: &[u8]) -> Result<u128, ()> {
    if data.len() < 16 {
        return Err(());
    }

    let mut bytes = [0u8; 16];
    bytes.copy_from_slice(&data[..16]);

    Ok(u128::from_le_bytes(bytes))
}

/// Calculate the total amount of tokens in cells belonging
/// to the currently executing type script.
///
/// Only cells whose type script hash matches the current
/// script hash are included.
fn calculate_total(
    source: Source,
    script_hash: &[u8; 32],
) -> Result<u128, ()> {
    let mut total: u128 = 0;

    for (index, _cell) in QueryIter::new(
        ckb_std::high_level::load_cell,
        source,
    )
    .enumerate()
    {
        let type_hash = load_cell_type_hash(index, source)
            .map_err(|_| ())?;

        // Ignore cells that do not belong to this UDT.
        if type_hash.as_ref() != Some(script_hash) {
            continue;
        }

        let data = load_cell_data(index, source)
            .map_err(|_| ())?;

        let amount = parse_amount(&data)?;

        total = total.checked_add(amount).ok_or(())?;
    }

    Ok(total)
}

/// Check whether the owner has authorized minting.
///
/// Authorization is established by the presence of at least
/// one transaction input whose lock script hash matches
/// the owner lock script hash stored in the UDT arguments.
fn owner_authorized(owner_lock_hash: &[u8; 32]) -> Result<bool, ()> {
    for (index, _cell) in QueryIter::new(
        ckb_std::high_level::load_cell,
        Source::Input,
    )
    .enumerate()
    {
        let lock_hash = load_cell_lock_hash(index, Source::Input)
            .map_err(|_| ())?;

        if &lock_hash == owner_lock_hash {
            return Ok(true);
        }
    }

    Ok(false)
}

/// Validate the UDT transaction.
fn validate() -> Result<(), ()> {
    // Load the currently executing type script.
    let script = load_script().map_err(|_| ())?;

    // The script arguments must contain exactly 32 bytes:
    // the owner's lock script hash.
    let args = script.args().raw_data();

    if args.len() != 32 {
        return Err(());
    }

    let mut owner_lock_hash = [0u8; 32];
    owner_lock_hash.copy_from_slice(args.as_ref());

    // Obtain the hash of the currently executing type script.
    let script_hash = load_script_hash().map_err(|_| ())?;

    // Calculate the total token supply consumed by this transaction.
    let input_total = calculate_total(
        Source::Input,
        &script_hash,
    )?;

    // Calculate the total token supply created by this transaction.
    let output_total = calculate_total(
        Source::Output,
        &script_hash,
    )?;

    // Check whether the owner has authorized minting.
    let is_owner = owner_authorized(&owner_lock_hash)?;

    // Non-owners cannot increase the total token supply.
    //
    // Burning is permitted because output_total may be
    // smaller than input_total.
    if !is_owner && output_total > input_total {
        return Err(());
    }

    Ok(())
}

/// CKB script entry point.
///
/// Returns:
///   0 - Transaction accepted.
///   1 - Transaction rejected.
pub fn program_entry() -> i8 {
    match validate() {
        Ok(()) => 0,
        Err(()) => ERROR_INVALID_TRANSACTION,
    }
}
