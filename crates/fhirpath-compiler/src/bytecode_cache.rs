// Copyright 2024 OctoFHIR Team
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use crate::bytecode::{Bytecode, BytecodeMetadata, Instruction};
use dashmap::DashMap;
use fhirpath_model::FhirPathValue;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::sync::{Arc, Weak};
use std::time::{SystemTime, UNIX_EPOCH};

/// Shared bytecode structure with Arc-wrapped components for efficient sharing
#[derive(Debug, Clone)]
pub struct SharedBytecode {
    /// Instruction sequence shared via Arc
    pub instructions: Arc<[Instruction]>,
    /// Constants pool shared via Arc
    pub constants: Arc<[FhirPathValue]>,
    /// String constants shared via Arc
    pub strings: Arc<[String]>,
    /// Compilation metadata shared via Arc
    pub metadata: Arc<CompilationMetadata>,
    /// Cache entry metadata
    pub cache_metadata: CacheMetadata,
}

/// Extended compilation metadata for cache management
#[derive(Debug, Clone)]
pub struct CompilationMetadata {
    /// Original bytecode metadata
    pub bytecode_metadata: BytecodeMetadata,
    /// Expression hash for cache key generation
    pub expression_hash: u64,
    /// Compilation timestamp
    pub compilation_time: u64,
    /// Maximum stack depth required
    pub max_stack_depth: usize,
    /// Estimated execution complexity (for cache priority)
    pub complexity_score: u32,
}

/// Cache entry metadata for management and eviction
#[derive(Debug, Clone)]
pub struct CacheMetadata {
    /// Number of times this bytecode has been accessed
    pub access_count: u64,
    /// Last access timestamp
    pub last_accessed: u64,
    /// Size in bytes (estimated)
    pub estimated_size: usize,
    /// Version for cache invalidation
    pub cache_version: u32,
}

impl SharedBytecode {
    /// Create a new SharedBytecode from regular Bytecode
    pub fn from_bytecode(bytecode: Bytecode, expression: &str, complexity_score: u32) -> Self {
        let expression_hash = Self::hash_expression(expression);
        let compilation_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let estimated_size = Self::estimate_size(&bytecode);

        Self {
            instructions: Arc::from(bytecode.instructions),
            constants: Arc::from(bytecode.constants),
            strings: Arc::from(bytecode.strings),
            metadata: Arc::new(CompilationMetadata {
                bytecode_metadata: bytecode.metadata,
                expression_hash,
                compilation_time,
                max_stack_depth: bytecode.max_stack_depth,
                complexity_score,
            }),
            cache_metadata: CacheMetadata {
                access_count: 0,
                last_accessed: compilation_time,
                estimated_size,
                cache_version: CACHE_VERSION.load(std::sync::atomic::Ordering::Relaxed),
            },
        }
    }

    /// Convert back to regular Bytecode for execution
    pub fn to_bytecode(&self) -> Bytecode {
        Bytecode {
            instructions: self.instructions.to_vec(),
            constants: self.constants.to_vec(),
            strings: self.strings.to_vec(),
            metadata: self.metadata.bytecode_metadata.clone(),
            max_stack_depth: self.metadata.max_stack_depth,
        }
    }

    /// Get a compressed representation for storage
    pub fn compress(&self) -> CompressedBytecode {
        CompressedBytecode::from_shared(self)
    }

    /// Record an access to this bytecode for cache statistics
    pub fn record_access(&mut self) {
        self.cache_metadata.access_count += 1;
        self.cache_metadata.last_accessed = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
    }

    /// Check if this bytecode is still valid for the current cache version
    pub fn is_valid(&self) -> bool {
        self.cache_metadata.cache_version
            == CACHE_VERSION.load(std::sync::atomic::Ordering::Relaxed)
    }

    /// Hash an expression string for cache key generation
    fn hash_expression(expression: &str) -> u64 {
        let mut hasher = DefaultHasher::new();
        expression.hash(&mut hasher);
        hasher.finish()
    }

    /// Estimate the memory size of a bytecode object
    fn estimate_size(bytecode: &Bytecode) -> usize {
        let instructions_size = bytecode.instructions.len() * std::mem::size_of::<Instruction>();
        let constants_size = bytecode.constants.len() * std::mem::size_of::<FhirPathValue>();
        let strings_size: usize = bytecode.strings.iter().map(|s| s.len()).sum::<usize>()
            + bytecode.strings.len() * std::mem::size_of::<String>();

        instructions_size + constants_size + strings_size + std::mem::size_of::<BytecodeMetadata>()
    }
}

