#![cfg_attr(not(any(feature = "library", test)), no_std)]
#![cfg_attr(not(test), no_main)]

#[cfg(any(feature = "library", test))]
extern crate alloc;

#[cfg(not(any(feature = "library", test)))]
ckb_std::entry!(program_entry);
#[cfg(not(any(feature = "library", test)))]
ckb_std::default_alloc!(16384, 1258306, 64);

// Reference solution for task A01: hash-preimage lock.
// See tasks/A01-hash-preimage-lock/spec.md.

use ckb_hash::blake2b_256;
use ckb_std::{
    ckb_constants::Source,
    error::SysError,
    high_level::{load_script, load_witness_args},
};

#[repr(i8)]
enum Error {
    IndexOutOfBound = 1,
    ItemMissing = 2,
    LengthNotEnough = 3,
    Encoding = 4,
    InvalidArgs = 5,
    MissingPreimage = 6,
    WrongPreimage = 7,
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

    // Witness at the index of the first input in this script group.
    let witness_args = load_witness_args(0, Source::GroupInput)?;
    let preimage = witness_args
        .lock()
        .to_opt()
        .ok_or(Error::MissingPreimage)?
        .raw_data();

    if blake2b_256(&preimage)[..] != args[..] {
        return Err(Error::WrongPreimage);
    }
    Ok(())
}
