#[cfg(windows)]
pub mod convert;
#[cfg(windows)]
pub mod last_word;
pub mod mapping;
#[cfg(windows)]
mod selection_probe;
#[cfg(windows)]
mod smart;

#[cfg(windows)]
pub use convert::{switch_keyboard_layout, wait_shift_released};

/// Compatibility shim: the implementation lives in the cross-platform core
/// crate.
pub mod case_switch {
    pub use rust_switcher_core::text::case_switch::*;
}