/// Compressed bytecode representation for efficient storage
#[derive(Debug, Clone)]
pub struct CompressedBytecode {
    /// Compressed instruction data
    compressed_instructions: Vec<u8>,
    /// Constants (not compressed as they're already optimized)
    constants: Arc<[FhirPathValue]>,
    /// Strings (shared)
    strings: Arc<[String]>,
    /// Metadata
    metadata: Arc<CompilationMetadata>,
    /// Original instruction count for validation
    original_instruction_count: usize,
}

impl CompressedBytecode {
    /// Create compressed bytecode from shared bytecode
    pub fn from_shared(shared: &SharedBytecode) -> Self {
        let compressed_instructions = Self::compress_instructions(&shared.instructions);

        Self {
            compressed_instructions,
            constants: shared.constants.clone(),
            strings: shared.strings.clone(),
            metadata: shared.metadata.clone(),
            original_instruction_count: shared.instructions.len(),
        }
    }

    /// Decompress back to SharedBytecode
    pub fn decompress(&self) -> Result<SharedBytecode, CompressionError> {
        let instructions = Self::decompress_instructions(&self.compressed_instructions)?;

        if instructions.len() != self.original_instruction_count {
            return Err(CompressionError::ValidationFailed);
        }

        Ok(SharedBytecode {
            instructions: Arc::from(instructions),
            constants: self.constants.clone(),
            strings: self.strings.clone(),
            metadata: self.metadata.clone(),
            cache_metadata: CacheMetadata {
                access_count: 0,
                last_accessed: SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs(),
                estimated_size: self.estimate_decompressed_size(),
                cache_version: CACHE_VERSION.load(std::sync::atomic::Ordering::Relaxed),
            },
        })
    }

    /// Simple compression: pack instructions into bytes
    /// This is a placeholder - could be improved with better compression algorithms
    fn compress_instructions(instructions: &[Instruction]) -> Vec<u8> {
        let mut compressed = Vec::new();

        for instruction in instructions {
            compressed.push(Self::instruction_to_byte(instruction));

            // Add operands as additional bytes
            match instruction {
                Instruction::PushConstant(idx)
                | Instruction::LoadProperty(idx)
                | Instruction::LoadIndexedProperty(idx)
                | Instruction::LoadVariable(idx)
                | Instruction::StoreVariable(idx)
                | Instruction::BindParameter(idx)
                | Instruction::FastConstant(idx)
                | Instruction::FastProperty(idx) => {
                    compressed.extend_from_slice(&idx.to_le_bytes());
                }
                Instruction::CallFunction(func_idx, arity) => {
                    compressed.extend_from_slice(&func_idx.to_le_bytes());
                    compressed.push(*arity);
                }
                Instruction::CallMethod(name_idx, arity) => {
                    compressed.extend_from_slice(&name_idx.to_le_bytes());
                    compressed.push(*arity);
                }
                Instruction::Jump(offset)
                | Instruction::JumpIfFalse(offset)
                | Instruction::JumpIfTrue(offset) => {
                    compressed.extend_from_slice(&offset.to_le_bytes());
                }
                Instruction::IsType(type_id) | Instruction::AsType(type_id) => {
                    compressed.extend_from_slice(&type_id.to_le_bytes());
                }
                Instruction::MakeCollection(count) => {
                    compressed.push(*count);
                }
                _ => {} // No operands
            }
        }

        compressed
    }

