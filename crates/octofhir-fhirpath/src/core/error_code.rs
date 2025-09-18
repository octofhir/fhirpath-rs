//! Simplified error code system similar to Rust compiler (E0001, E0002, etc.)
//!
//! This module provides a centralized error code system using the pattern:
//! FP0001, FP0002, etc. similar to Rust's E0001, E0002 error codes.

use std::fmt;

/// Error categories for organizing error codes
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ErrorCategory {
    /// Parser errors (FP0001-FP0050)
    Parser,
    /// Evaluation errors (FP0051-FP0100)
    Evaluation,
    /// Model provider errors (FP0101-FP0150)
    ModelProvider,
    /// Analysis errors (FP0151-FP0200)
    Analysis,
}

/// Error code following Rust compiler pattern (FP0001, FP0002, etc.)
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ErrorCode {
    pub code: u16,
}

impl ErrorCode {
    /// Create a new error code
    pub const fn new(code: u16) -> Self {
        Self { code }
    }

    /// Get the full error code string (e.g., "FP0001")
    pub fn code_str(&self) -> String {
        format!("FP{:04}", self.code)
    }

    /// Get error information from the registry
    pub fn info(&self) -> &'static ErrorInfo {
        ERROR_REGISTRY.get_error_info(self)
    }

    /// Get documentation URL for this error code
    pub fn docs_url(&self) -> String {
        format!(
            "https://octofhir.github.io/fhirpath-rs/errors/FP{:04}",
            self.code
        )
    }

    /// Get error category for this error code
    pub fn category(&self) -> ErrorCategory {
        match self.code {
            1..=50 => ErrorCategory::Parser,
            51..=100 => ErrorCategory::Evaluation,
            101..=150 => ErrorCategory::ModelProvider,
            151..=200 => ErrorCategory::Analysis,
            _ => ErrorCategory::Parser, // Default fallback
        }
    }

    /// Get human-readable description for this error code
    pub fn description(&self) -> &'static str {
        self.info().title
    }
}

impl fmt::Display for ErrorCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "FP{:04}", self.code)
    }
}

/// Rich error information with documentation links (simplified)
#[derive(Debug, Clone)]
pub struct ErrorInfo {
    /// Error code number
    pub code: u16,
    /// Human-readable error title
    pub title: &'static str,
    /// Detailed description of the error
    pub description: &'static str,
    /// Help information and suggested solutions
    pub help: &'static str,
}

impl ErrorInfo {
    pub const fn new(
        code: u16,
        title: &'static str,
        description: &'static str,
        help: &'static str,
    ) -> Self {
        Self {
            code,
            title,
            description,
            help,
        }
    }

    /// Get documentation URL for this error
    pub fn docs_url(&self) -> String {
        format!(
            "https://octofhir.github.io/fhirpath-rs/errors/FP{:04}",
            self.code
        )
    }
}

/// Central error registry containing all error definitions
pub struct ErrorRegistry;

