#[cfg(unix)]
pub mod unix;
#[cfg(unix)]
pub use unix as platform;

#[cfg(windows)]
pub mod windows;
#[cfg(windows)]
pub use windows as platform;

#[cfg(not(any(unix, windows)))]
compile_error!("This crate currently supports only unix and windows targets.");
