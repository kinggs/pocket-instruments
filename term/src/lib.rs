//! Pocket Instruments' terminal host: Rhythm in a terminal (docs/TERMINAL.md, D123). It sends
//! what a finger did and draws the view, from the core's static library through its C ABI.
//!
//! SPDX-License-Identifier: MIT
pub mod audio;
pub mod beat;
pub mod ffi;
pub mod host;
pub mod input;
pub mod samples;
pub mod screen;