impl ErrorRegistry {
    /// Get error information for a given error code
    pub fn get_error_info(&self, error_code: &ErrorCode) -> &'static ErrorInfo {
        match error_code.code {
            // Parser Errors (FP0001-FP0050)
            1 => &FP0001_INFO,
            2 => &FP0002_INFO,
            3 => &FP0003_INFO,
            4 => &FP0004_INFO,
            5 => &FP0005_INFO,
            6 => &FP0006_INFO,
            7 => &FP0007_INFO,
            8 => &FP0008_INFO,
            9 => &FP0009_INFO,
            10 => &FP0010_INFO,

            // Evaluation Errors (FP0051-FP0100)
            51 => &FP0051_INFO,
            52 => &FP0052_INFO,
            53 => &FP0053_INFO,
            54 => &FP0054_INFO,
            55 => &FP0055_INFO,
            56 => &FP0056_INFO,
            57 => &FP0057_INFO,
            58 => &FP0058_INFO,
            59 => &FP0059_INFO,
            60 => &FP0060_INFO,
            61 => &FP0061_INFO,

            // Temporal/Date validation errors (FP0070-FP0080)
            70 => &FP0070_INFO,
            71 => &FP0071_INFO,
            72 => &FP0072_INFO,
            73 => &FP0073_INFO,
            74 => &FP0074_INFO,
            75 => &FP0075_INFO,
            76 => &FP0076_INFO,
            77 => &FP0077_INFO,
            78 => &FP0078_INFO,
            79 => &FP0079_INFO,
            80 => &FP0080_INFO,

            // Model Provider Errors (FP0101-FP0150)
            101 => &FP0101_INFO,
            102 => &FP0102_INFO,
            103 => &FP0103_INFO,
            104 => &FP0104_INFO,
            105 => &FP0105_INFO,
            106 => &FP0106_INFO,
            107 => &FP0107_INFO,
            108 => &FP0108_INFO,
            109 => &FP0109_INFO,
            110 => &FP0110_INFO,
            121 => &FP0121_INFO,
            122 => &FP0122_INFO,
            123 => &FP0123_INFO,
            124 => &FP0124_INFO,
            125 => &FP0125_INFO,

            // Analysis Errors (FP0151-FP0200)
            151 => &FP0151_INFO,
            152 => &FP0152_INFO,
            153 => &FP0153_INFO,
            154 => &FP0154_INFO,
            155 => &FP0155_INFO,
            156 => &FP0156_INFO,
            157 => &FP0157_INFO,
            158 => &FP0158_INFO,
            159 => &FP0159_INFO,
            160 => &FP0160_INFO,
            200 => &FP0200_INFO,

            // Default fallback for unknown error codes
            _ => &UNKNOWN_ERROR_INFO,
        }
    }
}

/// Global error registry instance
pub static ERROR_REGISTRY: ErrorRegistry = ErrorRegistry;

// ========== Error Code Definitions (Rust Compiler Style) ==========

// Parser Error Codes (FP0001-FP0050)
pub const FP0001: ErrorCode = ErrorCode::new(1); // Invalid FHIRPath syntax
pub const FP0002: ErrorCode = ErrorCode::new(2); // Unexpected token in expression
pub const FP0003: ErrorCode = ErrorCode::new(3); // Missing closing parenthesis
pub const FP0004: ErrorCode = ErrorCode::new(4); // Missing closing bracket
pub const FP0005: ErrorCode = ErrorCode::new(5); // Invalid string literal
pub const FP0006: ErrorCode = ErrorCode::new(6); // Invalid number literal
pub const FP0007: ErrorCode = ErrorCode::new(7); // Invalid identifier
pub const FP0008: ErrorCode = ErrorCode::new(8); // Unexpected end of input
pub const FP0009: ErrorCode = ErrorCode::new(9); // Invalid operator
pub const FP0010: ErrorCode = ErrorCode::new(10); // Invalid function call syntax

// Evaluation Error Codes (FP0051-FP0100)
pub const FP0051: ErrorCode = ErrorCode::new(51); // Type mismatch in operation
pub const FP0052: ErrorCode = ErrorCode::new(52); // Division by zero
pub const FP0053: ErrorCode = ErrorCode::new(53); // Invalid function arguments
pub const FP0054: ErrorCode = ErrorCode::new(54); // Unknown function
pub const FP0055: ErrorCode = ErrorCode::new(55); // Property not found
pub const FP0056: ErrorCode = ErrorCode::new(56); // Invalid property access
pub const FP0057: ErrorCode = ErrorCode::new(57); // Reference resolution failed
pub const FP0058: ErrorCode = ErrorCode::new(58); // Invalid type conversion
pub const FP0059: ErrorCode = ErrorCode::new(59); // Collection index out of bounds
pub const FP0060: ErrorCode = ErrorCode::new(60); // Variable not defined
pub const FP0061: ErrorCode = ErrorCode::new(61); // Resource type mismatch
pub const FP0062: ErrorCode = ErrorCode::new(62); // Invalid type identifier for type operator
pub const FP0063: ErrorCode = ErrorCode::new(63); // Type operator requires single item collection

