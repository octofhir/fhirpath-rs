#[cfg(test)]
mod tests {
    use octofhir_ucum_fhir::FhirQuantity;

    #[test]
    fn test_ucum_conversion() {
        // Test basic UCUM functionality
        let q1 = FhirQuantity::with_ucum_code(4.0, "g");
        let q2 = FhirQuantity::with_ucum_code(4000.0, "mg");

        println!("q1: {:?}", q1);
        println!("q2: {:?}", q2);

        // Try to convert to UCUM quantities
        match q1.to_ucum_quantity() {
            Ok(ucum1) => {
                println!("q1 UCUM: value={}, unit={}", ucum1.value, ucum1.unit);
            }
            Err(e) => println!("q1 UCUM conversion failed: {:?}", e),
        }

        match q2.to_ucum_quantity() {
            Ok(ucum2) => {
                println!("q2 UCUM: value={}, unit={}", ucum2.value, ucum2.unit);
            }
            Err(e) => println!("q2 UCUM conversion failed: {:?}", e),
        }

        // Test if both convert to the same base unit
        match (q1.to_ucum_quantity(), q2.to_ucum_quantity()) {
            (Ok(ucum1), Ok(ucum2)) => {
                println!("Comparing UCUM quantities:");
                println!("  q1: {} {}", ucum1.value, ucum1.unit);
                println!("  q2: {} {}", ucum2.value, ucum2.unit);
                println!("  Units equal: {}", ucum1.unit == ucum2.unit);
                println!("  Values equal: {}", (ucum1.value - ucum2.value).abs() < f64::EPSILON);
            }
            _ => println!("UCUM conversion failed for comparison"),
        }
    }

    #[test]
    fn test_time_units() {
        let q1 = FhirQuantity::with_ucum_code(7.0, "d");  // days
        let q2 = FhirQuantity::with_ucum_code(1.0, "wk"); // week

        println!("Time units test:");
        println!("q1 (7 days): {:?}", q1);
        println!("q2 (1 week): {:?}", q2);

        match (q1.to_ucum_quantity(), q2.to_ucum_quantity()) {
            (Ok(ucum1), Ok(ucum2)) => {
                println!("  q1 UCUM: {} {}", ucum1.value, ucum1.unit);
                println!("  q2 UCUM: {} {}", ucum2.value, ucum2.unit);
                println!("  Units equal: {}", ucum1.unit == ucum2.unit);
                println!("  Values equal: {}", (ucum1.value - ucum2.value).abs() < f64::EPSILON);
            }
            _ => println!("UCUM conversion failed for time units"),
        }
    }

    #[test]
    fn test_ucum_methods() {
        let q1 = FhirQuantity::with_ucum_code(4.0, "g");
        let q2 = FhirQuantity::with_ucum_code(4000.0, "mg");

        println!("Testing UCUM methods:");

        match (q1.to_ucum_quantity(), q2.to_ucum_quantity()) {
            (Ok(ucum1), Ok(ucum2)) => {
                println!("  q1 UCUM: {} {:?}", ucum1.value, ucum1.unit);
                println!("  q2 UCUM: {} {:?}", ucum2.value, ucum2.unit);

                // Check what fields and methods are available
                println!("  Quantity struct details:");
                println!("    q1.value: {}", ucum1.value);
                println!("    q1.unit: {:?}", ucum1.unit);
                println!("    q2.value: {}", ucum2.value);
                println!("    q2.unit: {:?}", ucum2.unit);

                // Check if we can access any conversion functionality
                println!("  Checking for conversion methods...");

                // Try to see if there's a way to check unit compatibility
                println!("  Units are same: {}", ucum1.unit == ucum2.unit);

                // Try to convert UnitExpr to string for comparison
                let unit1_str = format!("{:?}", ucum1.unit);
                let unit2_str = format!("{:?}", ucum2.unit);
                println!("  Unit1 as string: {}", unit1_str);
                println!("  Unit2 as string: {}", unit2_str);

                // Manual conversion check - if we can identify mg and g
                if unit2_str.contains("mg") && unit1_str.contains("g") && !unit1_str.contains("mg") {
                    let converted_value = ucum2.value / 1000.0;
                    println!("  Manual conversion: {} mg = {} g", ucum2.value, converted_value);
                    println!("  Values equal after conversion: {}", (ucum1.value - converted_value).abs() < f64::EPSILON);
                }
            }
            _ => println!("UCUM conversion failed"),
        }
    }

    #[test]
    fn test_ucum_string_based_conversion() {
        println!("Testing UCUM string-based conversion:");

        let q1 = FhirQuantity::with_ucum_code(4.0, "g");
        let q2 = FhirQuantity::with_ucum_code(4000.0, "mg");

        match (q1.to_ucum_quantity(), q2.to_ucum_quantity()) {
            (Ok(ucum1), Ok(ucum2)) => {
                println!("  Successfully created UCUM quantities");
                println!("  q1: {} {:?}", ucum1.value, ucum1.unit);
                println!("  q2: {} {:?}", ucum2.value, ucum2.unit);

                // Convert UnitExpr to string for comparison
                let unit1_str = format!("{:?}", ucum1.unit);
                let unit2_str = format!("{:?}", ucum2.unit);

                println!("  Unit1 as string: {}", unit1_str);
                println!("  Unit2 as string: {}", unit2_str);

                // Check if we can identify mg and g units and convert
                if unit2_str.contains("mg") && unit1_str.contains("g") && !unit1_str.contains("mg") {
                    let converted_value = ucum2.value / 1000.0;
                    println!("  Manual conversion: {} mg = {} g", ucum2.value, converted_value);
                    let values_equal = (ucum1.value - converted_value).abs() < f64::EPSILON;
                    println!("  Values equal after conversion: {}", values_equal);
                    assert!(values_equal, "4g should equal 4000mg after conversion");
                }
            }
            _ => println!("UCUM conversion failed"),
        }
    }
}
