//! Some common traits that need to be often available.

#[cfg(feature = "threadsafe")]
pub use super::delegate::{DelegateError, DelegateFuture, DelegateFutureFuture};
pub use super::prop::*;
pub use browser_window_core::prelude::*;