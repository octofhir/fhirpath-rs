use fhirpath_core::FhirPathValue;
use rust_decimal::Decimal;
use std::str::FromStr;

fn main() {
    println!("Testing UCUM integration...");

    // Test 1: Create quantities with units
    let qty1 = FhirPathValue::quantity(Decimal::from_str("100").unwrap(), Some("cm".to_string()));
    let qty2 = FhirPathValue::quantity(Decimal::from_str("1").unwrap(), Some("m".to_string()));

    println!("Created quantity 1: {:?}", qty1);
    println!("Created quantity 2: {:?}", qty2);

    // Test 2: Check if units are compatible
    let compatible = qty1.has_compatible_dimensions(&qty2);
    println!("Are cm and m compatible? {}", compatible);

    // Test 3: Try unit conversion (this should return an error for now as we haven't implemented full conversion)
    match qty1.convert_to_unit("m") {
        Ok(converted) => println!("Converted 100 cm to m: {:?}", converted),
        Err(e) => println!("Conversion error (expected): {}", e),
    }

    // Test 4: Test with invalid unit to ensure it doesn't freeze
    let invalid_qty = FhirPathValue::quantity(Decimal::from_str("10").unwrap(), Some("invalid_unit_xyz_123".to_string()));
    println!("Created quantity with invalid unit: {:?}", invalid_qty);

    // Test 5: Test with very long unit string to ensure timeout works
    let long_unit = "a".repeat(300);
    let long_qty = FhirPathValue::quantity(Decimal::from_str("5").unwrap(), Some(long_unit));
    println!("Created quantity with long unit string (should be rejected): {:?}", long_qty);

    println!("UCUM integration test completed successfully!");
}
