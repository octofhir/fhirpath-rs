//! Function builder pattern for easy registration

use super::{
    AsyncFunction, FunctionCategory, FunctionMetadata, FunctionRegistry, ParameterMetadata,
    SyncFunction,
};
use crate::core::Result;

pub struct FunctionBuilder {
    name: String,
    category: FunctionCategory,
    description: String,
    parameters: Vec<ParameterMetadata>,
    return_type: Option<String>,
    examples: Vec<String>,
    requires_model_provider: bool,
    requires_terminology_provider: bool,
    does_not_propagate_empty: bool,
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
            requires_model_provider: false,
            requires_terminology_provider: false,
            does_not_propagate_empty: false,
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

    pub fn requires_model_provider(mut self) -> Self {
        self.requires_model_provider = true;
        self
    }

    pub fn requires_terminology_provider(mut self) -> Self {
        self.requires_terminology_provider = true;
        self
    }

    pub fn does_not_propagate_empty(mut self) -> Self {
        self.does_not_propagate_empty = true;
        self
    }

    pub fn register_sync(self, registry: &FunctionRegistry, function: SyncFunction) -> Result<()> {
        let metadata = FunctionMetadata {
            name: self.name.clone(),
            category: self.category,
            description: self.description,
            parameters: self.parameters,
            return_type: self.return_type,
            is_async: false,
            examples: self.examples,
            requires_model_provider: self.requires_model_provider,
            requires_terminology_provider: self.requires_terminology_provider,
            does_not_propagate_empty: self.does_not_propagate_empty,
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
            requires_model_provider: self.requires_model_provider,
            requires_terminology_provider: self.requires_terminology_provider,
            does_not_propagate_empty: self.does_not_propagate_empty,
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

    // Macro variant with provider requirements
    (
        $registry:expr,
        sync $name:literal,
        category: $category:expr,
        description: $desc:literal,
        parameters: [$($param_name:literal : $param_type:expr => $param_desc:literal),*],
        return_type: $return_type:expr,
        examples: [$($example:literal),*],
        requires_model_provider: $model_req:expr,
        requires_terminology_provider: $term_req:expr,
        implementation: $impl:expr
    ) => {
        {
            let mut builder = $crate::registry::builder::FunctionBuilder::new($name, $category)
                .description($desc)
                .return_type($return_type);

            if $model_req {
                builder = builder.requires_model_provider();
            }
            
            if $term_req {
                builder = builder.requires_terminology_provider();
            }

            $(
                builder = builder.parameter($param_name, $param_type, false, $param_desc);
            )*

            $(
                builder = builder.example($example);
            )*

            builder.register_sync($registry, std::sync::Arc::new($impl))
        }
    };
}
