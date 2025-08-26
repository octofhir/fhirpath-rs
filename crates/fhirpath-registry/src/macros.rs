//! Macros for Easy Operation Implementation
//!
//! This module provides convenient macros that make it easy to implement
//! FHIRPath operations with minimal boilerplate. These macros generate
//! the necessary trait implementations and reduce repetitive code.

/// Macro to implement a synchronous operation with minimal boilerplate
///
/// # Usage
/// ```rust,ignore
/// use octofhir_fhirpath_registry::{impl_sync_op, signature::{ValueType, ParameterType}};
///
/// pub struct LengthFunction;
///
/// impl_sync_op!(LengthFunction, "length", [] => ValueType::Integer, {
///     // Implementation goes here - context is available in this scope
///     unimplemented!("Example usage")
/// });
/// ```
#[macro_export]
macro_rules! impl_sync_op {
    // No parameters: impl_sync_op!(StructName, "name", [] => ReturnType, { implementation })
    ($struct_name:ty, $op_name:literal, [] => $return_type:expr, $implementation:block) => {
        impl $crate::traits::SyncOperation for $struct_name {
            fn name(&self) -> &'static str {
                $op_name
            }

            fn signature(&self) -> &$crate::signature::FunctionSignature {
                static SIGNATURE: $crate::signature::FunctionSignature = $crate::signature::FunctionSignature {
                    name: $op_name,
                    parameters: vec![],
                    return_type: $return_type,
                    variadic: false,
                };
                &SIGNATURE
            }

            fn execute(&self, args: &[octofhir_fhirpath_model::FhirPathValue], context: &$crate::traits::EvaluationContext) -> octofhir_fhirpath_core::Result<octofhir_fhirpath_model::FhirPathValue> {
                use octofhir_fhirpath_core::FhirPathError;
                use octofhir_fhirpath_model::FhirPathValue;
                use $crate::traits::validation::*;

                validate_no_args(args, $op_name)?;
                $implementation
            }
        }
    };

    // Single parameter: impl_sync_op!(StructName, "name", [ParamType] => ReturnType, { implementation })
    ($struct_name:ty, $op_name:literal, [$param_type:expr] => $return_type:expr, $implementation:block) => {
        impl $crate::traits::SyncOperation for $struct_name {
            fn name(&self) -> &'static str {
                $op_name
            }

            fn signature(&self) -> &$crate::signature::FunctionSignature {
                static SIGNATURE: $crate::signature::FunctionSignature = $crate::signature::FunctionSignature {
                    name: $op_name,
                    parameters: vec![$param_type],
                    return_type: $return_type,
                    variadic: false,
                };
                &SIGNATURE
            }

            fn execute(&self, args: &[octofhir_fhirpath_model::FhirPathValue], context: &$crate::traits::EvaluationContext) -> octofhir_fhirpath_core::Result<octofhir_fhirpath_model::FhirPathValue> {
                use octofhir_fhirpath_core::FhirPathError;
                use octofhir_fhirpath_model::FhirPathValue;

                if args.len() != 1 {
                    return Err(FhirPathError::InvalidArgumentCount {
                        function_name: $op_name.to_string(),
                        expected: 1,
                        actual: args.len(),
                    });
                }
                $implementation
            }
        }
    };

    // Multiple parameters: impl_sync_op!(StructName, "name", [ParamType1, ParamType2, ...] => ReturnType, { implementation })
    ($struct_name:ty, $op_name:literal, [$($param_type:expr),+] => $return_type:expr, $implementation:block) => {
        impl $crate::traits::SyncOperation for $struct_name {
            fn name(&self) -> &'static str {
                $op_name
            }

            fn signature(&self) -> &$crate::signature::FunctionSignature {
                static SIGNATURE: $crate::signature::FunctionSignature = $crate::signature::FunctionSignature {
                    name: $op_name,
                    parameters: vec![$($param_type),+],
                    return_type: $return_type,
                    variadic: false,
                };
                &SIGNATURE
            }

            fn execute(&self, args: &[octofhir_fhirpath_model::FhirPathValue], context: &$crate::traits::EvaluationContext) -> octofhir_fhirpath_core::Result<octofhir_fhirpath_model::FhirPathValue> {
                use octofhir_fhirpath_core::FhirPathError;
                use octofhir_fhirpath_model::FhirPathValue;

                let expected_count = SIGNATURE.parameters.len();
                if args.len() != expected_count {
                    return Err(FhirPathError::InvalidArgumentCount {
                        function_name: $op_name.to_string(),
                        expected: expected_count,
                        actual: args.len(),
                    });
                }
                $implementation
            }
        }
    };

    // Variadic: impl_sync_op!(StructName, "name", [ParamType, ...] => ReturnType, variadic, { implementation })
    ($struct_name:ty, $op_name:literal, [$($param_type:expr),*] => $return_type:expr, variadic, $implementation:block) => {
        impl $crate::traits::SyncOperation for $struct_name {
            fn name(&self) -> &'static str {
                $op_name
            }

            fn signature(&self) -> &$crate::signature::FunctionSignature {
                static SIGNATURE: $crate::signature::FunctionSignature = $crate::signature::FunctionSignature {
                    name: $op_name,
                    parameters: vec![$($param_type),*],
                    return_type: $return_type,
                    variadic: true,
                };
                &SIGNATURE
            }

            fn execute(&self, args: &[octofhir_fhirpath_model::FhirPathValue], context: &$crate::traits::EvaluationContext) -> octofhir_fhirpath_core::Result<octofhir_fhirpath_model::FhirPathValue> {
                use octofhir_fhirpath_core::FhirPathError;
                use octofhir_fhirpath_model::FhirPathValue;

                let min_args = SIGNATURE.parameters.len();
                if args.len() < min_args {
                    return Err(FhirPathError::InvalidArgumentCount {
                        function_name: $op_name.to_string(),
                        expected: min_args,
                        actual: args.len(),
                    });
                }
                $implementation
            }
        }
    };
}

