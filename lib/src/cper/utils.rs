// Copyright (C) 2025 Intel Corporation
// SPDX-License-Identifier: MIT

#[inline]
pub fn bin_to_bcd(byte: u8) -> u8 {
    (byte / 10) << 4 | (byte % 10)
}
