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

#[repr(i8)]
enum Error {
    IndexOutOfBound = 1,
    ItemMissing,
    LengthNotEnough,
    Encoding,
    InvalidArgs,
    InvalidSince,
}

impl From<SysError> for Error {
    fn from(err: SysError) -> Self {
        match err {
            SysError::IndexOutOfBound => Self::IndexOutOfBound,
            SysError::ItemMissing => Self::ItemMissing,
            SysError::LengthNotEnough(_) => Self::LengthNotEnough,
            SysError::Encoding => Self::Encoding,
            _ => Self::Encoding,
        }
    }
}

fn validate() -> Result<(), Error> {
    let script = load_script()?;
    let args = script.args().raw_data();
    if args.len() != 8 {
        return Err(Error::InvalidArgs);
    }

    let mut buf = [0u8; 8];
    buf.copy_from_slice(&args);
    let target_block = u64::from_le_bytes(buf);
    if target_block == 0 {
        return Err(Error::InvalidArgs);
    }

    let mut index = 0;
    loop {
        match load_input_since(index, Source::GroupInput) {
            Ok(since) => {
                if (since >> 56) != 0 {
                    return Err(Error::InvalidSince);
                }
                let block_number = since & 0x00ff_ffff_ffff_ffff;
                if block_number < target_block {
                    return Err(Error::InvalidSince);
                }
            }
            Err(SysError::IndexOutOfBound) => break,
            Err(err) => return Err(err.into()),
        }
        index += 1;
    }

    Ok(())
}

pub fn program_entry() -> i8 {
    match validate() {
        Ok(()) => 0,
        Err(err) => err as i8,
    }
}