    /// Decompress instructions from byte representation
    fn decompress_instructions(compressed: &[u8]) -> Result<Vec<Instruction>, CompressionError> {
        let mut instructions = Vec::new();
        let mut i = 0;

        while i < compressed.len() {
            let opcode = compressed[i];
            i += 1;

            let instruction = match opcode {
                0 => {
                    if i + 2 > compressed.len() {
                        return Err(CompressionError::UnexpectedEnd);
                    }
                    let idx = u16::from_le_bytes([compressed[i], compressed[i + 1]]);
                    i += 2;
                    Instruction::PushConstant(idx)
                }
                1 => Instruction::PushInput,
                2 => Instruction::Duplicate,
                3 => Instruction::Pop,
                4 => Instruction::Swap,
                5 => {
                    if i + 2 > compressed.len() {
                        return Err(CompressionError::UnexpectedEnd);
                    }
                    let idx = u16::from_le_bytes([compressed[i], compressed[i + 1]]);
                    i += 2;
                    Instruction::LoadProperty(idx)
                }
                6 => {
                    if i + 2 > compressed.len() {
                        return Err(CompressionError::UnexpectedEnd);
                    }
                    let idx = u16::from_le_bytes([compressed[i], compressed[i + 1]]);
                    i += 2;
                    Instruction::LoadIndexedProperty(idx)
                }
                7 => Instruction::IndexAccess,
                8 => {
                    if i + 3 > compressed.len() {
                        return Err(CompressionError::UnexpectedEnd);
                    }
                    let func_idx = u16::from_le_bytes([compressed[i], compressed[i + 1]]);
                    let arity = compressed[i + 2];
                    i += 3;
                    Instruction::CallFunction(func_idx, arity)
                }
                9 => {
                    if i + 3 > compressed.len() {
                        return Err(CompressionError::UnexpectedEnd);
                    }
                    let name_idx = u16::from_le_bytes([compressed[i], compressed[i + 1]]);
                    let arity = compressed[i + 2];
                    i += 3;
                    Instruction::CallMethod(name_idx, arity)
                }
                // Add more opcodes as needed...
                10 => Instruction::Return,
                _ => return Err(CompressionError::UnknownOpcode(opcode)),
            };

            instructions.push(instruction);
        }

        Ok(instructions)
    }

    /// Convert instruction to byte opcode
    fn instruction_to_byte(instruction: &Instruction) -> u8 {
        match instruction {
            Instruction::PushConstant(_) => 0,
            Instruction::PushInput => 1,
            Instruction::Duplicate => 2,
            Instruction::Pop => 3,
            Instruction::Swap => 4,
            Instruction::LoadProperty(_) => 5,
            Instruction::LoadIndexedProperty(_) => 6,
            Instruction::IndexAccess => 7,
            Instruction::CallFunction(_, _) => 8,
            Instruction::CallMethod(_, _) => 9,
            Instruction::Return => 10,
            // Add more mappings as needed...
            _ => 255, // Unknown/unsupported instruction
        }
    }

    fn estimate_decompressed_size(&self) -> usize {
        self.original_instruction_count * std::mem::size_of::<Instruction>()
            + self.constants.len() * std::mem::size_of::<FhirPathValue>()
            + self.strings.iter().map(|s| s.len()).sum::<usize>()
    }
}

/// Global bytecode cache with weak references and automatic cleanup
pub struct GlobalBytecodeCache {
    /// Main cache storage using weak references to enable automatic cleanup
    cache: DashMap<String, Weak<SharedBytecode>>,
    /// Strong references for recently accessed items to prevent premature cleanup
    recent_cache: DashMap<String, Arc<SharedBytecode>>,
    /// Cache configuration
    config: CacheConfig,
    /// Statistics
    stats: CacheStats,
}

/// Cache configuration
#[derive(Debug, Clone)]
pub struct CacheConfig {
    /// Maximum number of entries in the main cache
    pub max_entries: usize,
    /// Maximum number of entries to keep strong references to
    pub recent_entries_limit: usize,
    /// Maximum total estimated size in bytes
    pub max_total_size: usize,
    /// Whether to enable compression for stored bytecode
    pub enable_compression: bool,
    /// Cleanup interval in seconds
    pub cleanup_interval: u64,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            max_entries: 10000,
            recent_entries_limit: 100,
            max_total_size: 50 * 1024 * 1024, // 50MB
            enable_compression: true,
            cleanup_interval: 300, // 5 minutes
        }
    }
}

/// Cache statistics for monitoring and optimization
#[derive(Debug, Default)]
pub struct CacheStats {
    /// Total cache hits
    pub hits: std::sync::atomic::AtomicU64,
    /// Total cache misses
    pub misses: std::sync::atomic::AtomicU64,
    /// Number of entries evicted
    pub evictions: std::sync::atomic::AtomicU64,
    /// Number of cleanup operations performed
    pub cleanups: std::sync::atomic::AtomicU64,
    /// Current estimated cache size
    pub current_size: std::sync::atomic::AtomicUsize,
}

impl CacheStats {
    /// Calculates the cache hit rate as a percentage
    ///
    /// # Returns
    /// * `f64` - Hit rate between 0.0 and 1.0, or 0.0 if no cache operations occurred
    pub fn hit_rate(&self) -> f64 {
        let hits = self.hits.load(std::sync::atomic::Ordering::Relaxed);
        let misses = self.misses.load(std::sync::atomic::Ordering::Relaxed);
        let total = hits + misses;

        if total == 0 {
            0.0
        } else {
            hits as f64 / total as f64
        }
    }
}

