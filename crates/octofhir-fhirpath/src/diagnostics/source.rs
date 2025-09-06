//! Source text management for precise span tracking

use std::collections::HashMap;

/// Information about a source file or text
#[derive(Debug, Clone)]
pub struct SourceInfo {
    /// Name/identifier for the source (e.g., filename or description)
    pub name: String,
    /// Source text content
    pub content: String,
}

/// Manages source text and provides span mapping for diagnostics
#[derive(Debug, Default)]
pub struct SourceManager {
    /// Storage for source texts by ID
    sources: HashMap<usize, SourceInfo>,
    /// Next available source ID
    next_id: usize,
}

impl SourceManager {
    /// Create a new source manager
    pub fn new() -> Self {
        Self {
            sources: HashMap::new(),
            next_id: 0,
        }
    }

    /// Add source text and return its ID for reference
    pub fn add_source(&mut self, name: String, content: String) -> usize {
        let id = self.next_id;
        self.next_id += 1;

        self.sources.insert(id, SourceInfo { name, content });
        id
    }

    /// Get source information by ID
    pub fn get_source(&self, id: usize) -> Option<&SourceInfo> {
        self.sources.get(&id)
    }

    /// Get all sources as an iterator
    pub fn sources(&self) -> impl Iterator<Item = (usize, &SourceInfo)> {
        self.sources.iter().map(|(id, info)| (*id, info))
    }

    /// Check if a source ID exists
    pub fn has_source(&self, id: usize) -> bool {
        self.sources.contains_key(&id)
    }

    /// Get the number of sources managed
    pub fn source_count(&self) -> usize {
        self.sources.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_source_manager_basic_operations() {
        let mut manager = SourceManager::new();

        // Add first source
        let id1 = manager.add_source("test1.fhirpath".to_string(), "Patient.name".to_string());
        assert_eq!(id1, 0);

        // Add second source
        let id2 = manager.add_source("test2.fhirpath".to_string(), "age > 18".to_string());
        assert_eq!(id2, 1);

        // Verify sources can be retrieved
        let source1 = manager.get_source(id1).unwrap();
        assert_eq!(source1.name, "test1.fhirpath");
        assert_eq!(source1.content, "Patient.name");

        let source2 = manager.get_source(id2).unwrap();
        assert_eq!(source2.name, "test2.fhirpath");
        assert_eq!(source2.content, "age > 18");

        // Verify source count
        assert_eq!(manager.source_count(), 2);
        assert!(manager.has_source(id1));
        assert!(manager.has_source(id2));
        assert!(!manager.has_source(999));
    }

    #[test]
    fn test_source_manager_iteration() {
        let mut manager = SourceManager::new();

        manager.add_source("source1".to_string(), "content1".to_string());
        manager.add_source("source2".to_string(), "content2".to_string());

        let sources: Vec<_> = manager.sources().collect();
        assert_eq!(sources.len(), 2);

        // Check that all sources are present (order might vary due to HashMap)
        let names: Vec<&str> = sources.iter().map(|(_, info)| info.name.as_str()).collect();
        assert!(names.contains(&"source1"));
        assert!(names.contains(&"source2"));
    }
}