// Temporal/Date validation errors (FP0070-FP0080)
pub const FP0070: ErrorCode = ErrorCode::new(70); // Invalid date format
pub const FP0071: ErrorCode = ErrorCode::new(71); // Invalid date value (day > 31)
pub const FP0072: ErrorCode = ErrorCode::new(72); // Invalid month value (month > 12)
pub const FP0073: ErrorCode = ErrorCode::new(73); // Invalid year value
pub const FP0074: ErrorCode = ErrorCode::new(74); // Invalid time format
pub const FP0075: ErrorCode = ErrorCode::new(75); // Invalid datetime format
pub const FP0076: ErrorCode = ErrorCode::new(76); // Date out of valid range
pub const FP0077: ErrorCode = ErrorCode::new(77); // Invalid timezone format
pub const FP0078: ErrorCode = ErrorCode::new(78); // Temporal precision mismatch
pub const FP0079: ErrorCode = ErrorCode::new(79); // Invalid leap year date
pub const FP0080: ErrorCode = ErrorCode::new(80); // Temporal parsing error
pub const FP0081: ErrorCode = ErrorCode::new(81); // Invalid UCUM unit in temporal arithmetic
pub const FP0082: ErrorCode = ErrorCode::new(82); // Invalid temporal arithmetic (date + plain number)

// Model Provider Error Codes (FP0101-FP0150)
pub const FP0101: ErrorCode = ErrorCode::new(101); // Resource not found
pub const FP0102: ErrorCode = ErrorCode::new(102); // Invalid resource type
pub const FP0103: ErrorCode = ErrorCode::new(103); // Property validation failed
pub const FP0104: ErrorCode = ErrorCode::new(104); // Schema validation error
pub const FP0105: ErrorCode = ErrorCode::new(105); // ModelProvider connection failed
pub const FP0106: ErrorCode = ErrorCode::new(106); // Timeout in ModelProvider operation
pub const FP0107: ErrorCode = ErrorCode::new(107); // Invalid FHIR resource
pub const FP0108: ErrorCode = ErrorCode::new(108); // Unsupported FHIR version
pub const FP0109: ErrorCode = ErrorCode::new(109); // Missing required property
pub const FP0110: ErrorCode = ErrorCode::new(110); // Invalid reference format
pub const FP0121: ErrorCode = ErrorCode::new(121); // Unknown resource type
pub const FP0122: ErrorCode = ErrorCode::new(122); // Resource type suggestion available
pub const FP0123: ErrorCode = ErrorCode::new(123); // Invalid resource type format
pub const FP0124: ErrorCode = ErrorCode::new(124); // Unknown function name
pub const FP0125: ErrorCode = ErrorCode::new(125); // Enhanced property validation

// Analysis Error Codes (FP0151-FP0200)
pub const FP0151: ErrorCode = ErrorCode::new(151); // Type inference failed
pub const FP0152: ErrorCode = ErrorCode::new(152); // Unreachable code detected
pub const FP0153: ErrorCode = ErrorCode::new(153); // Performance warning
pub const FP0154: ErrorCode = ErrorCode::new(154); // Optimization suggestion
pub const FP0155: ErrorCode = ErrorCode::new(155); // Static analysis error
pub const FP0156: ErrorCode = ErrorCode::new(156); // Property type mismatch
pub const FP0157: ErrorCode = ErrorCode::new(157); // Function signature mismatch
pub const FP0158: ErrorCode = ErrorCode::new(158); // Dead code detected
pub const FP0159: ErrorCode = ErrorCode::new(159); // Inefficient expression pattern
pub const FP0160: ErrorCode = ErrorCode::new(160); // Missing type annotation