impl GlobalBytecodeCache {
    /// Create a new cache with default configuration
    pub fn new() -> Self {
        Self::with_config(CacheConfig::default())
    }

    /// Create a new cache with custom configuration
    pub fn with_config(config: CacheConfig) -> Self {
        Self {
            cache: DashMap::new(),
            recent_cache: DashMap::new(),
            config,
            stats: CacheStats::default(),
        }
    }

    /// Get bytecode from cache or return None if not found
    pub fn get(&self, key: &str) -> Option<Arc<SharedBytecode>> {
        // First check recent cache (strong references)
        if let Some(bytecode) = self.recent_cache.get(key) {
            self.stats
                .hits
                .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            let mut shared = (*bytecode.clone()).clone();
            shared.record_access();
            return Some(Arc::new(shared));
        }

        // Check main cache (weak references)
        if let Some(weak_ref) = self.cache.get(key) {
            if let Some(strong_ref) = weak_ref.upgrade() {
                self.stats
                    .hits
                    .fetch_add(1, std::sync::atomic::Ordering::Relaxed);

                // Move to recent cache
                self.add_to_recent_cache(key.to_string(), strong_ref.clone());

                let mut shared = (*strong_ref).clone();
                shared.record_access();
                return Some(Arc::new(shared));
            } else {
                // Weak reference is dead, remove it
                self.cache.remove(key);
            }
        }

        self.stats
            .misses
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        None
    }

    /// Insert bytecode into cache
    pub fn insert(&self, key: String, bytecode: SharedBytecode) -> Arc<SharedBytecode> {
        let shared = Arc::new(bytecode);

        // Add to main cache as weak reference
        self.cache.insert(key.clone(), Arc::downgrade(&shared));

        // Add to recent cache as strong reference
        self.add_to_recent_cache(key, shared.clone());

        // Update size tracking
        self.stats.current_size.fetch_add(
            shared.cache_metadata.estimated_size,
            std::sync::atomic::Ordering::Relaxed,
        );

        // Trigger cleanup if needed
        self.maybe_cleanup();

        shared
    }

    /// Add entry to recent cache with eviction if needed
    fn add_to_recent_cache(&self, key: String, bytecode: Arc<SharedBytecode>) {
        // Remove oldest entry if at limit
        if self.recent_cache.len() >= self.config.recent_entries_limit {
            self.evict_oldest_recent();
        }

        self.recent_cache.insert(key, bytecode);
    }

    /// Evict the oldest entry from recent cache based on access time
    fn evict_oldest_recent(&self) {
        let mut oldest_key: Option<String> = None;
        let mut oldest_time = u64::MAX;

        for entry in self.recent_cache.iter() {
            let access_time = entry.value().cache_metadata.last_accessed;
            if access_time < oldest_time {
                oldest_time = access_time;
                oldest_key = Some(entry.key().clone());
            }
        }

        if let Some(key) = oldest_key {
            if let Some((_key, bytecode)) = self.recent_cache.remove(&key) {
                self.stats.current_size.fetch_sub(
                    bytecode.cache_metadata.estimated_size,
                    std::sync::atomic::Ordering::Relaxed,
                );
                self.stats
                    .evictions
                    .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            }
        }
    }

    /// Perform cache cleanup if needed
    fn maybe_cleanup(&self) {
        let current_size = self
            .stats
            .current_size
            .load(std::sync::atomic::Ordering::Relaxed);

        if current_size > self.config.max_total_size || self.cache.len() > self.config.max_entries {
            self.cleanup();
        }
    }

    /// Cleanup dead weak references and evict entries if needed
    pub fn cleanup(&self) {
        self.stats
            .cleanups
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);

        // Remove dead weak references
        self.cache
            .retain(|_key, weak_ref| weak_ref.strong_count() > 0);

        // If still over size limit, evict from recent cache
        while self
            .stats
            .current_size
            .load(std::sync::atomic::Ordering::Relaxed)
            > self.config.max_total_size
            && !self.recent_cache.is_empty()
        {
            self.evict_oldest_recent();
        }
    }

    /// Get cache statistics
    pub fn stats(&self) -> &CacheStats {
        &self.stats
    }

    /// Clear all cache entries
    pub fn clear(&self) {
        self.cache.clear();
        self.recent_cache.clear();
        self.stats
            .current_size
            .store(0, std::sync::atomic::Ordering::Relaxed);
    }

    /// Invalidate cache by incrementing version
    pub fn invalidate(&self) {
        CACHE_VERSION.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        self.clear();
    }
}

