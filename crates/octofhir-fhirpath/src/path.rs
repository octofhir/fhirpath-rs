//! Canonical path representation for FHIRPath evaluation
//! 
//! This module provides efficient path tracking during evaluation, supporting
//! property navigation, array indexing, and path composition for metadata propagation.

use std::fmt;
use std::hash::{Hash, Hasher};

/// A segment in a canonical path
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum PathSegment {
    /// Root resource type (e.g., "Patient", "Observation")
    Root(String),
    /// Property name (e.g., "name", "given", "family")
    Property(String),
    /// Array index (e.g., [0], [1])
    Index(usize),
    /// Wildcard for collection operations (e.g., [*])
    Wildcard,
}

impl PathSegment {
    /// Check if this segment represents an index
    pub fn is_index(&self) -> bool {
        matches!(self, PathSegment::Index(_) | PathSegment::Wildcard)
    }
    
    /// Check if this segment represents a property
    pub fn is_property(&self) -> bool {
        matches!(self, PathSegment::Property(_))
    }
    
    /// Check if this segment represents a root
    pub fn is_root(&self) -> bool {
        matches!(self, PathSegment::Root(_))
    }
    
    /// Get the property name if this is a property segment
    pub fn as_property(&self) -> Option<&str> {
        match self {
            PathSegment::Property(name) => Some(name),
            _ => None,
        }
    }
    
    /// Get the index if this is an index segment
    pub fn as_index(&self) -> Option<usize> {
        match self {
            PathSegment::Index(idx) => Some(*idx),
            _ => None,
        }
    }
}

impl fmt::Display for PathSegment {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PathSegment::Root(name) => write!(f, "{}", name),
            PathSegment::Property(name) => write!(f, "{}", name),
            PathSegment::Index(idx) => write!(f, "[{}]", idx),
            PathSegment::Wildcard => write!(f, "[*]"),
        }
    }
}

/// Canonical path representation optimized for common patterns
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CanonicalPath {
    segments: Vec<PathSegment>,
    // Cache the string representation for performance
    cached_string: Option<String>,
}

impl CanonicalPath {
    /// Create a new path with a root segment
    pub fn root(resource_type: impl Into<String>) -> Self {
        Self {
            segments: vec![PathSegment::Root(resource_type.into())],
            cached_string: None,
        }
    }
    
    /// Create an empty path (used for temporary operations)
    pub fn empty() -> Self {
        Self {
            segments: Vec::new(),
            cached_string: Some(String::new()),
        }
    }
    
    /// Parse a path from string representation
    /// Examples: "Patient", "Patient.name", "Patient.name[0].given"
    pub fn parse(path_str: &str) -> Result<Self, PathParseError> {
        if path_str.is_empty() {
            return Ok(Self::empty());
        }
        
        let mut segments = Vec::new();
        let mut chars = path_str.chars().peekable();
        let mut current_segment = String::new();
        let mut is_first_segment = true;
        
        while let Some(ch) = chars.next() {
            match ch {
                '.' => {
                    if !current_segment.is_empty() {
                        if is_first_segment {
                            segments.push(PathSegment::Root(current_segment.clone()));
                            is_first_segment = false;
                        } else {
                            segments.push(PathSegment::Property(current_segment.clone()));
                        }
                        current_segment.clear();
                    }
                }
                '[' => {
                    // Handle property before index
                    if !current_segment.is_empty() {
                        if is_first_segment {
                            segments.push(PathSegment::Root(current_segment.clone()));
                            is_first_segment = false;
                        } else {
                            segments.push(PathSegment::Property(current_segment.clone()));
                        }
                        current_segment.clear();
                    }
                    
                    // Parse index content
                    let mut index_content = String::new();
                    while let Some(index_ch) = chars.next() {
                        if index_ch == ']' {
                            break;
                        }
                        index_content.push(index_ch);
                    }
                    
                    if index_content == "*" {
                        segments.push(PathSegment::Wildcard);
                    } else if let Ok(index) = index_content.parse::<usize>() {
                        segments.push(PathSegment::Index(index));
                    } else {
                        return Err(PathParseError::InvalidIndex(index_content));
                    }
                }
                _ => {
                    current_segment.push(ch);
                }
            }
        }
        
        // Handle final segment
        if !current_segment.is_empty() {
            if is_first_segment {
                segments.push(PathSegment::Root(current_segment));
            } else {
                segments.push(PathSegment::Property(current_segment));
            }
        }
        
        Ok(Self {
            segments,
            cached_string: Some(path_str.to_string()),
        })
    }
    
