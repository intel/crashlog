// Copyright (C) 2026 Intel Corporation
// SPDX-License-Identifier: MIT

#[cfg(feature = "std")]
use std::{fmt, str::FromStr};

#[cfg(not(feature = "std"))]
use alloc::{fmt, str::FromStr, vec::Vec};

#[derive(Debug, Clone, Eq, Ord, PartialEq, PartialOrd)]
pub struct PciBdf {
    pub segment: u16,
    pub bus: u8,
    pub device: u8,
    pub function: u8,
}

impl PciBdf {
    pub fn new(segment: u16, bus: u8, device: u8, function: u8) -> Self {
        Self {
            segment,
            bus,
            device,
            function,
        }
    }

    pub fn from_pci_device(device: &str) -> Option<PciBdf> {
        let bdf: Vec<_> = device.split([':', '.']).collect();

        if bdf.len() != 4 {
            log::warn!(
                "Could not parse segment, bus, device function from: {}",
                device
            );
            return None;
        }

        let segment = u16::from_str_radix(bdf.first()?, 16).ok()?;
        let bus = u8::from_str_radix(bdf.get(1)?, 16).ok()?;
        let device = u8::from_str_radix(bdf.get(2)?, 16).ok()?;
        let function = u8::from_str_radix(bdf.get(3)?, 16).ok()?;

        Some(PciBdf {
            segment,
            bus,
            device,
            function,
        })
    }
}

impl fmt::Display for PciBdf {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{:04x}:{:02x}:{:02x}.{:x}",
            self.segment, self.bus, self.device, self.function
        )
    }
}

impl FromStr for PciBdf {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (segment, bdf) = s.split_once(":").ok_or(())?;
        let (bus, df) = bdf.split_once(":").ok_or(())?;
        let (device, function) = df.split_once(".").ok_or(())?;

        Ok(Self {
            segment: u16::from_str_radix(segment, 16).map_err(|_| ())?,
            bus: u8::from_str_radix(bus, 16).map_err(|_| ())?,
            device: u8::from_str_radix(device, 16).map_err(|_| ())?,
            function: u8::from_str_radix(function, 16).map_err(|_| ())?,
        })
    }
}
