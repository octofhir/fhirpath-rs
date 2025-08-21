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

//! Background schema loading system for eliminating startup delays

use crate::cache::{CacheConfig, CacheManager};
use crate::loading_metrics::{LoadingMetricsCollector, LoadingMetricsSnapshot};
use crate::precomputed_registry::PrecomputedTypeRegistry;
use crate::priority_queue::{LoadPriority, LoadRequester, PriorityQueue, SchemaLoadRequest};
use crate::provider::{ElementInfo, TypeReflectionInfo};

use super::provider::ModelError;
use dashmap::DashMap;
use octofhir_fhirschema::{
    Element as FhirSchemaElement, FhirSchema, FhirSchemaPackageManager,
    ModelProvider as FhirSchemaModelProviderTrait,
};
use parking_lot::Mutex;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::task::JoinHandle;
use tokio::time::timeout;
use tracing::{debug, info, warn};

/// Configuration for background schema loading
#[derive(Debug, Clone)]
pub struct BackgroundLoadingConfig {
    /// Number of concurrent loading workers
    pub worker_count: usize,

    /// Essential types to load immediately
    pub essential_types: Vec<String>,

    /// Common types to prioritize
    pub common_types: Vec<String>,

    /// Maximum time to wait for essential types
    pub essential_timeout: Duration,

    /// Background loading batch size
    pub batch_size: usize,

    /// Enable predictive loading based on relationships
    pub enable_predictive_loading: bool,

    /// Retry configuration for failed loads
    pub retry_config: RetryConfig,

    /// Cache configuration
    pub cache_config: CacheConfig,
}

impl Default for BackgroundLoadingConfig {
    fn default() -> Self {
        Self {
            worker_count: 4,
            essential_types: vec![
                "Patient".to_string(),
                "Observation".to_string(),
                "Practitioner".to_string(),
                "Organization".to_string(),
                "Bundle".to_string(),
            ],
            common_types: vec![
                "HumanName".to_string(),
                "Address".to_string(),
                "ContactPoint".to_string(),
                "Identifier".to_string(),
                "CodeableConcept".to_string(),
                "Reference".to_string(),
                "Quantity".to_string(),
                "Period".to_string(),
                "Range".to_string(),
                "Meta".to_string(),
                "Narrative".to_string(),
            ],
            essential_timeout: Duration::from_secs(10),
            batch_size: 10,
            enable_predictive_loading: true,
            retry_config: RetryConfig::default(),
            cache_config: CacheConfig::default(),
        }
    }
}

/// Retry configuration for failed schema loads
#[derive(Debug, Clone)]
pub struct RetryConfig {
    /// Maximum number of retry attempts
    pub max_retries: usize,

    /// Base delay between retries
    pub base_delay: Duration,

    /// Exponential backoff multiplier
    pub backoff_multiplier: f64,

    /// Maximum retry delay
    pub max_delay: Duration,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_retries: 3,
            base_delay: Duration::from_millis(500),
            backoff_multiplier: 2.0,
            max_delay: Duration::from_secs(30),
        }
    }
}

/// State of a schema loading operation
#[derive(Debug)]
struct LoadingState {
    started_at: Instant,
    priority: LoadPriority,
    requester: LoadRequester,
    retry_count: usize,
    task_handle: Option<JoinHandle<Result<(), ModelError>>>,
}

/// Status information for ongoing loading operations
#[derive(Debug, Clone)]
pub struct LoadingStatus {
    pub essential_loaded: usize,
    pub essential_total: usize,
    pub total_loaded: u32,
    pub queue_length: usize,
    pub in_progress_count: usize,
    pub load_failures: u32,
    pub average_load_time: Duration,
    pub cache_hit_rate: f64,
    pub success_rate: f64,
}

/// Asynchronous background schema loading system
#[derive(Debug)]
pub struct BackgroundSchemaLoader {
    /// Package manager for schema operations
    package_manager: Arc<FhirSchemaPackageManager>,

    /// Cache for storing loaded schemas
    cache: Arc<CacheManager>,

    /// Pre-computed registry for essential types
    registry: Arc<PrecomputedTypeRegistry>,

    /// Loading queue for prioritized schema loading
    loading_queue: Arc<PriorityQueue<SchemaLoadRequest>>,

