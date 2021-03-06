#![deny(
    missing_docs,
    missing_debug_implementations,
    missing_copy_implementations,
    trivial_casts,
    trivial_numeric_casts,
    unsafe_code,
    unstable_features,
    unused_import_braces,
    unused_qualifications,
    warnings
)]
#![cfg_attr(feature = "dev", allow(unstable_features))]
#![cfg_attr(feature = "dev", feature(plugin))]
#![cfg_attr(feature = "dev", plugin(clippy))]

//! Manages all hooks and calls the given function
//! when a hook is called
extern crate futures_cpupool;

mod message_state;
mod hook_codes;
mod hooks;
mod notification;
mod register;

pub use hook_codes::{as_enum, as_number, HookCodes};
pub use hooks::Hooks;
pub use notification::HookNotification;
pub use register::HookRegister;
pub use message_state::MessageState;