// Additional error codes for system errors
pub const FP0200: ErrorCode = ErrorCode::new(200); // System external error

// ========== Error Information Definitions ==========

// Parser Error Information (FP0001-FP0050)
static FP0001_INFO: ErrorInfo = ErrorInfo::new(
    1,
    "Invalid FHIRPath syntax",
    "The FHIRPath expression contains invalid syntax that cannot be parsed.",
    "Check the expression syntax against the FHIRPath specification. Common issues include mismatched parentheses, invalid operators, or incorrect identifier names.",
);

static FP0002_INFO: ErrorInfo = ErrorInfo::new(
    2,
    "Unexpected token in expression",
    "The parser encountered a token that was not expected in the current context.",
    "Review the expression around the reported position. This often indicates missing operators, incorrect punctuation, or misplaced keywords.",
);

static FP0003_INFO: ErrorInfo = ErrorInfo::new(
    3,
    "Missing closing parenthesis",
    "A parenthesis was opened but never closed, resulting in unbalanced parentheses.",
    "Check that all opening parentheses '(' have matching closing parentheses ')'. Consider using an editor with bracket matching.",
);

static FP0004_INFO: ErrorInfo = ErrorInfo::new(
    4,
    "Missing closing bracket",
    "A bracket was opened but never closed, resulting in unbalanced brackets.",
    "Check that all opening brackets '[' have matching closing brackets ']'. This commonly occurs in index operations.",
);

static FP0005_INFO: ErrorInfo = ErrorInfo::new(
    5,
    "Invalid string literal",
    "A string literal has invalid syntax or contains unsupported escape sequences.",
    "Ensure string literals are properly quoted with single or double quotes. Check for unescaped quotes within the string.",
);

static FP0006_INFO: ErrorInfo = ErrorInfo::new(
    6,
    "Invalid number literal",
    "A numeric literal has an invalid format and cannot be parsed.",
    "Check the number format. FHIRPath supports integers and decimals. Ensure proper decimal notation.",
);

static FP0007_INFO: ErrorInfo = ErrorInfo::new(
    7,
    "Invalid identifier",
    "An identifier contains invalid characters or follows invalid naming rules.",
    "Identifiers must start with a letter or underscore and contain only letters, digits, and underscores.",
);

static FP0008_INFO: ErrorInfo = ErrorInfo::new(
    8,
    "Unexpected end of input",
    "The expression ended unexpectedly while the parser was expecting more tokens.",
    "The expression appears to be incomplete. Check if you're missing function arguments, operators, or closing delimiters.",
);

static FP0009_INFO: ErrorInfo = ErrorInfo::new(
    9,
    "Invalid operator",
    "An invalid or unsupported operator was encountered in the expression.",
    "Check that you're using valid FHIRPath operators. Common operators include =, !=, and, or, +, -, *, /.",
);

static FP0010_INFO: ErrorInfo = ErrorInfo::new(
    10,
    "Invalid function call syntax",
    "The syntax for a function call is incorrect or malformed.",
    "Function calls must follow the pattern 'functionName(arg1, arg2, ...)'. Check parentheses and argument separation.",
);

// Evaluation Error Information (FP0051-FP0100)
static FP0051_INFO: ErrorInfo = ErrorInfo::new(
    51,
    "Type mismatch in operation",
    "An operation was attempted between incompatible types.",
    "Check that operations are performed between compatible types. For example, you cannot add a string to a number directly.",
);

static FP0052_INFO: ErrorInfo = ErrorInfo::new(
    52,
    "Division by zero",
    "An attempt was made to divide by zero.",
    "Check divisor values before performing division operations. Use conditional logic to handle zero divisors appropriately.",
);

static FP0053_INFO: ErrorInfo = ErrorInfo::new(
    53,
    "Invalid function arguments",
    "A function was called with invalid or incompatible argument types.",
    "Check the function signature and ensure arguments match the expected types and constraints.",
);

