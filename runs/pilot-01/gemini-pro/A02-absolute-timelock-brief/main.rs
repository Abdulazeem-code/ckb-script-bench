#![cfg_attr(not(any(feature = "library", test)), no_std)]
#![cfg_attr(not(test), no_main)]

#[cfg(any(feature = "library", test))]
extern crate alloc;

#[cfg(not(any(feature = "library", test)))]
ckb_std::entry!(program_entry);
#[cfg(not(any(feature = "library", test)))]
ckb_std::default_alloc!(16384, 1258306, 64);

use ckb_std::{
    ckb_constants::Source,
    error::SysError,
    high_level::{load_input_since, load_script},
};

const SINCE_FLAGS_MASK: u64 = 0xff00_0000_0000_0000;
const SINCE_VALUE_MASK: u64 = 0x00ff_ffff_ffff_ffff;

#[repr(i8)]
enum Error {
    IndexOutOfBound = 1,
    ItemMissing = 2,
    LengthNotEnough = 3,
    Encoding = 4,
    UnknownSysError = 5,
    InvalidArgs = 6,
    InvalidSinceFlags = 7,
    TimelockNotReached = 8,
}

impl From<SysError> for Error {
    fn from(err: SysError) -> Self {
        match err {
            SysError::IndexOutOfBound => Self::IndexOutOfBound,
            SysError::ItemMissing => Self::ItemMissing,
            SysError::LengthNotEnough(_) => Self::LengthNotEnough,
            SysError::Encoding => Self::Encoding,
            _ => Self::UnknownSysError,
        }
    }
}

fn verify() -> Result<(), Error> {
    let script = load_script()?;
    let args = script.args().raw_data();
    let unlock_block_bytes: [u8; 8] = args.as_ref().try_into().map_err(|_| Error::InvalidArgs)?;
    let unlock_block = u64::from_le_bytes(unlock_block_bytes);

    let mut i = 0;
    loop {
        match load_input_since(i, Source::GroupInput) {
            Ok(since) => {
                // Absolute block number requires bit 63 = 0 (absolute),
                // bits 62..61 = 00 (block number), and bits 60..56 = 0 (reserved).
                if (since & SINCE_FLAGS_MASK) != 0 {
                    return Err(Error::InvalidSinceFlags);
                }
                let since_block = since & SINCE_VALUE_MASK;
                if since_block < unlock_block {
                    return Err(Error::TimelockNotReached);
                }
                i += 1;
            }
            Err(SysError::IndexOutOfBound) => break,
            Err(err) => return Err(err.into()),
        }
    }

    Ok(())
}

pub fn program_entry() -> i8 {
    match verify() {
        Ok(()) => 0,
        Err(err) => err as i8,
    }
}
