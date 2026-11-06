#![forbid(unsafe_code)]
//! Filesystem and randomness for Periapsis, real and simulated (INT-0005).
//!
//! Storage code does all of its filesystem I/O through [`Fs`] and draws its
//! randomness from [`Rng`]. [`RealFs`] is the operating system's filesystem.
//! [`SimFs`] is an in-memory filesystem that counts syscalls, can crash at any
//! one of them, and then recovers to a state chosen by its seed from what a
//! real filesystem may keep, so a whole run is reproducible from the seed.

mod fs;
mod real;
mod rng;
mod simfs;

pub use fs::{Crash, Fs, is_crash};
pub use real::RealFs;
pub use rng::Rng;
pub use simfs::{Hook, Mode, Node, SimFs};
