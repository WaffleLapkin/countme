//! A library to quickly get the live/total/max counts of allocated instances.
//!
//! # Example
//!
//! ```
//! # if cfg!(not(feature = "enable")) { return; }
//!
//! #[derive(Default)]
//! struct Widget {
//!   _c: countme::Count<Self>,
//! }
//!
//! let w1 = Widget::default();
//! let w2 = Widget::default();
//! let w3 = Widget::default();
//! drop(w1);
//!
//! let counts = countme::get::<Widget>();
//! assert_eq!(counts.live, 2);
//! assert_eq!(counts.max_live, 3);
//! assert_eq!(counts.total, 3);
//!
//! eprintln!("{}", countme::get_all());
//! ```
//!
//! # Configuration
//!
//! By default, the implementation compiles to no-ops. Therefore it is possible
//! to include `Count` fields into library types.
//!
//! The `enable` cargo feature ungates the counting code. The feature can be
//! enabled anywhere in the crate graph.
//!
//! At run-time, the counters are controlled with [`enable`] function. Counts
//! are enabled by default. Call `enable(false)` early in main to disable:
//!
//! ```rust
//! fn main() {
//!     countme::enable(std::env::var("COUNTME").is_ok());
//! }
//! ```
//!
//! The code is optimized for the case where counting is not enabled at runtime
//! (counting is a relaxed load and a branch to function call).
//!
//! The `print_at_exit` Cargo feature uses `atexit` call to print final counts
//! before the program exits. Use it only when you can't modify the main to
//! print counts -- `atexit` is not guaranteed to work with rust's runtime.
#[cfg(feature = "enable")]
mod imp;

use std::{fmt, marker::PhantomData};

#[derive(Debug, Clone, PartialEq, Eq, Default)]
#[non_exhaustive]
pub struct Counts {
    /// The total number of tokens created.
    pub total: usize,
    /// The historical maximum of the `live` count.
    pub max_live: usize,
    /// The number of tokens which were created, but are not destroyed yet.
    pub live: usize,
}

/// Store this inside your struct as `_c: countme::Count<Self>`.
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Count<T> {
    ghost: PhantomData<fn(T)>,
}

impl<T> Default for Count<T> {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl<T> Clone for Count<T> {
    #[inline]
    fn clone(&self) -> Self {
        Self::new()
    }
}

impl<T> Count<T> {
    #[inline]
    pub fn new() -> Count<T> {
        #[cfg(feature = "enable")]
        imp::inc::<T>();
        Count { ghost: PhantomData }
    }
}

impl<T> Drop for Count<T> {
    #[inline]
    fn drop(&mut self) {
        #[cfg(feature = "enable")]
        imp::dec::<T>();
    }
}

/// Enable or disable counting at runtime.
///
/// Counting is enabled by default.
pub fn enable(_yes: bool) {
    #[cfg(feature = "enable")]
    imp::enable(_yes);
}

/// Returns the counts for the `T` type.
#[inline]
pub fn get<T>() -> Counts {
    #[cfg(feature = "enable")]
    {
        return imp::get::<T>();
    }
    #[cfg(not(feature = "enable"))]
    {
        return Counts::default();
    }
}

/// Returns a collection of counts for all types.
pub fn get_all() -> AllCounts {
    #[cfg(feature = "enable")]
    {
        return imp::get_all();
    }
    #[cfg(not(feature = "enable"))]
    {
        return AllCounts::default();
    }
}

/// A collection of counts for all types.
#[derive(Default, Clone, Debug)]
pub struct AllCounts {
    entries: Vec<(&'static str, Counts)>,
}

impl fmt::Display for AllCounts {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.entries.is_empty() {
            return if cfg!(feature = "enable") {
                writeln!(f, "all counts are zero")
            } else {
                writeln!(f, "counts are disabled")
            };
        }

        writeln!(f, "cnt: {:>7} {:>10} {:>10}", "total", "max_live", "live")?;
        for (name, counts) in &self.entries {
            writeln!(
                f,
                "{}:\n  {:>10} {:>10} {:>10}",
                name, counts.total, counts.max_live, counts.live,
            )?;
        }
        if self.entries.len() > 10 {
            writeln!(f, "  {:>10} {:>10} {:>10}", "total", "max_live", "live")?;
        }
        Ok(())
    }
}
