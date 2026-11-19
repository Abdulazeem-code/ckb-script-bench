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
    high_level::{load_cell_lock_hash, load_script},
    syscalls::load_cell_data,
};

const UDT_LEN: usize = 16;
const OWNER_LOCK_HASH_LEN: usize = 32;

#[repr(i8)]
enum Error {
    IndexOutOfBound = 1,
    ItemMissing = 2,
    LengthNotEnough = 3,
    Encoding = 4,
    InvalidArgs = 5,
    InvalidDataLength = 6,
    Overflow = 7,
    AmountExceeded = 8,
}

impl From<SysError> for Error {
    fn from(err: SysError) -> Self {
        match err {
            SysError::IndexOutOfBound => Self::IndexOutOfBound,
            SysError::ItemMissing => Self::ItemMissing,
            SysError::LengthNotEnough(_) => Self::LengthNotEnough,
            SysError::Encoding => Self::Encoding,
            SysError::Unknown(err_code) => panic!("unexpected sys error {}", err_code),
            _ => Self::Encoding,
        }
    }
}

fn check_owner_mode(owner_lock_hash: &[u8]) -> Result<bool, Error> {
    let mut index = 0;
    loop {
        match load_cell_lock_hash(index, Source::Input) {
            Ok(lock_hash) => {
                if lock_hash.as_slice() == owner_lock_hash {
                    return Ok(true);
                }
            }
            Err(SysError::IndexOutOfBound) => break,
            Err(err) => return Err(err.into()),
        }
        index += 1;
    }
    Ok(false)
}

fn sum_udt_amount(source: Source) -> Result<u128, Error> {
    let mut sum: u128 = 0;
    let mut index = 0;
    let mut buf = [0u8; UDT_LEN];

    loop {
        match load_cell_data(&mut buf, 0, index, source) {
            Ok(len) if len == UDT_LEN => {}
            Ok(_) => return Err(Error::InvalidDataLength),
            Err(SysError::LengthNotEnough(_)) => {}
            Err(SysError::IndexOutOfBound) => break,
            Err(err) => return Err(err.into()),
        }

        let amount = u128::from_le_bytes(buf);
        sum = sum.checked_add(amount).ok_or(Error::Overflow)?;
        index += 1;
    }

    Ok(sum)
}

fn run() -> Result<(), Error> {
    let script = load_script()?;
    let args = script.args().raw_data();

    if args.len() != OWNER_LOCK_HASH_LEN {
        return Err(Error::InvalidArgs);
    }

    if check_owner_mode(&args)? {
        return Ok(());
    }

    let input_sum = sum_udt_amount(Source::GroupInput)?;
    let output_sum = sum_udt_amount(Source::GroupOutput)?;

    if output_sum > input_sum {
        return Err(Error::AmountExceeded);
    }

    Ok(())
}

pub fn program_entry() -> i8 {
    match run() {
        Ok(()) => 0,
        Err(err) => err as i8,
    }
}
