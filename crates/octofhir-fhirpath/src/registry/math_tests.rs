//! Tests for math functions

#[cfg(test)]
mod tests {
    use super::super::*;
    use crate::core::{FhirPathValue, ModelProvider};
    use crate::mock_provider::MockModelProvider;
    use std::collections::HashMap;
    use rust_decimal::Decimal;
    use rust_decimal::prelude::ToPrimitive;
    use std::str::FromStr;

    fn create_test_context_with_globals<'a>(
        input: &'a [FhirPathValue],
        arguments: &'a [FhirPathValue],
    ) -> FunctionContext<'a> {
        use std::sync::OnceLock;
        
        static MODEL_PROVIDER: OnceLock<MockModelProvider> = OnceLock::new();
        static VARIABLES: OnceLock<HashMap<String, FhirPathValue>> = OnceLock::new();
        
        let mp = MODEL_PROVIDER.get_or_init(|| MockModelProvider::default());
        let vars = VARIABLES.get_or_init(|| HashMap::new());
        
        FunctionContext {
            input,
            arguments,
            model_provider: mp,
            variables: vars,
            resource_context: None,
            terminology: None,
        }
    }
    
    macro_rules! create_test_context {
        ($input:expr, $args:expr) => {
            create_test_context_with_globals($input, $args)
        };
    }

    #[test]
    fn test_abs_function() {
        let registry = FunctionRegistry::default();
        let dispatcher = dispatcher::FunctionDispatcher::new(registry);

        // Test with positive integer
        let input = vec![FhirPathValue::Integer(5)];
        let context = create_test_context!(&input, &[]);
        let result = dispatcher.dispatch_sync("abs", &context).unwrap();
        assert_eq!(result[0], FhirPathValue::Integer(5));

        // Test with negative integer
        let input = vec![FhirPathValue::Integer(-5)];
        let context = create_test_context!(&input, &[]);
        let result = dispatcher.dispatch_sync("abs", &context).unwrap();
        assert_eq!(result[0], FhirPathValue::Integer(5));

        // Test with negative decimal
        let input = vec![FhirPathValue::Decimal(Decimal::from_str("-3.14").unwrap())];
        let context = create_test_context!(&input, &[]);
        let result = dispatcher.dispatch_sync("abs", &context).unwrap();
        assert_eq!(result[0], FhirPathValue::Decimal(Decimal::from_str("3.14").unwrap()));

        // Test with quantity
        let input = vec![FhirPathValue::Quantity {
            value: Decimal::from_str("-10.5").unwrap(),
            unit: Some("kg".to_string()),
            ucum_unit: None,
            calendar_unit: None,
        }];
        let context = create_test_context!(&input, &[]);
        let result = dispatcher.dispatch_sync("abs", &context).unwrap();
        if let FhirPathValue::Quantity { value, unit, .. } = &result[0] {
            assert_eq!(*value, Decimal::from_str("10.5").unwrap());
            assert_eq!(unit.as_ref().unwrap(), "kg");
        } else {
            panic!("Expected quantity result");
        }

        // Test with non-numeric value (should error)
        let input = vec![FhirPathValue::String("test".to_string())];
        let context = create_test_context!(&input, &[]);
        let result = dispatcher.dispatch_sync("abs", &context);
        assert!(result.is_err());
    }

    #[test]
    fn test_ceiling_function() {
        let registry = FunctionRegistry::default();
        let dispatcher = dispatcher::FunctionDispatcher::new(registry);

        // Test with positive decimal
        let input = vec![FhirPathValue::Decimal(Decimal::from_str("3.14").unwrap())];
        let context = create_test_context!(&input, &[]);
        let result = dispatcher.dispatch_sync("ceiling", &context).unwrap();
        assert_eq!(result[0], FhirPathValue::Integer(4));

        // Test with negative decimal
        let input = vec![FhirPathValue::Decimal(Decimal::from_str("-2.5").unwrap())];
        let context = create_test_context!(&input, &[]);
        let result = dispatcher.dispatch_sync("ceiling", &context).unwrap();
        assert_eq!(result[0], FhirPathValue::Integer(-2));

        // Test with integer (should return same integer)
        let input = vec![FhirPathValue::Integer(5)];
        let context = create_test_context!(&input, &[]);
        let result = dispatcher.dispatch_sync("ceiling", &context).unwrap();
        assert_eq!(result[0], FhirPathValue::Integer(5));
    }

    #[test]
    fn test_floor_function() {
        let registry = FunctionRegistry::default();
        let dispatcher = dispatcher::FunctionDispatcher::new(registry);

        // Test with positive decimal
        let input = vec![FhirPathValue::Decimal(Decimal::from_str("3.14").unwrap())];
        let context = create_test_context!(&input, &[]);
        let result = dispatcher.dispatch_sync("floor", &context).unwrap();
        assert_eq!(result[0], FhirPathValue::Integer(3));

        // Test with negative decimal
        let input = vec![FhirPathValue::Decimal(Decimal::from_str("-2.5").unwrap())];
        let context = create_test_context!(&input, &[]);
        let result = dispatcher.dispatch_sync("floor", &context).unwrap();
        assert_eq!(result[0], FhirPathValue::Integer(-3));

        // Test with integer
        let input = vec![FhirPathValue::Integer(-5)];
        let context = create_test_context!(&input, &[]);
        let result = dispatcher.dispatch_sync("floor", &context).unwrap();
        assert_eq!(result[0], FhirPathValue::Integer(-5));
    }

    #[test]
    fn test_round_function() {
        let registry = FunctionRegistry::default();
        let dispatcher = dispatcher::FunctionDispatcher::new(registry);

        // Test rounding with precision
        let input = vec![FhirPathValue::Decimal(Decimal::from_str("3.14159").unwrap())];
        let arguments = vec![FhirPathValue::Integer(2)];
        let context = create_test_context!(&input, &arguments);
        let result = dispatcher.dispatch_sync("round", &context).unwrap();
        assert_eq!(result[0], FhirPathValue::Decimal(Decimal::from_str("3.14").unwrap()));

        // Test rounding without precision (should round to integer)
        let input = vec![FhirPathValue::Decimal(Decimal::from_str("3.6").unwrap())];
        let context = create_test_context!(&input, &[]);
        let result = dispatcher.dispatch_sync("round", &context).unwrap();
        assert_eq!(result[0], FhirPathValue::Integer(4));

        // Test rounding with negative value
        let input = vec![FhirPathValue::Decimal(Decimal::from_str("-2.5").unwrap())];
        let context = create_test_context!(&input, &[]);
        let result = dispatcher.dispatch_sync("round", &context).unwrap();
        assert_eq!(result[0], FhirPathValue::Integer(-3)); // Banker's rounding

        // Test with invalid precision (should error)
        let input = vec![FhirPathValue::Decimal(Decimal::from_str("3.14").unwrap())];
        let arguments = vec![FhirPathValue::Integer(-1)];
        let context = create_test_context!(&input, &arguments);
        let result = dispatcher.dispatch_sync("round", &context);
        assert!(result.is_err());

        // Test with quantity
        let input = vec![FhirPathValue::Quantity {
            value: Decimal::from_str("3.14159").unwrap(),
            unit: Some("m".to_string()),
            ucum_unit: None,
            calendar_unit: None,
        }];
        let arguments = vec![FhirPathValue::Integer(2)];
        let context = create_test_context!(&input, &arguments);
        let result = dispatcher.dispatch_sync("round", &context).unwrap();
        if let FhirPathValue::Quantity { value, unit, .. } = &result[0] {
            assert_eq!(*value, Decimal::from_str("3.14").unwrap());
            assert_eq!(unit.as_ref().unwrap(), "m");
        } else {
            panic!("Expected quantity result");
        }
    }

    #[test]
    fn test_truncate_function() {
        let registry = FunctionRegistry::default();
        let dispatcher = dispatcher::FunctionDispatcher::new(registry);

        // Test with positive decimal
        let input = vec![FhirPathValue::Decimal(Decimal::from_str("3.14").unwrap())];
        let context = create_test_context!(&input, &[]);
        let result = dispatcher.dispatch_sync("truncate", &context).unwrap();
        assert_eq!(result[0], FhirPathValue::Integer(3));

        // Test with negative decimal (truncate towards zero)
        let input = vec![FhirPathValue::Decimal(Decimal::from_str("-2.9").unwrap())];
        let context = create_test_context!(&input, &[]);
        let result = dispatcher.dispatch_sync("truncate", &context).unwrap();
        assert_eq!(result[0], FhirPathValue::Integer(-2));

        // Test with integer
        let input = vec![FhirPathValue::Integer(5)];
        let context = create_test_context!(&input, &[]);
        let result = dispatcher.dispatch_sync("truncate", &context).unwrap();
        assert_eq!(result[0], FhirPathValue::Integer(5));
    }

    #[test]
    fn test_sqrt_function() {
        let registry = FunctionRegistry::default();
        let dispatcher = dispatcher::FunctionDispatcher::new(registry);

        // Test perfect square
        let input = vec![FhirPathValue::Integer(16)];
        let context = create_test_context!(&input, &[]);
        let result = dispatcher.dispatch_sync("sqrt", &context).unwrap();
        if let FhirPathValue::Decimal(d) = &result[0] {
            assert!((d.to_f64().unwrap() - 4.0).abs() < 0.0001);
        } else {
            panic!("Expected decimal result");
        }

        // Test decimal value
        let input = vec![FhirPathValue::Decimal(Decimal::from_str("2.25").unwrap())];
        let context = create_test_context!(&input, &[]);
        let result = dispatcher.dispatch_sync("sqrt", &context).unwrap();
        if let FhirPathValue::Decimal(d) = &result[0] {
            assert!((d.to_f64().unwrap() - 1.5).abs() < 0.0001);
        } else {
            panic!("Expected decimal result");
        }

        // Test zero
        let input = vec![FhirPathValue::Integer(0)];
        let context = create_test_context!(&input, &[]);
        let result = dispatcher.dispatch_sync("sqrt", &context).unwrap();
        if let FhirPathValue::Decimal(d) = &result[0] {
            assert_eq!(d.to_f64().unwrap(), 0.0);
        } else {
            panic!("Expected decimal result");
        }

        // Test negative value (should error)
        let input = vec![FhirPathValue::Integer(-4)];
        let context = create_test_context!(&input, &[]);
        let result = dispatcher.dispatch_sync("sqrt", &context);
        assert!(result.is_err());
    }

    #[test]
    fn test_ln_function() {
        let registry = FunctionRegistry::default();
        let dispatcher = dispatcher::FunctionDispatcher::new(registry);

        // Test ln(1) = 0
        let input = vec![FhirPathValue::Integer(1)];
        let context = create_test_context!(&input, &[]);
        let result = dispatcher.dispatch_sync("ln", &context).unwrap();
        if let FhirPathValue::Decimal(d) = &result[0] {
            assert!(d.to_f64().unwrap().abs() < 0.0001);
        } else {
            panic!("Expected decimal result");
        }

        // Test ln(e) ≈ 1
        let e_value = Decimal::try_from(std::f64::consts::E).unwrap();
        let input = vec![FhirPathValue::Decimal(e_value)];
        let context = create_test_context!(&input, &[]);
        let result = dispatcher.dispatch_sync("ln", &context).unwrap();
        if let FhirPathValue::Decimal(d) = &result[0] {
            assert!((d.to_f64().unwrap() - 1.0).abs() < 0.0001);
        } else {
            panic!("Expected decimal result");
        }

        // Test negative value (should error)
        let input = vec![FhirPathValue::Integer(-1)];
        let context = create_test_context!(&input, &[]);
        let result = dispatcher.dispatch_sync("ln", &context);
        assert!(result.is_err());

        // Test zero (should error)
        let input = vec![FhirPathValue::Integer(0)];
        let context = create_test_context!(&input, &[]);
        let result = dispatcher.dispatch_sync("ln", &context);
        assert!(result.is_err());
    }

    #[test]
    fn test_log_function() {
        let registry = FunctionRegistry::default();
        let dispatcher = dispatcher::FunctionDispatcher::new(registry);

        // Test log(100, 10) = 2
        let input = vec![FhirPathValue::Integer(100)];
        let arguments = vec![FhirPathValue::Integer(10)];
        let context = create_test_context!(&input, &arguments);
        let result = dispatcher.dispatch_sync("log", &context).unwrap();
        if let FhirPathValue::Decimal(d) = &result[0] {
            assert!((d.to_f64().unwrap() - 2.0).abs() < 0.0001);
        } else {
            panic!("Expected decimal result");
        }

        // Test log(8, 2) = 3
        let input = vec![FhirPathValue::Integer(8)];
        let arguments = vec![FhirPathValue::Integer(2)];
        let context = create_test_context!(&input, &arguments);
        let result = dispatcher.dispatch_sync("log", &context).unwrap();
        if let FhirPathValue::Decimal(d) = &result[0] {
            assert!((d.to_f64().unwrap() - 3.0).abs() < 0.0001);
        } else {
            panic!("Expected decimal result");
        }

        // Test with base 1 (should error)
        let input = vec![FhirPathValue::Integer(10)];
        let arguments = vec![FhirPathValue::Integer(1)];
        let context = create_test_context!(&input, &arguments);
        let result = dispatcher.dispatch_sync("log", &context);
        assert!(result.is_err());

        // Test with negative base (should error)
        let input = vec![FhirPathValue::Integer(10)];
        let arguments = vec![FhirPathValue::Integer(-2)];
        let context = create_test_context!(&input, &arguments);
        let result = dispatcher.dispatch_sync("log", &context);
        assert!(result.is_err());

        // Test with negative value (should error)
        let input = vec![FhirPathValue::Integer(-10)];
        let arguments = vec![FhirPathValue::Integer(2)];
        let context = create_test_context!(&input, &arguments);
        let result = dispatcher.dispatch_sync("log", &context);
        assert!(result.is_err());
    }

    #[test]
    fn test_power_function() {
        let registry = FunctionRegistry::default();
        let dispatcher = dispatcher::FunctionDispatcher::new(registry);

        // Test 2^3 = 8
        let input = vec![FhirPathValue::Integer(2)];
        let arguments = vec![FhirPathValue::Integer(3)];
        let context = create_test_context!(&input, &arguments);
        let result = dispatcher.dispatch_sync("power", &context).unwrap();
        if let FhirPathValue::Decimal(d) = &result[0] {
            assert!((d.to_f64().unwrap() - 8.0).abs() < 0.0001);
        } else {
            panic!("Expected decimal result");
        }

        // Test 16^0.5 = 4 (square root)
        let input = vec![FhirPathValue::Integer(16)];
        let arguments = vec![FhirPathValue::Decimal(Decimal::from_str("0.5").unwrap())];
        let context = create_test_context!(&input, &arguments);
        let result = dispatcher.dispatch_sync("power", &context).unwrap();
        if let FhirPathValue::Decimal(d) = &result[0] {
            assert!((d.to_f64().unwrap() - 4.0).abs() < 0.0001);
        } else {
            panic!("Expected decimal result");
        }

        // Test 5^0 = 1
        let input = vec![FhirPathValue::Integer(5)];
        let arguments = vec![FhirPathValue::Integer(0)];
        let context = create_test_context!(&input, &arguments);
        let result = dispatcher.dispatch_sync("power", &context).unwrap();
        if let FhirPathValue::Decimal(d) = &result[0] {
            assert!((d.to_f64().unwrap() - 1.0).abs() < 0.0001);
        } else {
            panic!("Expected decimal result");
        }

        // Test negative base with fractional exponent (can result in NaN)
        let input = vec![FhirPathValue::Integer(-4)];
        let arguments = vec![FhirPathValue::Decimal(Decimal::from_str("0.5").unwrap())];
        let context = create_test_context!(&input, &arguments);
        let result = dispatcher.dispatch_sync("power", &context);
        assert!(result.is_err()); // Should error because result is NaN
    }

    #[test]
    fn test_exp_function() {
        let registry = FunctionRegistry::default();
        let dispatcher = dispatcher::FunctionDispatcher::new(registry);

        // Test exp(0) = 1
        let input = vec![FhirPathValue::Integer(0)];
        let context = create_test_context!(&input, &[]);
        let result = dispatcher.dispatch_sync("exp", &context).unwrap();
        if let FhirPathValue::Decimal(d) = &result[0] {
            assert!((d.to_f64().unwrap() - 1.0).abs() < 0.0001);
        } else {
            panic!("Expected decimal result");
        }

        // Test exp(1) ≈ e
        let input = vec![FhirPathValue::Integer(1)];
        let context = create_test_context!(&input, &[]);
        let result = dispatcher.dispatch_sync("exp", &context).unwrap();
        if let FhirPathValue::Decimal(d) = &result[0] {
            assert!((d.to_f64().unwrap() - std::f64::consts::E).abs() < 0.0001);
        } else {
            panic!("Expected decimal result");
        }

        // Test exp(2) ≈ e^2
        let input = vec![FhirPathValue::Integer(2)];
        let context = create_test_context!(&input, &[]);
        let result = dispatcher.dispatch_sync("exp", &context).unwrap();
        if let FhirPathValue::Decimal(d) = &result[0] {
            assert!((d.to_f64().unwrap() - std::f64::consts::E.powi(2)).abs() < 0.0001);
        } else {
            panic!("Expected decimal result");
        }
    }

    #[test]
    fn test_arithmetic_operations() {
        use super::super::math::ArithmeticOperations;

        // Test integer addition
        let result = ArithmeticOperations::add(
            &FhirPathValue::Integer(5),
            &FhirPathValue::Integer(3)
        ).unwrap();
        assert_eq!(result, FhirPathValue::Integer(8));

        // Test mixed integer/decimal addition
        let result = ArithmeticOperations::add(
            &FhirPathValue::Integer(5),
            &FhirPathValue::Decimal(Decimal::from_str("3.5").unwrap())
        ).unwrap();
        assert_eq!(result, FhirPathValue::Decimal(Decimal::from_str("8.5").unwrap()));

        // Test string concatenation
        let result = ArithmeticOperations::add(
            &FhirPathValue::String("Hello".to_string()),
            &FhirPathValue::String(" World".to_string())
        ).unwrap();
        assert_eq!(result, FhirPathValue::String("Hello World".to_string()));

        // Test quantity addition with same units
        let result = ArithmeticOperations::add(
            &FhirPathValue::Quantity {
                value: Decimal::from_str("5.0").unwrap(),
                unit: Some("kg".to_string()),
                ucum_unit: None,
                calendar_unit: None,
            },
            &FhirPathValue::Quantity {
                value: Decimal::from_str("3.0").unwrap(),
                unit: Some("kg".to_string()),
                ucum_unit: None,
                calendar_unit: None,
            }
        ).unwrap();
        if let FhirPathValue::Quantity { value, unit, .. } = result {
            assert_eq!(value, Decimal::from_str("8.0").unwrap());
            assert_eq!(unit.as_ref().unwrap(), "kg");
        } else {
            panic!("Expected quantity result");
        }

        // Test quantity addition with different units (should error)
        let result = ArithmeticOperations::add(
            &FhirPathValue::Quantity {
                value: Decimal::from_str("5.0").unwrap(),
                unit: Some("kg".to_string()),
                ucum_unit: None,
                calendar_unit: None,
            },
            &FhirPathValue::Quantity {
                value: Decimal::from_str("3.0").unwrap(),
                unit: Some("g".to_string()),
                ucum_unit: None,
                calendar_unit: None,
            }
        );
        assert!(result.is_err());

        // Test subtraction
        let result = ArithmeticOperations::subtract(
            &FhirPathValue::Integer(10),
            &FhirPathValue::Integer(3)
        ).unwrap();
        assert_eq!(result, FhirPathValue::Integer(7));

        // Test multiplication
        let result = ArithmeticOperations::multiply(
            &FhirPathValue::Integer(4),
            &FhirPathValue::Integer(3)
        ).unwrap();
        assert_eq!(result, FhirPathValue::Integer(12));

        // Test quantity multiplication by scalar
        let result = ArithmeticOperations::multiply(
            &FhirPathValue::Quantity {
                value: Decimal::from_str("5.0").unwrap(),
                unit: Some("m".to_string()),
                ucum_unit: None,
                calendar_unit: None,
            },
            &FhirPathValue::Integer(3)
        ).unwrap();
        if let FhirPathValue::Quantity { value, unit, .. } = result {
            assert_eq!(value, Decimal::from_str("15.0").unwrap());
            assert_eq!(unit.as_ref().unwrap(), "m");
        } else {
            panic!("Expected quantity result");
        }

        // Test division (integer division returns decimal in FHIRPath)
        let result = ArithmeticOperations::divide(
            &FhirPathValue::Integer(10),
            &FhirPathValue::Integer(4)
        ).unwrap();
        assert_eq!(result, FhirPathValue::Decimal(Decimal::from_str("2.5").unwrap()));

        // Test division by zero
        let result = ArithmeticOperations::divide(
            &FhirPathValue::Integer(5),
            &FhirPathValue::Integer(0)
        );
        assert!(result.is_err());

        // Test modulo
        let result = ArithmeticOperations::modulo(
            &FhirPathValue::Integer(10),
            &FhirPathValue::Integer(3)
        ).unwrap();
        assert_eq!(result, FhirPathValue::Integer(1));

        // Test modulo by zero
        let result = ArithmeticOperations::modulo(
            &FhirPathValue::Integer(5),
            &FhirPathValue::Integer(0)
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_multiple_input_values_error() {
        let registry = FunctionRegistry::default();
        let dispatcher = dispatcher::FunctionDispatcher::new(registry);

        // Test functions that should only work on single values
        let input = vec![
            FhirPathValue::Integer(5),
            FhirPathValue::Integer(3)
        ];

        for function_name in &["abs", "ceiling", "floor", "truncate", "sqrt", "ln", "exp"] {
            let context = create_test_context!(&input, &[]);
            let result = dispatcher.dispatch_sync(function_name, &context);
            assert!(result.is_err(), "Function {} should error with multiple inputs", function_name);
        }
    }

    #[test]
    fn test_missing_arguments_error() {
        let registry = FunctionRegistry::default();
        let dispatcher = dispatcher::FunctionDispatcher::new(registry);

        let input = vec![FhirPathValue::Integer(10)];

        // Test functions that require arguments
        for function_name in &["log", "power"] {
            let context = create_test_context!(&input, &[]);
            let result = dispatcher.dispatch_sync(function_name, &context);
            assert!(result.is_err(), "Function {} should error without required arguments", function_name);
        }
    }
}
