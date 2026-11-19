#![cfg_attr(not(any(feature = "library", test)), no_std)]
#![cfg_attr(not(test), no_main)]

#[cfg(any(feature = "library", test))]
extern crate alloc;

#[cfg(not(any(feature = "library", test)))]
ckb_std::entry!(program_entry);
#[cfg(not(any(feature = "library", test)))]
ckb_std::default_alloc!(16384, 1258306, 64);

// Reference solution for task B01: simple fungible token (UDT).
// See tasks/B01-simple-udt/spec.md.

use ckb_std::{
    ckb_constants::Source,
    error::SysError,
    high_level::{load_cell_data, load_cell_lock_hash, load_script, QueryIter},
};

#[repr(i8)]
enum Error {
    IndexOutOfBound = 1,
    ItemMissing = 2,
    LengthNotEnough = 3,
    Encoding = 4,
    InvalidArgs = 5,
    ShortData = 6,
    Overflow = 7,
    Inflation = 8,
}

impl From<SysError> for Error {
    fn from(err: SysError) -> Self {
        match err {
            SysError::IndexOutOfBound => Error::IndexOutOfBound,
            SysError::ItemMissing => Error::ItemMissing,
            SysError::LengthNotEnough(_) => Error::LengthNotEnough,
            _ => Error::Encoding,
        }
    }
}

pub fn program_entry() -> i8 {
    match run() {
        Ok(()) => 0,
        Err(err) => err as i8,
    }
}

fn run() -> Result<(), Error> {
    let script = load_script()?;
    let args = script.args().raw_data();
    if args.len() != 32 {
        return Err(Error::InvalidArgs);
    }

    // Owner mode: the owner's lock must guard one of the inputs.
    let owner_mode = QueryIter::new(load_cell_lock_hash, Source::Input)
        .any(|lock_hash| lock_hash[..] == args[..]);
    if owner_mode {
        return Ok(());
    }

    let input_sum = sum_amounts(Source::GroupInput)?;
    let output_sum = sum_amounts(Source::GroupOutput)?;
    if output_sum > input_sum {
        return Err(Error::Inflation);
    }
    Ok(())
}

fn sum_amounts(source: Source) -> Result<u128, Error> {
    let mut total: u128 = 0;
    for data in QueryIter::new(load_cell_data, source) {
        if data.len() < 16 {
            return Err(Error::ShortData);
        }
        let mut buf = [0u8; 16];
        buf.copy_from_slice(&data[..16]);
        total = total
            .checked_add(u128::from_le_bytes(buf))
            .ok_or(Error::Overflow)?;
    }
    Ok(total)
}
