#![cfg_attr(not(any(feature = "library", test)), no_std)]
#![cfg_attr(not(test), no_main)]

#[cfg(any(feature = "library", test))]
extern crate alloc;

#[cfg(not(any(feature = "library", test)))]
ckb_std::entry!(program_entry);
#[cfg(not(any(feature = "library", test)))]
ckb_std::default_alloc!(16384, 1258306, 64);

// Reference solution for task A02: absolute block-number timelock.
// See tasks/A02-absolute-timelock/spec.md.

use ckb_std::{ckb_constants::Source, error::SysError, high_level::load_input_since, high_level::load_script};

#[repr(i8)]
enum Error {
    IndexOutOfBound = 1,
    ItemMissing = 2,
    LengthNotEnough = 3,
    Encoding = 4,
    InvalidArgs = 5,
    WrongSinceFlags = 6,
    TooEarly = 7,
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

const VALUE_MASK: u64 = 0x00ff_ffff_ffff_ffff;

pub fn program_entry() -> i8 {
    match run() {
        Ok(()) => 0,
        Err(err) => err as i8,
    }
}

fn run() -> Result<(), Error> {
    let script = load_script()?;
    let args = script.args().raw_data();
    if args.len() != 8 {
        return Err(Error::InvalidArgs);
    }
    let mut buf = [0u8; 8];
    buf.copy_from_slice(&args);
    let unlock_block = u64::from_le_bytes(buf);
    if unlock_block == 0 {
        return Err(Error::InvalidArgs);
    }

    // Check every input in the group, not just the first.
    let mut index = 0;
    loop {
        let since = match load_input_since(index, Source::GroupInput) {
            Ok(since) => since,
            Err(SysError::IndexOutOfBound) => break,
            Err(err) => return Err(err.into()),
        };
        // Top byte 0x00: absolute, block-number metric, reserved bits clear.
        if since >> 56 != 0 {
            return Err(Error::WrongSinceFlags);
        }
        if since & VALUE_MASK < unlock_block {
            return Err(Error::TooEarly);
        }
        index += 1;
    }
    Ok(())
}