    /// Currently loading schemas (to avoid duplicates)
    loading_in_progress: Arc<DashMap<String, LoadingState>>,

    /// Background task handles
    task_handles: Arc<Mutex<Vec<JoinHandle<()>>>>,

    /// Loading statistics and metrics
    metrics: Arc<LoadingMetricsCollector>,

    /// Configuration
    config: BackgroundLoadingConfig,

    /// Shutdown flag
    shutdown_flag: Arc<parking_lot::RwLock<bool>>,
}

impl BackgroundSchemaLoader {
    /// Create new background loader
    pub async fn new(
        package_manager: Arc<FhirSchemaPackageManager>,
        cache: Arc<CacheManager>,
        registry: Arc<PrecomputedTypeRegistry>,
        config: BackgroundLoadingConfig,
    ) -> Result<Self, ModelError> {
        let loader = Self {
            package_manager,
            cache,
            registry,
            loading_queue: Arc::new(PriorityQueue::new()),
            loading_in_progress: Arc::new(DashMap::new()),
            task_handles: Arc::new(Mutex::new(Vec::new())),
            metrics: Arc::new(LoadingMetricsCollector::new()),
            config,
            shutdown_flag: Arc::new(parking_lot::RwLock::new(false)),
        };

        // Start background workers
        loader.start_workers().await;

        // Queue essential types for immediate loading
        loader.queue_essential_types();

        info!(
            "Background schema loader started with {} workers",
            loader.config.worker_count
        );

        Ok(loader)
    }

    /// Start background worker tasks
    async fn start_workers(&self) {
        let mut handles = self.task_handles.lock();

        for worker_id in 0..self.config.worker_count {
            let worker = BackgroundWorker::new(
                worker_id,
                self.package_manager.clone(),
                self.cache.clone(),
                self.registry.clone(),
                self.loading_queue.clone(),
                self.loading_in_progress.clone(),
                self.metrics.clone(),
                self.config.retry_config.clone(),
                self.shutdown_flag.clone(),
            );

            let handle = tokio::spawn(async move {
                worker.run().await;
            });

            handles.push(handle);
        }

        info!("Started {} background workers", self.config.worker_count);
    }

    /// Queue essential types for immediate loading
    fn queue_essential_types(&self) {
        for type_name in &self.config.essential_types {
            self.request_load(
                type_name.clone(),
                LoadPriority::Essential,
                LoadRequester::Initialization,
            );
        }

        for type_name in &self.config.common_types {
            self.request_load(
                type_name.clone(),
                LoadPriority::Common,
                LoadRequester::Initialization,
            );
        }

        debug!(
            "Queued {} essential and {} common types for loading",
            self.config.essential_types.len(),
            self.config.common_types.len()
        );
    }

    /// Request loading of a specific type
    pub fn request_load(
        &self,
        type_name: String,
        priority: LoadPriority,
        requester: LoadRequester,
    ) {
        // Check if shutting down
        if *self.shutdown_flag.read() {
            return;
        }

        // Check if already loaded in cache
        if self.cache.get(&type_name).is_some() {
            self.metrics.record_cache_hit();
            return;
        }

        // Check if already in progress
        if self.loading_in_progress.contains_key(&type_name) {
            return;
        }

        // Mark as in progress
        self.loading_in_progress.insert(
            type_name.clone(),
            LoadingState {
                started_at: Instant::now(),
                priority,
                requester: requester.clone(),
                retry_count: 0,
                task_handle: None,
            },
        );

        // Queue for loading
        let request = SchemaLoadRequest {
            type_name,
            priority,
            requested_at: Instant::now(),
            requester,
        };

        self.loading_queue.push(request, priority);

        // Update peak queue length metric
        self.metrics
            .update_peak_queue_length(self.loading_queue.len());
    }

