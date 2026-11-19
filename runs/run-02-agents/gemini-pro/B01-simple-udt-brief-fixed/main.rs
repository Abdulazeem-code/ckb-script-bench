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
    high_level::{load_cell_data, load_cell_lock_hash, load_script},
};

const OWNER_LOCK_HASH_LEN: usize = 32;
const UDT_AMOUNT_LEN: usize = 16;

#[repr(i8)]
enum Error {
    IndexOutOfBound = 1,
    ItemMissing = 2,
    LengthNotEnough = 3,
    Encoding = 4,
    InvalidArgs = 5,
    AmountOverflow = 6,
    InsufficientInputAmount = 7,
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

fn is_owner_mode(owner_lock_hash: &[u8]) -> Result<bool, Error> {
    for i in 0.. {
        match load_cell_lock_hash(i, Source::Input) {
            Ok(lock_hash) => {
                if lock_hash.as_slice() == owner_lock_hash {
                    return Ok(true);
                }
            }
            Err(SysError::IndexOutOfBound) => break,
            Err(err) => return Err(err.into()),
        }
    }
    Ok(false)
}

fn collect_amount(source: Source) -> Result<u128, Error> {
    let mut total: u128 = 0;
    for i in 0.. {
        match load_cell_data(i, source) {
            Ok(data) => {
                if data.len() < UDT_AMOUNT_LEN {
                    return Err(Error::Encoding);
                }
                let mut buf = [0u8; UDT_AMOUNT_LEN];
                buf.copy_from_slice(&data[..UDT_AMOUNT_LEN]);
                let amount = u128::from_le_bytes(buf);
                total = total.checked_add(amount).ok_or(Error::AmountOverflow)?;
            }
            Err(SysError::IndexOutOfBound) => break,
            Err(err) => return Err(err.into()),
        }
    }
    Ok(total)
}

fn verify() -> Result<(), Error> {
    let script = load_script()?;
    let args = script.args().raw_data();
    if args.len() != OWNER_LOCK_HASH_LEN {
        return Err(Error::InvalidArgs);
    }

    if is_owner_mode(&args)? {
        return Ok(());
    }

    let inputs_amount = collect_amount(Source::GroupInput)?;
    let outputs_amount = collect_amount(Source::GroupOutput)?;

    if inputs_amount < outputs_amount {
        return Err(Error::InsufficientInputAmount);
    }

    Ok(())
}

pub fn program_entry() -> i8 {
    match verify() {
        Ok(()) => 0,
        Err(err) => err as i8,
    }
}
