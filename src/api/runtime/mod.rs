// SPDX-License-Identifier: MIT OR Apache-2.0
use super::*;

use std::hint;

mod environment;
mod exports;
mod memory;
mod module;
mod version;

pub(super) use self::{environment::*, memory::*, module::*, version::*};

pub(super) fn retain_exported_symbols() {
    hint::black_box(&exports::EXPORTED_SYMBOLS);
}
