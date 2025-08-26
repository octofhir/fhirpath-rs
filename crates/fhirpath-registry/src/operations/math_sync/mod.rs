//! Simplified math operations module

pub mod abs;
pub mod ceiling;
pub mod exp;
pub mod floor;
pub mod ln;
pub mod log;
pub mod power;
pub mod precision;
pub mod round;
pub mod sqrt;
pub mod truncate;

// Arithmetic operations
pub mod add;
pub mod divide;
pub mod modulo;
pub mod multiply;
pub mod subtract;

pub use abs::SimpleAbsFunction;
pub use ceiling::SimpleCeilingFunction;
pub use exp::SimpleExpFunction;
pub use floor::SimpleFloorFunction;
pub use ln::SimpleLnFunction;
pub use log::SimpleLogFunction;
pub use power::SimplePowerFunction;
pub use precision::SimplePrecisionFunction;
pub use round::SimpleRoundFunction;
pub use sqrt::SimpleSqrtFunction;
pub use truncate::SimpleTruncateFunction;

// Arithmetic operations
pub use add::SimpleAddFunction;
pub use divide::SimpleDivideFunction;
pub use modulo::SimpleModuloFunction;
pub use multiply::SimpleMultiplyFunction;
pub use subtract::SimpleSubtractFunction;

#[cfg(not(test))]
mod tests {

    use crate::traits::EvaluationContext;
    use octofhir_fhirpath_model::FhirPathValue;

    fn create_test_context(input: FhirPathValue) -> EvaluationContext {
        let model_provider = std::sync::Arc::new(octofhir_fhirpath_model::MockModelProvider::new());
        EvaluationContext::new(input.clone(), std::sync::Arc::new(input), model_provider)
    }

    #[test]
    fn test_abs_function() {
        let func = SimpleAbsFunction::new();
        assert_eq!(func.name(), "abs");
        assert!(matches!(func.signature().return_type, ValueType::Any));

        // Test with negative integer
        let context = create_test_context(FhirPathValue::Integer(-5));
        let result = func.execute(&[], &context).unwrap();
        assert_eq!(result, FhirPathValue::Integer(5));

        // Test with negative decimal
        let context = create_test_context(FhirPathValue::Decimal(
            rust_decimal::Decimal::from_str("-3.14").unwrap(),
        ));
        let result = func.execute(&[], &context).unwrap();
        assert_eq!(
            result,
            FhirPathValue::Decimal(rust_decimal::Decimal::from_str("3.14").unwrap())
        );
    }

    #[test]
    fn test_sqrt_function() {
        let func = SimpleSqrtFunction::new();
        assert_eq!(func.name(), "sqrt");

        // Test with positive integer
        let context = create_test_context(FhirPathValue::Integer(9));
        let result = func.execute(&[], &context).unwrap();
        assert_eq!(
            result,
            FhirPathValue::Decimal(rust_decimal::Decimal::from_str("3.0").unwrap())
        );

        // Test with negative number should error
        let context = create_test_context(FhirPathValue::Integer(-4));
        let result = func.execute(&[], &context);
        assert!(result.is_err());
    }

    #[test]
    fn test_ceiling_function() {
        let func = SimpleCeilingFunction::new();
        assert_eq!(func.name(), "ceiling");

        // Test with decimal
        let context = create_test_context(FhirPathValue::Decimal(
            rust_decimal::Decimal::from_str("3.2").unwrap(),
        ));
        let result = func.execute(&[], &context).unwrap();
        assert_eq!(result, FhirPathValue::Integer(4));

        // Test with integer (should return same)
        let context = create_test_context(FhirPathValue::Integer(5));
        let result = func.execute(&[], &context).unwrap();
        assert_eq!(result, FhirPathValue::Integer(5));
    }

    #[test]
    fn test_floor_function() {
        let func = SimpleFloorFunction::new();
        assert_eq!(func.name(), "floor");

        // Test with decimal
        let context = create_test_context(FhirPathValue::Decimal(
            rust_decimal::Decimal::from_str("3.8").unwrap(),
        ));
        let result = func.execute(&[], &context).unwrap();
        assert_eq!(result, FhirPathValue::Integer(3));
    }

    #[test]
    fn test_round_function() {
        let func = SimpleRoundFunction::new();
        assert_eq!(func.name(), "round");

        // Test with decimal, no precision
        let context = create_test_context(FhirPathValue::Decimal(
            rust_decimal::Decimal::from_str("3.7").unwrap(),
        ));
        let result = func.execute(&[], &context).unwrap();
        assert_eq!(result, FhirPathValue::Integer(4));

        // Test with precision
        let context = create_test_context(FhirPathValue::Decimal(
            rust_decimal::Decimal::from_str("3.14159").unwrap(),
        ));
        let result = func
            .execute(&[FhirPathValue::Integer(2)], &context)
            .unwrap();
        assert_eq!(
            result,
            FhirPathValue::Decimal(rust_decimal::Decimal::from_str("3.14").unwrap())
        );
    }
}