    /// Wait for essential types to be loaded
    pub async fn wait_for_essential_types(&self) -> Result<(), ModelError> {
        let start_time = Instant::now();
        let timeout_duration = self.config.essential_timeout;

        info!(
            "Waiting for {} essential types to load (timeout: {:?})",
            self.config.essential_types.len(),
            timeout_duration
        );

        while start_time.elapsed() < timeout_duration {
            let loaded_count = self
                .config
                .essential_types
                .iter()
                .filter(|type_name| self.cache.get(type_name).is_some())
                .count();

            if loaded_count == self.config.essential_types.len() {
                let load_time = start_time.elapsed();
                self.metrics.set_essential_load_time(load_time);
                info!(
                    "All {} essential types loaded in {:?}",
                    loaded_count, load_time
                );
                return Ok(());
            }

            tokio::time::sleep(Duration::from_millis(100)).await;
        }

        // Timeout - return error with status
        let loaded_count = self
            .config
            .essential_types
            .iter()
            .filter(|type_name| self.cache.get(type_name).is_some())
            .count();

        warn!(
            "Essential types timeout: loaded {}/{} in {:?}",
            loaded_count,
            self.config.essential_types.len(),
            timeout_duration
        );

        Err(ModelError::schema_load_error(format!(
            "Timeout waiting for essential types. Loaded {}/{} types in {:?}",
            loaded_count,
            self.config.essential_types.len(),
            timeout_duration
        )))
    }

    /// Get loading status
    pub fn get_loading_status(&self) -> LoadingStatus {
        let metrics = self.metrics.snapshot();
        let queue_length = self.loading_queue.len();
        let in_progress_count = self.loading_in_progress.len();

        LoadingStatus {
            essential_loaded: self
                .config
                .essential_types
                .iter()
                .filter(|t| self.cache.get(t).is_some())
                .count(),
            essential_total: self.config.essential_types.len(),
            total_loaded: metrics.total_loaded,
            queue_length,
            in_progress_count,
            load_failures: metrics.load_failures,
            average_load_time: metrics.average_load_time,
            cache_hit_rate: metrics.cache_hit_rate,
            success_rate: metrics.success_rate,
        }
    }

    /// Trigger predictive loading based on accessed type
    pub async fn trigger_predictive_loading(&self, accessed_type: &str) {
        if !self.config.enable_predictive_loading {
            return;
        }

        let related_types = self.get_predictive_types(accessed_type);

        for related_type in &related_types {
            self.request_load(
                related_type.clone(),
                LoadPriority::Predictive,
                LoadRequester::PredictiveSystem,
            );
        }

        if !related_types.is_empty() {
            debug!(
                "Triggered predictive loading for {} related to '{}'",
                related_types.len(),
                accessed_type
            );
        }
    }

    /// Get types that should be predictively loaded
    fn get_predictive_types(&self, accessed_type: &str) -> Vec<String> {
        // Common type relationships for predictive loading
        match accessed_type {
            "Patient" => vec![
                "HumanName".to_string(),
                "Address".to_string(),
                "ContactPoint".to_string(),
                "Identifier".to_string(),
            ],
            "Observation" => vec![
                "Quantity".to_string(),
                "CodeableConcept".to_string(),
                "Reference".to_string(),
                "Period".to_string(),
                "Range".to_string(),
            ],
            "Bundle" => vec![
                "BundleEntry".to_string(),
                "Meta".to_string(),
                "Narrative".to_string(),
            ],
            "Practitioner" => vec![
                "HumanName".to_string(),
                "ContactPoint".to_string(),
                "Address".to_string(),
                "Identifier".to_string(),
            ],
            "Organization" => vec![
                "ContactPoint".to_string(),
                "Address".to_string(),
                "Reference".to_string(),
                "Identifier".to_string(),
            ],
            "DiagnosticReport" => vec![
                "Reference".to_string(),
                "CodeableConcept".to_string(),
                "Attachment".to_string(),
                "Period".to_string(),
            ],
            "Procedure" => vec![
                "Reference".to_string(),
                "CodeableConcept".to_string(),
                "Period".to_string(),
            ],
            "Encounter" => vec![
                "Reference".to_string(),
                "Period".to_string(),
                "Coding".to_string(),
            ],
            _ => vec![],
        }
    }

    /// Get metrics snapshot
    pub fn get_metrics(&self) -> LoadingMetricsSnapshot {
        self.metrics.snapshot()
    }

    /// Check if schema loader is healthy
    pub fn is_healthy(&self) -> bool {
        !*self.shutdown_flag.read()
    }