static FP0054_INFO: ErrorInfo = ErrorInfo::new(
    54,
    "Unknown function",
    "A function was called that is not registered in the function registry.",
    "Check the function name for typos. Verify the FHIRPath specification for standard function names.",
);

static FP0055_INFO: ErrorInfo = ErrorInfo::new(
    55,
    "Property not found",
    "An attempt was made to access a property that doesn't exist on the given type.",
    "Check the FHIR specification for the correct property names. Verify the resource type and available properties.",
);

static FP0056_INFO: ErrorInfo = ErrorInfo::new(
    56,
    "Invalid property access",
    "Property access was attempted in an invalid context or manner.",
    "Ensure property access follows correct syntax and the target object supports the property.",
);

static FP0057_INFO: ErrorInfo = ErrorInfo::new(
    57,
    "Reference resolution failed",
    "Failed to resolve a reference to another resource or element.",
    "Check that the referenced resource exists and is accessible. Verify reference syntax and target availability.",
);

static FP0058_INFO: ErrorInfo = ErrorInfo::new(
    58,
    "Invalid type conversion",
    "An attempt was made to convert a value to an incompatible type.",
    "Check if the type conversion is supported. Some conversions may require intermediate steps or different conversion functions.",
);

static FP0059_INFO: ErrorInfo = ErrorInfo::new(
    59,
    "Collection index out of bounds",
    "An attempt was made to access a collection element at an invalid index.",
    "Ensure the index is within the bounds of the collection. Use functions like count() to check collection size.",
);

static FP0060_INFO: ErrorInfo = ErrorInfo::new(
    60,
    "Variable not defined",
    "An attempt was made to access a variable that has not been defined.",
    "Check that the variable is properly defined before use. Variables must be declared in the current scope.",
);

static FP0061_INFO: ErrorInfo = ErrorInfo::new(
    61,
    "Resource type mismatch",
    "Expression expects a specific resource type but the context contains a different resource type.",
    "Ensure the FHIRPath expression matches the resource type being evaluated. For example, use 'Patient.name' only when evaluating Patient resources, not Encounter resources.",
);

// Temporal/Date validation error information (FP0070-FP0080)
static FP0070_INFO: ErrorInfo = ErrorInfo::new(
    70,
    "Invalid date format",
    "The date string does not match the expected FHIR date format.",
    "Use the format YYYY-MM-DD for dates or YYYY-MM-DDTHH:MM:SSZ for datetimes. Check for typos in the date string.",
);

static FP0071_INFO: ErrorInfo = ErrorInfo::new(
    71,
    "Invalid date value (day > 31)",
    "The day component of the date is greater than 31, which is not valid.",
    "Ensure the day value is between 1 and 31, and valid for the given month (e.g., February cannot have 30 days).",
);

static FP0072_INFO: ErrorInfo = ErrorInfo::new(
    72,
    "Invalid month value (month > 12)",
    "The month component of the date is greater than 12, which is not valid.",
    "Ensure the month value is between 1 and 12 (January=1, December=12).",
);

static FP0073_INFO: ErrorInfo = ErrorInfo::new(
    73,
    "Invalid year value",
    "The year component of the date is outside the valid range for FHIR dates.",
    "Ensure the year is a valid 4-digit year. FHIR typically supports years 1900-2100.",
);

static FP0074_INFO: ErrorInfo = ErrorInfo::new(
    74,
    "Invalid time format",
    "The time string does not match the expected HH:MM:SS format.",
    "Use the format HH:MM:SS for times. Hours should be 00-23, minutes and seconds 00-59.",
);

static FP0075_INFO: ErrorInfo = ErrorInfo::new(
    75,
    "Invalid datetime format",
    "The datetime string does not match the expected ISO 8601 format.",
    "Use the format YYYY-MM-DDTHH:MM:SSZ or YYYY-MM-DDTHH:MM:SS+HH:MM for datetimes.",
);