impl Default for GlobalBytecodeCache {
    fn default() -> Self {
        Self::new()
    }
}

/// Global cache version for invalidation
static CACHE_VERSION: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(1);

/// Global cache instance
static GLOBAL_CACHE: once_cell::sync::Lazy<GlobalBytecodeCache> =
    once_cell::sync::Lazy::new(GlobalBytecodeCache::new);

/// Get the global bytecode cache instance
pub fn global_cache() -> &'static GlobalBytecodeCache {
    &GLOBAL_CACHE
}

/// Compression error types
#[derive(Debug, Clone)]
pub enum CompressionError {
    /// Unexpected end of compressed data during decompression
    UnexpectedEnd,
    /// Unknown opcode encountered during decompression
    UnknownOpcode(u8),
    /// Validation failed during decompression process
    ValidationFailed,
}

impl std::fmt::Display for CompressionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnexpectedEnd => write!(f, "Unexpected end of compressed data"),
            Self::UnknownOpcode(opcode) => write!(f, "Unknown opcode: {opcode}"),
            Self::ValidationFailed => write!(f, "Validation failed during decompression"),
        }
    }
}

impl std::error::Error for CompressionError {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bytecode::OptimizationLevel;

    fn create_test_bytecode() -> Bytecode {
        use fhirpath_model::FhirPathValue;

        Bytecode {
            instructions: vec![
                Instruction::PushInput,
                Instruction::LoadProperty(0),
                Instruction::Return,
            ],
            constants: vec![FhirPathValue::Integer(42)],
            strings: vec!["name".to_string()],
            metadata: BytecodeMetadata {
                source: Some("Patient.name".to_string()),
                optimization_level: OptimizationLevel::Basic,
                uses_lambdas: false,
                modifies_variables: false,
                complexity_score: 1,
            },
            max_stack_depth: 2,
        }
    }

    #[test]
    fn test_shared_bytecode_conversion() {
        let original = create_test_bytecode();
        let shared = SharedBytecode::from_bytecode(original.clone(), "Patient.name", 1);
        let converted_back = shared.to_bytecode();

        assert_eq!(original.instructions, converted_back.instructions);
        assert_eq!(original.constants.len(), converted_back.constants.len());
        assert_eq!(original.strings, converted_back.strings);
    }

    #[test]
    fn test_cache_operations() {
        let cache = GlobalBytecodeCache::new();
        let bytecode = create_test_bytecode();
        let shared = SharedBytecode::from_bytecode(bytecode, "Patient.name", 1);

        // Test miss
        assert!(cache.get("Patient.name").is_none());
        assert_eq!(
            cache
                .stats()
                .misses
                .load(std::sync::atomic::Ordering::Relaxed),
            1
        );

        // Test insert and hit
        let cached = cache.insert("Patient.name".to_string(), shared);
        let retrieved = cache.get("Patient.name").unwrap();

        assert_eq!(cached.instructions.len(), retrieved.instructions.len());
        assert_eq!(
            cache
                .stats()
                .hits
                .load(std::sync::atomic::Ordering::Relaxed),
            1
        );
    }

    #[test]
    fn test_compression() {
        let original = create_test_bytecode();
        let shared = SharedBytecode::from_bytecode(original, "Patient.name", 1);
        let compressed = shared.compress();
        let decompressed = compressed.decompress().unwrap();

        assert_eq!(shared.instructions.len(), decompressed.instructions.len());
        assert_eq!(shared.constants.len(), decompressed.constants.len());
    }

    #[test]
    fn test_cache_eviction() {
        let config = CacheConfig {
            recent_entries_limit: 2,
            ..CacheConfig::default()
        };
        let cache = GlobalBytecodeCache::with_config(config);

        // Add more entries than the limit
        for i in 0..5 {
            let bytecode = create_test_bytecode();
            let shared = SharedBytecode::from_bytecode(bytecode, &format!("expr{i}"), 1);
            cache.insert(format!("expr{i}"), shared);
        }

        // Recent cache should be at limit
        assert!(cache.recent_cache.len() <= 2);
        assert!(
            cache
                .stats()
                .evictions
                .load(std::sync::atomic::Ordering::Relaxed)
                > 0
        );
    }
}
