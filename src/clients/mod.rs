#![allow(unused_imports)]

#[cfg(feature = "global")]
mod global;
#[cfg(feature = "global")]
pub use global::*;
#[cfg(feature = "cn")]
mod cn;
#[cfg(feature = "cn")]
pub use cn::*;
#[cfg(feature = "kr")]
mod kr;
#[cfg(feature = "kr")]
pub use kr::*;
#[cfg(feature = "steam")]
mod steam;
#[cfg(feature = "steam")]
pub use steam::*;

#[cfg(any(feature = "steam", feature="global"))]
pub(crate) mod global_utils;