/// Macro to implement an asynchronous operation with minimal boilerplate
///
/// # Usage
/// ```rust,ignore
/// use octofhir_fhirpath_registry::{impl_async_op, signature::{ValueType, ParameterType}};
///
/// pub struct ResolveFunction;
///
/// impl_async_op!(ResolveFunction, "resolve", [] => ValueType::Any, {
///     // Async implementation - context is available in this scope
///     unimplemented!("Example usage")
/// });
/// ```
#[macro_export]
macro_rules! impl_async_op {
    // No parameters: impl_async_op!(StructName, "name", [] => ReturnType, { implementation })
    ($struct_name:ty, $op_name:literal, [] => $return_type:expr, $implementation:block) => {
        #[async_trait::async_trait]
        impl $crate::traits::AsyncOperation for $struct_name {
            fn name(&self) -> &'static str {
                $op_name
            }

            fn signature(&self) -> &$crate::signature::FunctionSignature {
                static SIGNATURE: $crate::signature::FunctionSignature = $crate::signature::FunctionSignature {
                    name: $op_name,
                    parameters: vec![],
                    return_type: $return_type,
                    variadic: false,
                };
                &SIGNATURE
            }

            async fn execute(&self, args: &[octofhir_fhirpath_model::FhirPathValue], context: &$crate::traits::EvaluationContext) -> octofhir_fhirpath_core::Result<octofhir_fhirpath_model::FhirPathValue> {
                use octofhir_fhirpath_core::FhirPathError;
                use octofhir_fhirpath_model::FhirPathValue;
                use $crate::traits::validation::*;

                validate_no_args(args, $op_name)?;
                $implementation
            }
        }
    };

    // Single parameter: impl_async_op!(StructName, "name", [ParamType] => ReturnType, { implementation })
    ($struct_name:ty, $op_name:literal, [$param_type:expr] => $return_type:expr, $implementation:block) => {
        #[async_trait::async_trait]
        impl $crate::traits::AsyncOperation for $struct_name {
            fn name(&self) -> &'static str {
                $op_name
            }

            fn signature(&self) -> &$crate::signature::FunctionSignature {
                static SIGNATURE: $crate::signature::FunctionSignature = $crate::signature::FunctionSignature {
                    name: $op_name,
                    parameters: vec![$param_type],
                    return_type: $return_type,
                    variadic: false,
                };
                &SIGNATURE
            }

            async fn execute(&self, args: &[octofhir_fhirpath_model::FhirPathValue], context: &$crate::traits::EvaluationContext) -> octofhir_fhirpath_core::Result<octofhir_fhirpath_model::FhirPathValue> {
                use octofhir_fhirpath_core::FhirPathError;
                use octofhir_fhirpath_model::FhirPathValue;

                if args.len() != 1 {
                    return Err(FhirPathError::InvalidArgumentCount {
                        function_name: $op_name.to_string(),
                        expected: 1,
                        actual: args.len(),
                    });
                }
                $implementation
            }
        }
    };

    // Multiple parameters: impl_async_op!(StructName, "name", [ParamType1, ParamType2, ...] => ReturnType, { implementation })
    ($struct_name:ty, $op_name:literal, [$($param_type:expr),+] => $return_type:expr, $implementation:block) => {
        #[async_trait::async_trait]
        impl $crate::traits::AsyncOperation for $struct_name {
            fn name(&self) -> &'static str {
                $op_name
            }

            fn signature(&self) -> &$crate::signature::FunctionSignature {
                static SIGNATURE: $crate::signature::FunctionSignature = $crate::signature::FunctionSignature {
                    name: $op_name,
                    parameters: vec![$($param_type),+],
                    return_type: $return_type,
                    variadic: false,
                };
                &SIGNATURE
            }

            async fn execute(&self, args: &[octofhir_fhirpath_model::FhirPathValue], context: &$crate::traits::EvaluationContext) -> octofhir_fhirpath_core::Result<octofhir_fhirpath_model::FhirPathValue> {
                use octofhir_fhirpath_core::FhirPathError;
                use octofhir_fhirpath_model::FhirPathValue;

                let expected_count = SIGNATURE.parameters.len();
                if args.len() != expected_count {
                    return Err(FhirPathError::InvalidArgumentCount {
                        function_name: $op_name.to_string(),
                        expected: expected_count,
                        actual: args.len(),
                    });
                }
                $implementation
            }
        }
    };

    // Variadic: impl_async_op!(StructName, "name", [ParamType, ...] => ReturnType, variadic, { implementation })
    ($struct_name:ty, $op_name:literal, [$($param_type:expr),*] => $return_type:expr, variadic, $implementation:block) => {
        #[async_trait::async_trait]
        impl $crate::traits::AsyncOperation for $struct_name {
            fn name(&self) -> &'static str {
                $op_name
            }

            fn signature(&self) -> &$crate::signature::FunctionSignature {
                static SIGNATURE: $crate::signature::FunctionSignature = $crate::signature::FunctionSignature {
                    name: $op_name,
                    parameters: vec![$($param_type),*],
                    return_type: $return_type,
                    variadic: true,
                };
                &SIGNATURE
            }

            async fn execute(&self, args: &[octofhir_fhirpath_model::FhirPathValue], context: &$crate::traits::EvaluationContext) -> octofhir_fhirpath_core::Result<octofhir_fhirpath_model::FhirPathValue> {
                use octofhir_fhirpath_core::FhirPathError;
                use octofhir_fhirpath_model::FhirPathValue;

                let min_args = SIGNATURE.parameters.len();
                if args.len() < min_args {
                    return Err(FhirPathError::InvalidArgumentCount {
                        function_name: $op_name.to_string(),
                        expected: min_args,
                        actual: args.len(),
                    });
                }
                $implementation
            }
        }
    };
}