    /// Append a property segment to the path
    pub fn append_property(&self, property: impl Into<String>) -> Self {
        let mut new_segments = self.segments.clone();
        new_segments.push(PathSegment::Property(property.into()));
        Self {
            segments: new_segments,
            cached_string: None,
        }
    }
    
    /// Append an index segment to the path
    pub fn append_index(&self, index: usize) -> Self {
        let mut new_segments = self.segments.clone();
        new_segments.push(PathSegment::Index(index));
        Self {
            segments: new_segments,
            cached_string: None,
        }
    }
    
    /// Append a wildcard segment to the path (for collection operations)
    pub fn append_wildcard(&self) -> Self {
        let mut new_segments = self.segments.clone();
        new_segments.push(PathSegment::Wildcard);
        Self {
            segments: new_segments,
            cached_string: None,
        }
    }
    
    /// Get all segments
    pub fn segments(&self) -> &[PathSegment] {
        &self.segments
    }
    
    /// Get the root segment if present
    pub fn root_name(&self) -> Option<&str> {
        self.segments.first().and_then(|seg| match seg {
            PathSegment::Root(name) => Some(name.as_str()),
            _ => None,
        })
    }
    
    /// Get the last segment
    pub fn last_segment(&self) -> Option<&PathSegment> {
        self.segments.last()
    }
    
    /// Check if path is empty
    pub fn is_empty(&self) -> bool {
        self.segments.is_empty()
    }
    
    /// Get path depth (number of segments)
    pub fn depth(&self) -> usize {
        self.segments.len()
    }
    
    /// Check if this path is a parent of another path
    pub fn is_parent_of(&self, other: &CanonicalPath) -> bool {
        if self.segments.len() >= other.segments.len() {
            return false;
        }
        
        self.segments.iter()
            .zip(other.segments.iter())
            .all(|(a, b)| a == b)
    }
    
    /// Get the parent path (removing the last segment)
    pub fn parent(&self) -> Option<Self> {
        if self.segments.len() <= 1 {
            None
        } else {
            let mut parent_segments = self.segments.clone();
            parent_segments.pop();
            Some(Self {
                segments: parent_segments,
                cached_string: None,
            })
        }
    }
    
    /// Create a path for indexed access (replace wildcards with specific index)
    pub fn with_index(&self, index: usize) -> Self {
        let new_segments = self.segments.iter()
            .map(|seg| match seg {
                PathSegment::Wildcard => PathSegment::Index(index),
                other => other.clone(),
            })
            .collect();
        
        Self {
            segments: new_segments,
            cached_string: None,
        }
    }
}

impl fmt::Display for CanonicalPath {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Use cached string if available
        if let Some(ref cached) = self.cached_string {
            return write!(f, "{}", cached);
        }
        
        // Build string representation
        let mut result = String::new();
        for (i, segment) in self.segments.iter().enumerate() {
            match segment {
                PathSegment::Root(name) => {
                    result.push_str(name);
                }
                PathSegment::Property(name) => {
                    if i > 0 {
                        result.push('.');
                    }
                    result.push_str(name);
                }
                PathSegment::Index(idx) => {
                    result.push_str(&format!("[{}]", idx));
                }
                PathSegment::Wildcard => {
                    result.push_str("[*]");
                }
            }
        }
        
        write!(f, "{}", result)
    }
}

impl Hash for CanonicalPath {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.segments.hash(state);
    }
}

/// Error type for path parsing failures
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PathParseError {
    InvalidIndex(String),
    InvalidFormat(String),
}

impl fmt::Display for PathParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PathParseError::InvalidIndex(idx) => write!(f, "Invalid index: {}", idx),
            PathParseError::InvalidFormat(msg) => write!(f, "Invalid path format: {}", msg),
        }
    }
}

impl std::error::Error for PathParseError {}

/// Builder for constructing paths fluently
#[derive(Debug, Clone)]
pub struct PathBuilder {
    path: CanonicalPath,
}

impl PathBuilder {
    /// Start building a path with a root
    pub fn root(resource_type: impl Into<String>) -> Self {
        Self {
            path: CanonicalPath::root(resource_type),
        }
    }
    
    /// Start with an empty path
    pub fn empty() -> Self {
        Self {
            path: CanonicalPath::empty(),
        }
    }
    
    /// Add a property segment
    pub fn property(mut self, name: impl Into<String>) -> Self {
        self.path = self.path.append_property(name);
        self
    }
    
    /// Add an index segment
    pub fn index(mut self, index: usize) -> Self {
        self.path = self.path.append_index(index);
        self
    }
    