    /// Shutdown background loading
    pub async fn shutdown(&self) {
        info!("Shutting down background schema loader");

        // Set shutdown flag
        *self.shutdown_flag.write() = true;

        // Cancel all background tasks
        let mut handles = self.task_handles.lock();
        for handle in handles.drain(..) {
            handle.abort();
        }

        // Clear loading state
        self.loading_in_progress.clear();

        // Clear queue
        self.loading_queue.clear();

        info!("Background schema loader shutdown complete");
    }
}

/// Individual background worker for schema loading
#[derive(Debug)]
struct BackgroundWorker {
    worker_id: usize,
    package_manager: Arc<FhirSchemaPackageManager>,
    cache: Arc<CacheManager>,
    registry: Arc<PrecomputedTypeRegistry>,
    loading_queue: Arc<PriorityQueue<SchemaLoadRequest>>,
    loading_in_progress: Arc<DashMap<String, LoadingState>>,
    metrics: Arc<LoadingMetricsCollector>,
    retry_config: RetryConfig,
    shutdown_flag: Arc<parking_lot::RwLock<bool>>,
}

impl BackgroundWorker {
    pub fn new(
        worker_id: usize,
        package_manager: Arc<FhirSchemaPackageManager>,
        cache: Arc<CacheManager>,
        registry: Arc<PrecomputedTypeRegistry>,
        loading_queue: Arc<PriorityQueue<SchemaLoadRequest>>,
        loading_in_progress: Arc<DashMap<String, LoadingState>>,
        metrics: Arc<LoadingMetricsCollector>,
        retry_config: RetryConfig,
        shutdown_flag: Arc<parking_lot::RwLock<bool>>,
    ) -> Self {
        Self {
            worker_id,
            package_manager,
            cache,
            registry,
            loading_queue,
            loading_in_progress,
            metrics,
            retry_config,
            shutdown_flag,
        }
    }

    /// Main worker loop
    pub async fn run(&self) {
        info!("Background worker {} started", self.worker_id);

        while !*self.shutdown_flag.read() {
            // Try to get a request without blocking for too long
            let request = match timeout(Duration::from_millis(1000), self.loading_queue.pop()).await
            {
                Ok(Some(request)) => request,
                Ok(None) => continue, // Shouldn't happen with current implementation
                Err(_) => continue,   // Timeout - check shutdown flag
            };

            let start_time = Instant::now();

            match self.load_schema_with_retries(&request).await {
                Ok(type_info) => {
                    // Store in cache
                    self.cache
                        .put(request.type_name.clone(), Arc::new(type_info));

                    // Update metrics
                    let load_time = start_time.elapsed();
                    match request.requester {
                        LoadRequester::PredictiveSystem => {
                            self.metrics.record_predictive_load(load_time);
                        }
                        _ => {
                            self.metrics.record_success(load_time);
                        }
                    }

                    debug!(
                        "Worker {} loaded {} in {:?} (priority: {:?})",
                        self.worker_id, request.type_name, load_time, request.priority
                    );
                }
                Err(error) => {
                    self.metrics.record_failure();
                    warn!(
                        "Worker {} failed to load {} after retries: {}",
                        self.worker_id, request.type_name, error
                    );
                }
            }

            // Remove from in-progress tracking
            self.loading_in_progress.remove(&request.type_name);
        }

        info!("Background worker {} stopped", self.worker_id);
    }

    /// Load schema with retry logic
    async fn load_schema_with_retries(
        &self,
        request: &SchemaLoadRequest,
    ) -> Result<TypeReflectionInfo, ModelError> {
        let mut attempt = 0;
        let mut delay = self.retry_config.base_delay;

        loop {
            match self.load_schema(request).await {
                Ok(type_info) => return Ok(type_info),
                Err(error) => {
                    attempt += 1;

                    if attempt > self.retry_config.max_retries {
                        return Err(error);
                    }

                    self.metrics.record_retry();

                    debug!(
                        "Worker {} retrying {} (attempt {}/{}) after {:?}",
                        self.worker_id,
                        request.type_name,
                        attempt,
                        self.retry_config.max_retries,
                        delay
                    );

                    tokio::time::sleep(delay).await;

                    // Exponential backoff
                    delay = std::cmp::min(
                        Duration::from_millis(
                            (delay.as_millis() as f64 * self.retry_config.backoff_multiplier)
                                as u64,
                        ),
                        self.retry_config.max_delay,
                    );
                }
            }
        }
    }

