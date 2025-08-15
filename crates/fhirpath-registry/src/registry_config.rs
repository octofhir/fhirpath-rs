
#[derive(Clone, Copy)]
pub struct RegistryConfig {
    pub cache_size: usize,
    pub sync_fastpath: bool,
}

impl Default for RegistryConfig {
    fn default() -> Self {
        Self {
            cache_size: 128,
            sync_fastpath: true,
        }
    }
}