    /// Add a wildcard segment
    pub fn wildcard(mut self) -> Self {
        self.path = self.path.append_wildcard();
        self
    }
    
    /// Build the final path
    pub fn build(self) -> CanonicalPath {
        self.path
    }
}

/// Utility functions for common path operations
pub mod path_utils {
    use super::*;
    
    /// Create a path for resource root
    pub fn resource_root(resource_type: &str) -> CanonicalPath {
        CanonicalPath::root(resource_type)
    }
    
    /// Create a property path
    pub fn property_path(root: &str, property: &str) -> CanonicalPath {
        CanonicalPath::root(root).append_property(property)
    }
    
    /// Create an indexed path
    pub fn indexed_path(root: &str, property: &str, index: usize) -> CanonicalPath {
        CanonicalPath::root(root)
            .append_property(property)
            .append_index(index)
    }
    
    /// Check if a path represents an array element
    pub fn is_array_element(path: &CanonicalPath) -> bool {
        path.last_segment()
            .map(|seg| seg.is_index())
            .unwrap_or(false)
    }
    
    /// Get the array index if path represents an array element
    pub fn get_array_index(path: &CanonicalPath) -> Option<usize> {
        path.last_segment()
            .and_then(|seg| seg.as_index())
    }
    
    /// Convert a path to property-only (remove indices for type resolution)
    pub fn to_property_path(path: &CanonicalPath) -> CanonicalPath {
        let property_segments: Vec<PathSegment> = path.segments()
            .iter()
            .filter(|seg| !seg.is_index())
            .cloned()
            .collect();
        
        CanonicalPath {
            segments: property_segments,
            cached_string: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_path_creation_and_display() {
        let root_path = CanonicalPath::root("Patient");
        assert_eq!(root_path.to_string(), "Patient");
        assert_eq!(root_path.root_name(), Some("Patient"));
        
        let property_path = root_path.append_property("name");
        assert_eq!(property_path.to_string(), "Patient.name");
        
        let indexed_path = property_path.append_index(0);
        assert_eq!(indexed_path.to_string(), "Patient.name[0]");
        
        let deep_path = indexed_path.append_property("given").append_index(1);
        assert_eq!(deep_path.to_string(), "Patient.name[0].given[1]");
    }
    
    #[test]
    fn test_path_parsing() {
        let simple = CanonicalPath::parse("Patient").unwrap();
        assert_eq!(simple.segments().len(), 1);
        assert_eq!(simple.root_name(), Some("Patient"));
        
        let property = CanonicalPath::parse("Patient.name").unwrap();
        assert_eq!(property.to_string(), "Patient.name");
        
        let indexed = CanonicalPath::parse("Patient.name[0]").unwrap();
        assert_eq!(indexed.to_string(), "Patient.name[0]");
        
        let complex = CanonicalPath::parse("Patient.name[0].given[1]").unwrap();
        assert_eq!(complex.to_string(), "Patient.name[0].given[1]");
        assert_eq!(complex.depth(), 4);
    }
    
    #[test]
    fn test_path_builder() {
        let path = PathBuilder::root("Patient")
            .property("name")
            .index(0)
            .property("given")
            .index(1)
            .build();
        
        assert_eq!(path.to_string(), "Patient.name[0].given[1]");
    }
    
    #[test]
    fn test_path_relationships() {
        let parent = CanonicalPath::parse("Patient.name").unwrap();
        let child = CanonicalPath::parse("Patient.name[0].given").unwrap();
        
        assert!(parent.is_parent_of(&child));
        assert!(!child.is_parent_of(&parent));
        
        let child_parent = child.parent().unwrap();
        assert_eq!(child_parent.to_string(), "Patient.name[0]");
    }
    
    #[test]
    fn test_path_utilities() {
        let path = path_utils::indexed_path("Patient", "name", 0);
        assert_eq!(path.to_string(), "Patient.name[0]");
        
        assert!(path_utils::is_array_element(&path));
        assert_eq!(path_utils::get_array_index(&path), Some(0));
        
        let property_only = path_utils::to_property_path(&path);
        assert_eq!(property_only.to_string(), "Patient.name");
    }
    
    #[test]
    fn test_wildcard_paths() {
        let wildcard_path = CanonicalPath::root("Patient")
            .append_property("name")
            .append_wildcard();
        assert_eq!(wildcard_path.to_string(), "Patient.name[*]");
        
        let indexed_path = wildcard_path.with_index(0);
        assert_eq!(indexed_path.to_string(), "Patient.name[0]");
    }
    
    #[test]
    fn test_error_cases() {
        assert!(CanonicalPath::parse("Patient.name[invalid]").is_err());
    }
}