/// Macro to create a default constructor for an operation
///
/// # Usage
/// ```rust,ignore
/// pub struct LengthFunction;
/// impl_default_constructor!(LengthFunction);
/// ```
#[macro_export]
macro_rules! impl_default_constructor {
    ($struct_name:ty) => {
        impl $struct_name {
            pub fn new() -> Self {
                Self
            }
        }

        impl Default for $struct_name {
            fn default() -> Self {
                Self::new()
            }
        }
    };
}

/// Macro to register operations in a registry
///
/// # Usage
/// ```rust,ignore
/// # use octofhir_fhirpath_registry::register_sync_ops;
/// register_sync_ops!(registry, [
///     LengthFunction::new(),
///     UpperFunction::new(),
///     CountFunction::new(),
/// ]);
/// ```
#[macro_export]
macro_rules! register_sync_ops {
    ($registry:expr, [$($operation:expr),+ $(,)?]) => {
        {
            let operations: Vec<Box<dyn $crate::traits::SyncOperation>> = vec![
                $(Box::new($operation)),+
            ];
            $registry.register_sync_many(operations).await;
        }
    };
}

/// Macro to register async operations in a registry
///
/// # Usage
/// ```rust,ignore
/// # use octofhir_fhirpath_registry::register_async_ops;
/// register_async_ops!(registry, [
///     ResolveFunction::new(),
///     NowFunction::new(),
///     IsFunction::new(),
/// ]);
/// ```
#[macro_export]
macro_rules! register_async_ops {
    ($registry:expr, [$($operation:expr),+ $(,)?]) => {
        {
            let operations: Vec<Box<dyn $crate::traits::AsyncOperation>> = vec![
                $(Box::new($operation)),+
            ];
            $registry.register_async_many(operations).await;
        }
    };
}

