//! Function builder pattern for easy registration

use std::sync::Arc;
use super::{FunctionRegistry, FunctionMetadata, ParameterMetadata, FunctionCategory, SyncFunction, AsyncFunction};
use crate::core::{FhirPathError, Result};

pub struct FunctionBuilder {
    name: String,
    category: FunctionCategory,
    description: String,
    parameters: Vec<ParameterMetadata>,
    return_type: Option<String>,
    examples: Vec<String>,
}

impl FunctionBuilder {
    pub fn new(name: impl Into<String>, category: FunctionCategory) -> Self {
        Self {
            name: name.into(),
            category,
            description: String::new(),
            parameters: Vec::new(),
            return_type: None,
            examples: Vec::new(),
        }
    }

    pub fn description(mut self, description: impl Into<String>) -> Self {
        self.description = description.into();
        self
    }

    pub fn parameter(
        mut self,
        name: impl Into<String>,
        type_constraint: Option<String>,
        is_optional: bool,
        description: impl Into<String>,
    ) -> Self {
        self.parameters.push(ParameterMetadata {
            name: name.into(),
            type_constraint,
            is_optional,
            description: description.into(),
        });
        self
    }

    pub fn return_type(mut self, return_type: impl Into<String>) -> Self {
        self.return_type = Some(return_type.into());
        self
    }

    pub fn example(mut self, example: impl Into<String>) -> Self {
        self.examples.push(example.into());
        self
    }

    pub fn register_sync(
        self,
        registry: &FunctionRegistry,
        function: SyncFunction,
    ) -> Result<()> {
        let metadata = FunctionMetadata {
            name: self.name.clone(),
            category: self.category,
            description: self.description,
            parameters: self.parameters,
            return_type: self.return_type,
            is_async: false,
            examples: self.examples,
        };
        
        registry.register_sync_function(self.name, function, metadata)
    }

    pub fn register_async(
        self,
        registry: &FunctionRegistry,
        function: AsyncFunction,
    ) -> Result<()> {
        let metadata = FunctionMetadata {
            name: self.name.clone(),
            category: self.category,
            description: self.description,
            parameters: self.parameters,
            return_type: self.return_type,
            is_async: true,
            examples: self.examples,
        };
        
        registry.register_async_function(self.name, function, metadata)
    }
}

/// Convenience macro for function registration
#[macro_export]
macro_rules! register_function {
    (
        $registry:expr,
        sync $name:literal,
        category: $category:expr,
        description: $desc:literal,
        parameters: [$($param_name:literal : $param_type:expr => $param_desc:literal),*],
        return_type: $return_type:expr,
        examples: [$($example:literal),*],
        implementation: $impl:expr
    ) => {
        {
            let mut builder = $crate::registry::builder::FunctionBuilder::new($name, $category)
                .description($desc)
                .return_type($return_type);
            
            $(
                builder = builder.parameter($param_name, $param_type, false, $param_desc);
            )*
            
            $(
                builder = builder.example($example);
            )*
            
            builder.register_sync($registry, std::sync::Arc::new($impl))
        }
    };
    
    (
        $registry:expr,
        async $name:literal,
        category: $category:expr,
        description: $desc:literal,
        parameters: [$($param_name:literal : $param_type:expr => $param_desc:literal),*],
        return_type: $return_type:expr,
        examples: [$($example:literal),*],
        implementation: $impl:expr
    ) => {
        {
            let mut builder = $crate::registry::builder::FunctionBuilder::new($name, $category)
                .description($desc)
                .return_type($return_type);
            
            $(
                builder = builder.parameter($param_name, $param_type, false, $param_desc);
            )*
            
            $(
                builder = builder.example($example);
            )*
            
            builder.register_async($registry, std::sync::Arc::new($impl))
        }
    };
}