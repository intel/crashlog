// Copyright (C) 2025 Intel Corporation
// SPDX-License-Identifier: MIT

#[cfg(feature = "collateral_manager")]
use crate::{
    collateral::{ItemPath, PVSS},
    header::Version,
};
#[cfg(not(feature = "std"))]
use alloc::{fmt, str};
#[cfg(not(feature = "std"))]
use core::num;
#[cfg(feature = "std")]
use std::{fmt, io, num, str};

/// Errors reported by the Crash Log extraction and decoding functions.
#[derive(Debug)]
pub enum Error {
    InternalError,
    InvalidCrashLog,
    NoCrashLogFound,
    #[cfg(feature = "collateral_manager")]
    MissingCollateral(PVSS, ItemPath),
    #[cfg(feature = "collateral_manager")]
    MissingDecodeDefinitions(Version),
    InvalidBootErrorRecordRegion,
    InvalidHeader,
    EmptyRegion,
    InvalidHeaderType(u16),
    InvalidRecordType(u8),
    InvalidProductID(u32),
    #[cfg(feature = "serialize")]
    JsonError(serde_json::Error),
    Utf8Error(str::Utf8Error),
    ParseIntError(num::ParseIntError),
    #[cfg(feature = "std")]
    IOError(io::Error),
    #[cfg(feature = "std")]
    OsStringError(std::ffi::OsString),
}

#[cfg(feature = "std")]
impl std::error::Error for Error {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::InternalError => write!(f, "Internal error in the Crash Log library"),
            Error::NoCrashLogFound => write!(f, "No Crash Log could be found"),
            Error::InvalidCrashLog => write!(f, "The Crash Log is invalid"),
            #[cfg(feature = "collateral_manager")]
            Error::MissingCollateral(pvss, item) => {
                write!(f, "Missing {item} collateral file for {pvss}")
            }
            #[cfg(feature = "collateral_manager")]
            Error::MissingDecodeDefinitions(version) => {
                write!(f, "Missing decode definitions for {version}")
            }
            Error::InvalidBootErrorRecordRegion => write!(f, "Invalid Boot Error Record region"),
            Error::InvalidHeader => write!(f, "Invalid Crash Log Header"),
            Error::EmptyRegion => write!(f, "The Crash Log Region is not populated"),
            Error::InvalidHeaderType(ht) => write!(f, "Invalid Crash Log Header Type: {ht}"),
            Error::InvalidRecordType(rt) => write!(f, "Unknown Crash Log Record Type: {rt:#x}"),
            Error::InvalidProductID(pid) => write!(f, "Unknown Crash Log Product ID: {pid:#x}"),
            #[cfg(feature = "serialize")]
            Error::JsonError(err) => write!(f, "Invalid JSON file: {err}"),
            Error::Utf8Error(err) => write!(f, "UTF8 Error: {err}"),
            Error::ParseIntError(err) => write!(f, "Error while parsing integer: {err}"),
            #[cfg(feature = "std")]
            Error::IOError(err) => write!(f, "Encountered IO error: {err}"),
            #[cfg(feature = "std")]
            Error::OsStringError(s) => write!(f, "Cannot convert OS string: {s:?}"),
        }
    }
}

#[cfg(feature = "std")]
impl From<io::Error> for Error {
    fn from(err: io::Error) -> Self {
        Error::IOError(err)
    }
}

#[cfg(feature = "std")]
impl From<std::ffi::OsString> for Error {
    fn from(err: std::ffi::OsString) -> Self {
        Error::OsStringError(err)
    }
}

#[cfg(feature = "serialize")]
impl From<serde_json::Error> for Error {
    fn from(err: serde_json::Error) -> Self {
        Error::JsonError(err)
    }
}

impl From<str::Utf8Error> for Error {
    fn from(err: str::Utf8Error) -> Self {
        Error::Utf8Error(err)
    }
}

impl From<num::ParseIntError> for Error {
    fn from(err: num::ParseIntError) -> Self {
        Error::ParseIntError(err)
    }
}