static FP0076_INFO: ErrorInfo = ErrorInfo::new(
    76,
    "Date out of valid range",
    "The date is outside the valid range supported by the system.",
    "Check that the date falls within reasonable bounds (typically 1900-2100).",
);

static FP0077_INFO: ErrorInfo = ErrorInfo::new(
    77,
    "Invalid timezone format",
    "The timezone component of the datetime is not in the correct format.",
    "Use Z for UTC or +/-HH:MM format for timezone offsets (e.g., +05:30, -08:00).",
);

static FP0078_INFO: ErrorInfo = ErrorInfo::new(
    78,
    "Temporal precision mismatch",
    "The temporal value's precision does not match the expected precision for this operation.",
    "Ensure the date/datetime has the appropriate precision level for the operation being performed.",
);

static FP0079_INFO: ErrorInfo = ErrorInfo::new(
    79,
    "Invalid leap year date",
    "February 29th is only valid in leap years.",
    "Check if the year is a leap year before using February 29th. Leap years are divisible by 4, except century years which must be divisible by 400.",
);

static FP0080_INFO: ErrorInfo = ErrorInfo::new(
    80,
    "Temporal parsing error",
    "General error occurred while parsing a temporal (date/time) value.",
    "Check the format and content of the date/time string. Ensure it follows FHIR temporal format standards.",
);

// Model Provider Error Information (FP0101-FP0150)
static FP0101_INFO: ErrorInfo = ErrorInfo::new(
    101,
    "Resource not found",
    "The requested resource could not be found by the model provider.",
    "Verify the resource exists and is accessible. Check your model provider configuration and data sources.",
);

static FP0102_INFO: ErrorInfo = ErrorInfo::new(
    102,
    "Invalid resource type",
    "The specified resource type is not recognized or supported.",
    "Check that the resource type is a valid FHIR resource type and is supported by the model provider.",
);

static FP0103_INFO: ErrorInfo = ErrorInfo::new(
    103,
    "Property validation failed",
    "A property failed validation according to the model provider's schema.",
    "Check the property value against the FHIR specification constraints and data types.",
);

static FP0104_INFO: ErrorInfo = ErrorInfo::new(
    104,
    "Schema validation error",
    "The resource or data structure failed schema validation.",
    "Ensure the data structure conforms to the expected FHIR schema. Check required fields and data types.",
);

static FP0105_INFO: ErrorInfo = ErrorInfo::new(
    105,
    "ModelProvider connection failed",
    "The model provider failed to establish or maintain a connection to its data source.",
    "Check network connectivity, authentication credentials, and service availability. Verify model provider configuration.",
);

static FP0106_INFO: ErrorInfo = ErrorInfo::new(
    106,
    "Timeout in ModelProvider operation",
    "A ModelProvider operation exceeded the allowed time limit.",
    "Consider increasing timeout values or optimizing the query. Check network latency and data source performance.",
);

static FP0107_INFO: ErrorInfo = ErrorInfo::new(
    107,
    "Invalid FHIR resource",
    "The resource does not conform to valid FHIR structure or contains invalid data.",
    "Validate the resource against FHIR specifications. Check for missing required elements or invalid values.",
);

static FP0108_INFO: ErrorInfo = ErrorInfo::new(
    108,
    "Unsupported FHIR version",
    "The specified FHIR version is not supported by the model provider.",
    "Check which FHIR versions are supported and ensure compatibility. Consider upgrading or using a compatible version.",
);

static FP0109_INFO: ErrorInfo = ErrorInfo::new(
    109,
    "Missing required property",
    "A required property is missing from the resource or data structure.",
    "Check the FHIR specification for required properties and ensure they are present in the resource.",
);

static FP0110_INFO: ErrorInfo = ErrorInfo::new(
    110,
    "Invalid reference format",
    "A reference has invalid syntax or format.",
    "Check that references follow the correct FHIR reference format (e.g., 'ResourceType/id').",
);

