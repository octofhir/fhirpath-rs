//! # Cache Provider

pub struct CacheProvider;

impl CacheProvider {
    pub fn new() -> Self { Self }
}

impl Default for CacheProvider {
    fn default() -> Self { Self::new() }
}