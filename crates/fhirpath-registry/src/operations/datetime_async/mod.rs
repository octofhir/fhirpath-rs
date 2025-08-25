//! DateTime operations - async implementations
//! 
//! These operations require system calls and must be async.

pub mod now;
pub mod today;

pub use now::NowFunction;
pub use today::TodayFunction;