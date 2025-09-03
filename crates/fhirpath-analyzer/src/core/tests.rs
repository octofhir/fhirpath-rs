//! Unit tests for core analyzer components

#[cfg(test)]
mod tests {
    use crate::providers::MockFhirProvider;
    use crate::core::analyzer_engine::FhirPathAnalyzer;
    use crate::core::type_system::{FhirType, PrimitiveType, TypeInformation, TypeSystem};
    use crate::core::symbol_table::{SymbolTable, ScopeType};
    use crate::core::context::{AnalysisContext, AnalysisPhase};
    use std::sync::Arc;

    #[tokio::test]
    async fn test_analyzer_creation() {
        let provider = Arc::new(MockFhirProvider::new());
        let analyzer = FhirPathAnalyzer::new(provider);
        
        // Verify analyzer can be created
        assert_eq!(analyzer.config().enable_type_checking, true);
        assert_eq!(analyzer.config().enable_property_validation, true);
    }
    
    #[tokio::test]
    async fn test_type_system_creation() {
        let provider = Arc::new(MockFhirProvider::new());
        let type_system = TypeSystem::new(provider as Arc<dyn crate::providers::fhir_provider::FhirProvider>);
        
        // Test basic type compatibility
        let primitive_bool = FhirType::Primitive(PrimitiveType::Boolean);
        let primitive_string = FhirType::Primitive(PrimitiveType::String);
        let unknown_type = FhirType::Unknown;
        
        assert!(type_system.is_compatible(&primitive_bool, &primitive_bool));
        assert!(!type_system.is_compatible(&primitive_bool, &primitive_string));
        assert!(type_system.is_compatible(&unknown_type, &primitive_bool));
    }
    
    #[test]
    fn test_symbol_table_scoping() {
        use octofhir_fhirpath_diagnostics::{SourceLocation, Span, Position};
        use crate::core::symbol_table::{SymbolTable, ScopeType};
        use crate::core::type_system::{TypeInformation, FhirType};
        
        let mut symbol_table = SymbolTable::new();
        
        // Test scope creation
        let location = SourceLocation::new(Span::new(Position::new(0, 0), Position::new(0, 0)));
        let scope_id = symbol_table.push_scope(ScopeType::Function, location.clone());
        assert_eq!(scope_id, 1); // Global scope is 0, this should be 1
        
        // Test variable definition
        let type_info = TypeInformation::new(
            FhirType::Primitive(PrimitiveType::String),
            location.clone()
        );
        
        symbol_table.define_variable(
            "test_var".to_string(),
            type_info,
            false,
            location
        ).expect("Should be able to define variable");
        
        // Test variable resolution
        let resolved = symbol_table.resolve_variable("test_var");
        assert!(resolved.is_some());
        assert_eq!(resolved.unwrap().name, "test_var");
    }
    
    #[test]
    fn test_analysis_context() {
        use crate::core::context::{AnalysisContext, AnalysisPhase};
        
        let mut context = AnalysisContext::new();
        
        // Test phase transitions
        context.set_phase(AnalysisPhase::PropertyResolution);
        assert_eq!(context.analysis_phase, AnalysisPhase::PropertyResolution);
        
        // Test with root type
        let context_with_root = AnalysisContext::with_root_type("Patient".to_string());
        assert_eq!(context_with_root.root_resource_type, Some("Patient".to_string()));
    }
    
    #[tokio::test]
    async fn test_basic_expression_analysis() {
        let provider = Arc::new(MockFhirProvider::new());
        let mut analyzer = FhirPathAnalyzer::new(provider);
        
        // Test analyzing a simple expression
        let result = analyzer.analyze("true").await;
        assert!(result.is_ok());
        
        let analysis_result = result.unwrap();
        assert!(analysis_result.metadata.completed);
        assert!(!analysis_result.has_errors());
    }
    
    #[test]
    fn test_cardinality_operations() {
        use crate::core::type_system::Cardinality;
        
        let single_cardinality = Cardinality { min: 1, max: Some(1) };
        let optional_cardinality = Cardinality { min: 0, max: Some(1) };
        let multiple_cardinality = Cardinality { min: 0, max: None };
        
        // Test default cardinality
        let default_card = Cardinality::default();
        assert_eq!(default_card.min, 0);
        assert_eq!(default_card.max, Some(1));
        
        // Test various cardinalities
        assert_eq!(single_cardinality.min, 1);
        assert_eq!(optional_cardinality.min, 0);
        assert_eq!(multiple_cardinality.max, None);
    }
}