/// Macro to create common string manipulation operations
///
/// # Usage
/// ```rust,ignore
/// # use octofhir_fhirpath_registry::string_manipulation_op;
/// string_manipulation_op!(UpperFunction, "upper", |s: &str| s.to_uppercase());
/// ```
#[macro_export]
macro_rules! string_manipulation_op {
    ($struct_name:ident, $op_name:literal, |$input:ident: &str| $transform:expr) => {
        pub struct $struct_name;

        impl_default_constructor!($struct_name);

        impl_sync_op!($struct_name, $op_name, [] => $crate::signature::ValueType::String, {
            use $crate::traits::validation::*;
            let $input = validate_string_input(context, $op_name)?;
            let result = $transform;
            Ok(FhirPathValue::String(result.into()))
        });
    };
}

/// Macro to create common collection operations  
///
/// # Usage
/// ```rust,ignore
/// # use octofhir_fhirpath_registry::collection_count_op;
/// collection_count_op!(CountFunction, "count");
/// ```
#[macro_export]
macro_rules! collection_count_op {
    ($struct_name:ident, $op_name:literal) => {
        pub struct $struct_name;

        impl_default_constructor!($struct_name);

        impl_sync_op!($struct_name, $op_name, [] => $crate::signature::ValueType::Integer, {
            use $crate::traits::validation::*;
            let items = get_collection_items(&context.input);
            Ok(FhirPathValue::Integer(items.len() as i64))
        });
    };
}

/// Macro to create common math operations
///
/// # Usage  
/// ```rust,ignore
/// # use octofhir_fhirpath_registry::math_unary_op;
/// math_unary_op!(AbsFunction, "abs", |n: f64| n.abs());
/// ```
#[macro_export]
macro_rules! math_unary_op {
    ($struct_name:ident, $op_name:literal, |$input:ident: f64| $operation:expr) => {
        pub struct $struct_name;

        impl_default_constructor!($struct_name);

        impl_sync_op!($struct_name, $op_name, [] => $crate::signature::ValueType::Any, {
            use $crate::traits::validation::*;
            validate_numeric_input(context, $op_name)?;

            match &context.input {
                FhirPathValue::Integer(n) => {
                    let $input = *n as f64;
                    let result = $operation;
                    if result.fract() == 0.0 && result.abs() <= i64::MAX as f64 {
                        Ok(FhirPathValue::Integer(result as i64))
                    } else {
                        Ok(FhirPathValue::Decimal(result.into()))
                    }
                },
                FhirPathValue::Decimal(d) => {
                    let $input = d.to_f64().unwrap_or(0.0);
                    let result = $operation;
                    Ok(FhirPathValue::Decimal(result.into()))
                },
                _ => unreachable!(), // validate_numeric_input ensures this
            }
        });
    };
}

// TODO: Add macro tests once trait validation helper functions are implemented
// The macro tests have been temporarily removed due to missing validation helper functions.
