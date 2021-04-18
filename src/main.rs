use std::collections::HashMap;
use std::fs::{self, File};

// Should we add `use anyhow::{Error, Result};` ?
//
// That depends. Does this file also want to access `std::error::Error`?
// Does this file also want to access `std::result::Result`?
// The latter seems more likely, so we may want to only `use anyhow::Error`.
// The answer depends on what your code is doing.
use anyhow::{anyhow, bail, Context};

// Basic use of `anyhow` to handle disparate error types
//
// `anyhow` can do the same job as `dyn std::error::Error`: it can
// be a container for whatever error types you want to throw.
// In this example, we might get an `io::Error` when trying to read
// the file, but we might also get a `ParseIntError` from the
// call to `parse`.

pub fn open_file_1() -> anyhow::Result<u64> {
    let filename = "nonexistent_file";
    let data = fs::read_to_string(filename)?;
    let n: u64 = data.parse()?;
    Ok(n)
}

// Using `anyhow` to add context
//
// Note: must add `use anyhow::Context` to get access to
// `.context()` and `.with_context()`.
//
// If we propagate the error without any context, then main will print:
// |    Error: No such file or directory (os error 2)
//
// After adding context, then main will print:
// |   Error: failed to open "nonexistent"
// |
// |   Caused by:
// |       No such file or directory (os error 2)
//
// It's possible that our caller doesn't really want the io::Error
// to be propagated: do they care why the file open failed?
// It might be wiser to create a `LogfileError` type instead.
// Even though you can use `?` to do magic conversions from
// any error type to `anyhow::Error`, not all errors are useful
// when propagated to the caller.

pub fn open_file_2() -> anyhow::Result<()> {
    let filename = "nonexistent_logfile";
    File::open(filename).with_context(|| format!("failed to open {:?}", filename))?;
    Ok(())
}

// Generating an anyhow error from a string
//
// This is probably the fastest way to do error handling.
// If we're writing first-draft code and we just need it working
// as quickly as possible, this is a good way to get that.
//
// It's most suitable for high-level code, where we know there's
// not a caller that will be inspecting the errors we return. If
// the errors are only going to be printed to the terminal, this
// works fine.
//
// Note in many contexts it's easiest to use
// `bail!("something happened")`.
// That won't work inside `ok_or_else`, because `bail!` expands to `return Err(...)`
// and that's not the right return type _inside the closure_.
// So inside the closure we use the anyhow! macro instead.

pub fn access_map_1(key: u32) -> anyhow::Result<u32> {
    let map = HashMap::<u32, u32>::new();

    // This won't work: map.get(42)?
    // Reason: can't turn an Option into anyhow::Result
    // We need to summon a real error type.
    let n = *map.get(&key).ok_or_else(|| anyhow!("key lookup failure"))?;
    // It's a valid id number if it's a multiple of 7.
    if n % 7 != 0 {
        bail!("not divisible by 7");
    }
    Ok(n)
}

// Creating a very simple error type in `thiserror`
//
// Like the previous example, we need to convert Option::None into
// some kind of error type. In this example, we decide that we want
// a real error type. This might be desirable if:
// 1. The error path needs to be highly optimized, and you don't want
//    it allocating any memory or doing dynamic dispatch.
// 2. It's likely that we'll expand the error type into an enum with
//    multiple variants (that the caller will match on).
//
// Note that the error type has zero size. This is really friendly to
// the compiler's ability to inline or optimize.
// Note also that we don't return anyhow::Error here; that would kind
// of negate the potential optimization benefits.
//
// When printed by `main`, this error looks the same as `access_map_1()`

#[derive(Debug, thiserror::Error)]
#[error("key lookup failure")]
pub struct LookupFailure;

pub fn access_map_2(key: u32) -> Result<u32, LookupFailure> {
    let map = HashMap::<u32, u32>::new();

    // This won't work: map.get(42)?
    // Reason: can't turn an Option into anyhow::Result
    // We need to summon a real error type.
    let n = map.get(&key).ok_or(LookupFailure)?;
    Ok(*n)
}

// Creating an error enum type in `thiserror`
//
// If we anticipate that callers may want to match on our
// error type, we should add an enum variant for each.
//
// If this is a public crate, we might want to add the [`non_exhaustive`]
// attribute to error enums so we can add more variants in the
// future without it being a breaking change.
//
// [`non_exhaustive`]: https://doc.rust-lang.org/reference/attributes/type_system.html#the-non_exhaustive-attribute

#[derive(Debug, thiserror::Error)]
pub enum IdNumberError {
    #[error("id lookup failure")]
    LookupFailure,
    #[error("invalid id number ({0})")]
    InvalidNumber(u32),
}

pub fn access_map_3(key: u32) -> Result<u32, IdNumberError> {
    let mut map = HashMap::<u32, u32>::new();
    map.insert(41, 76);
    map.insert(42, 77);

    let n = *map.get(&key).ok_or(IdNumberError::LookupFailure)?;

    // It's a valid id number if it's a multiple of 7.
    if n % 7 == 0 {
        Ok(n)
    } else {
        Err(IdNumberError::InvalidNumber(n))
    }
}

fn main() -> anyhow::Result<()> {
    // Uncomment the one you want.

    //open_file_1()?;

    //open_file_2()?;

    //access_map_1(41)?;

    //access_map_2(41)?;

    access_map_3(41)?;

    println!("Success!");
    Ok(())
}