    /// Load a single schema
    async fn load_schema(
        &self,
        request: &SchemaLoadRequest,
    ) -> Result<TypeReflectionInfo, ModelError> {
        let canonical_url = format!(
            "http://hl7.org/fhir/StructureDefinition/{}",
            request.type_name
        );

        // Try to get from package manager with timeout
        let schema = timeout(
            Duration::from_secs(30), // Per-schema timeout
            self.package_manager.get_schema(&canonical_url),
        )
        .await
        .map_err(|_| {
            ModelError::schema_load_error(format!(
                "Timeout loading schema for {}",
                request.type_name
            ))
        })?;

        if let Some(schema) = schema {
            // Convert schema to TypeReflectionInfo
            self.convert_schema_to_type_info(&request.type_name, &schema)
        } else {
            Err(ModelError::schema_load_error(format!(
                "Schema not found for type {}",
                request.type_name
            )))
        }
    }

    /// Convert FhirSchema to TypeReflectionInfo
    fn convert_schema_to_type_info(
        &self,
        type_name: &str,
        schema: &FhirSchema,
    ) -> Result<TypeReflectionInfo, ModelError> {
        // Check if this is a System type
        if self.is_system_type(type_name) {
            return Ok(TypeReflectionInfo::SimpleType {
                namespace: "System".to_string(),
                name: type_name.to_string(),
                base_type: None,
            });
        }

        // Extract elements for FHIR types
        let elements: Vec<ElementInfo> = schema
            .elements
            .iter()
            .filter_map(|(path, element)| {
                if let Some(element_name) = path.strip_prefix(&format!("{type_name}.")) {
                    // Only include direct children (no nested paths)
                    if !element_name.contains('.') {
                        self.convert_element_to_info(element_name, element).ok()
                    } else {
                        None
                    }
                } else {
                    None
                }
            })
            .collect();

        Ok(TypeReflectionInfo::ClassInfo {
            namespace: "FHIR".to_string(),
            name: type_name.to_string(),
            base_type: self.extract_base_type(schema),
            elements,
        })
    }

    /// Check if type is a System primitive type
    fn is_system_type(&self, type_name: &str) -> bool {
        matches!(
            type_name,
            "Boolean"
                | "Integer"
                | "String"
                | "Decimal"
                | "Date"
                | "DateTime"
                | "Time"
                | "Quantity"
        )
    }

    /// Extract base type from schema
    fn extract_base_type(&self, schema: &FhirSchema) -> Option<String> {
        schema
            .base_definition
            .as_ref()
            .and_then(|url| url.path_segments())
            .and_then(|mut segments| segments.next_back())
            .map(|s| s.to_string())
    }

    /// Convert FhirSchema element to ElementInfo
    fn convert_element_to_info(
        &self,
        element_name: &str,
        element: &FhirSchemaElement,
    ) -> Result<ElementInfo, ModelError> {
        // Extract type information
        let type_info = if let Some(types) = &element.element_type {
            if types.len() == 1 {
                let element_type = &types[0];
                TypeReflectionInfo::SimpleType {
                    namespace: "FHIR".to_string(),
                    name: element_type.code.clone(),
                    base_type: None,
                }
            } else {
                // Choice type - return first option for simplicity
                // In a full implementation, this would be a ChoiceType
                TypeReflectionInfo::SimpleType {
                    namespace: "FHIR".to_string(),
                    name: types[0].code.clone(),
                    base_type: None,
                }
            }
        } else {
            TypeReflectionInfo::SimpleType {
                namespace: "FHIR".to_string(),
                name: "Element".to_string(),
                base_type: None,
            }
        };

        Ok(ElementInfo {
            name: element_name.to_string(),
            type_info,
            min_cardinality: element.min.unwrap_or(0),
            max_cardinality: element
                .max
                .as_ref()
                .and_then(|m| if m == "*" { None } else { m.parse().ok() }),
            is_modifier: element.is_modifier,
            is_summary: element.is_summary,
            documentation: element.definition.clone(),
        })
    }
}