static FP0121_INFO: ErrorInfo = ErrorInfo::new(
    121,
    "Unknown resource type",
    "The specified resource type is not recognized in the current FHIR schema.",
    "Check the FHIR specification for valid resource types. Consider using a spell checker for typos.",
);

static FP0122_INFO: ErrorInfo = ErrorInfo::new(
    122,
    "Resource type suggestion available",
    "A similar resource type was found that might be what you intended.",
    "Consider using the suggested resource type if it matches your intent.",
);

static FP0123_INFO: ErrorInfo = ErrorInfo::new(
    123,
    "Invalid resource type format",
    "The resource type name does not follow FHIR naming conventions.",
    "Resource types should be PascalCase without underscores (e.g., 'Patient', 'DiagnosticReport').",
);

static FP0124_INFO: ErrorInfo = ErrorInfo::new(
    124,
    "Unknown function name",
    "The specified function is not recognized in the FHIRPath function registry.",
    "Check the FHIRPath specification for valid function names. Consider checking for typos or using the suggested alternatives.",
);

static FP0125_INFO: ErrorInfo = ErrorInfo::new(
    125,
    "Invalid property access",
    "The specified property does not exist on the given type according to the FHIR schema.",
    "Check the FHIR specification for valid properties on this type. Consider using the suggested alternatives or checking for typos.",
);

// Analysis Error Information (FP0151-FP0200)
static FP0151_INFO: ErrorInfo = ErrorInfo::new(
    151,
    "Type inference failed",
    "The static analyzer was unable to infer the type of an expression or variable.",
    "Consider adding explicit type annotations or simplifying the expression to help with type inference.",
);

static FP0152_INFO: ErrorInfo = ErrorInfo::new(
    152,
    "Unreachable code detected",
    "Static analysis detected code that can never be executed.",
    "Review the control flow and remove or fix unreachable code sections. This often indicates logical errors.",
);

static FP0153_INFO: ErrorInfo = ErrorInfo::new(
    153,
    "Performance warning",
    "The analyzer detected a potentially inefficient expression or operation.",
    "Consider optimizing the expression for better performance. Check for unnecessary operations or inefficient patterns.",
);

static FP0154_INFO: ErrorInfo = ErrorInfo::new(
    154,
    "Optimization suggestion",
    "The analyzer suggests a more efficient alternative for the current expression.",
    "Review the suggested optimization to improve performance while maintaining correctness.",
);

static FP0155_INFO: ErrorInfo = ErrorInfo::new(
    155,
    "Static analysis error",
    "A general static analysis error occurred during expression analysis.",
    "Review the expression for potential issues with syntax, semantics, or logical structure.",
);

static FP0156_INFO: ErrorInfo = ErrorInfo::new(
    156,
    "Property type mismatch",
    "A property is being used in a context that doesn't match its expected type.",
    "Check that property usage aligns with the expected type from the FHIR specification.",
);

static FP0157_INFO: ErrorInfo = ErrorInfo::new(
    157,
    "Function signature mismatch",
    "A function call doesn't match any available function signature.",
    "Check the function name and argument types against available function signatures.",
);

static FP0158_INFO: ErrorInfo = ErrorInfo::new(
    158,
    "Dead code detected",
    "Code that will never be executed has been detected.",
    "Remove or fix dead code sections to improve maintainability and performance.",
);

static FP0159_INFO: ErrorInfo = ErrorInfo::new(
    159,
    "Inefficient expression pattern",
    "An inefficient expression pattern has been detected that could be optimized.",
    "Consider refactoring the expression using more efficient patterns or operations.",
);

static FP0160_INFO: ErrorInfo = ErrorInfo::new(
    160,
    "Missing type annotation",
    "A type annotation is missing where one is required or would be beneficial.",
    "Add explicit type annotations to improve clarity and help with static analysis.",
);

static FP0200_INFO: ErrorInfo = ErrorInfo::new(
    200,
    "System external error",
    "An external system error occurred that is outside the scope of FHIRPath evaluation.",
    "This indicates a system-level issue. Check logs and system status for more details.",
);

// Default fallback for unknown error codes
static UNKNOWN_ERROR_INFO: ErrorInfo = ErrorInfo::new(
    0,
    "Unknown error",
    "An unknown error occurred that is not registered in the error code system.",
    "This may indicate a bug in the library. Please report this issue with the error details.",
);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_code_formatting() {
        assert_eq!(FP0001.code_str(), "FP0001");
        assert_eq!(FP0051.code_str(), "FP0051");
        assert_eq!(FP0101.code_str(), "FP0101");
        assert_eq!(FP0151.code_str(), "FP0151");
    }

    #[test]
    fn test_error_info_retrieval() {
        let info = FP0001.info();
        assert_eq!(info.title, "Invalid FHIRPath syntax");
        assert!(info.description.contains("invalid syntax"));
    }

    #[test]
    fn test_docs_url() {
        assert_eq!(
            FP0001.docs_url(),
            "https://octofhir.github.io/fhirpath-rs/errors/FP0001"
        );
        assert_eq!(
            FP0051.docs_url(),
            "https://octofhir.github.io/fhirpath-rs/errors/FP0051"
        );
        assert_eq!(
            FP0101.docs_url(),
            "https://octofhir.github.io/fhirpath-rs/errors/FP0101"
        );
        assert_eq!(
            FP0151.docs_url(),
            "https://octofhir.github.io/fhirpath-rs/errors/FP0151"
        );
    }

    #[test]
    fn test_error_info_docs_url() {
        let info = FP0002.info();
        assert_eq!(
            info.docs_url(),
            "https://octofhir.github.io/fhirpath-rs/errors/FP0002"
        );
    }

    #[test]
    fn test_error_categories() {
        assert_eq!(FP0001.category(), ErrorCategory::Parser);
        assert_eq!(FP0010.category(), ErrorCategory::Parser);
        assert_eq!(FP0051.category(), ErrorCategory::Evaluation);
        assert_eq!(FP0060.category(), ErrorCategory::Evaluation);
        assert_eq!(FP0101.category(), ErrorCategory::ModelProvider);
        assert_eq!(FP0110.category(), ErrorCategory::ModelProvider);
        assert_eq!(FP0151.category(), ErrorCategory::Analysis);
        assert_eq!(FP0160.category(), ErrorCategory::Analysis);
    }

    #[test]
    fn test_error_descriptions() {
        assert_eq!(FP0001.description(), "Invalid FHIRPath syntax");
        assert_eq!(FP0002.description(), "Unexpected token in expression");
        assert_eq!(FP0051.description(), "Type mismatch in operation");
        assert_eq!(FP0101.description(), "Resource not found");
        assert_eq!(FP0151.description(), "Type inference failed");
    }

    #[test]
    fn test_all_parser_codes() {
        for code in 1..=10 {
            let error_code = ErrorCode::new(code);
            assert_eq!(error_code.category(), ErrorCategory::Parser);
            assert!(!error_code.description().is_empty());
        }
    }

    #[test]
    fn test_all_evaluation_codes() {
        for code in 51..=61 {
            let error_code = ErrorCode::new(code);
            assert_eq!(error_code.category(), ErrorCategory::Evaluation);
            assert!(!error_code.description().is_empty());
        }
    }

    #[test]
    fn test_all_model_provider_codes() {
        for code in 101..=110 {
            let error_code = ErrorCode::new(code);
            assert_eq!(error_code.category(), ErrorCategory::ModelProvider);
            assert!(!error_code.description().is_empty());
        }
    }

    #[test]
    fn test_all_analysis_codes() {
        for code in 151..=160 {
            let error_code = ErrorCode::new(code);
            assert_eq!(error_code.category(), ErrorCategory::Analysis);
            assert!(!error_code.description().is_empty());
        }
    }
}
