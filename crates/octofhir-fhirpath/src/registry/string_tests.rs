//! Tests for string functions

#[cfg(test)]
mod tests {
    use super::super::*;
    use crate::core::{FhirPathValue, ModelProvider};
    use crate::MockModelProvider;
    use std::collections::HashMap;
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
    fn test_contains_function() {
        let registry = FunctionRegistry::default();
        let dispatcher = dispatcher::FunctionDispatcher::new(registry);

        // Test successful contains
        let input = vec![FhirPathValue::String("Hello World".to_string())];
        let arguments = vec![FhirPathValue::String("World".to_string())];
        let context = create_test_context!(&input, &arguments);
        let result = dispatcher.dispatch_sync("contains", &context).unwrap();
        assert_eq!(result[0], FhirPathValue::Boolean(true));

        // Test unsuccessful contains
        let arguments = vec![FhirPathValue::String("Universe".to_string())];
        let context = create_test_context!(&input, &arguments);
        let result = dispatcher.dispatch_sync("contains", &context).unwrap();
        assert_eq!(result[0], FhirPathValue::Boolean(false));

        // Test case sensitivity
        let arguments = vec![FhirPathValue::String("world".to_string())];
        let context = create_test_context!(&input, &arguments);
        let result = dispatcher.dispatch_sync("contains", &context).unwrap();
        assert_eq!(result[0], FhirPathValue::Boolean(false));

        // Test empty substring
        let arguments = vec![FhirPathValue::String("".to_string())];
        let context = create_test_context!(&input, &arguments);
        let result = dispatcher.dispatch_sync("contains", &context).unwrap();
        assert_eq!(result[0], FhirPathValue::Boolean(true));

        // Test error: non-string input
        let input = vec![FhirPathValue::Integer(123)];
        let arguments = vec![FhirPathValue::String("test".to_string())];
        let context = create_test_context!(&input, &arguments);
        let result = dispatcher.dispatch_sync("contains", &context);
        assert!(result.is_err());
    }

    #[test]
    fn test_indexOf_function() {
        let registry = FunctionRegistry::default();
        let dispatcher = dispatcher::FunctionDispatcher::new(registry);

        // Test successful indexOf
        let input = vec![FhirPathValue::String("Hello World".to_string())];
        let arguments = vec![FhirPathValue::String("World".to_string())];
        let context = create_test_context!(&input, &arguments);
        let result = dispatcher.dispatch_sync("indexOf", &context).unwrap();
        assert_eq!(result[0], FhirPathValue::Integer(6));

        // Test unsuccessful indexOf
        let arguments = vec![FhirPathValue::String("Universe".to_string())];
        let context = create_test_context!(&input, &arguments);
        let result = dispatcher.dispatch_sync("indexOf", &context).unwrap();
        assert_eq!(result[0], FhirPathValue::Integer(-1));

        // Test indexOf at beginning
        let arguments = vec![FhirPathValue::String("Hello".to_string())];
        let context = create_test_context!(&input, &arguments);
        let result = dispatcher.dispatch_sync("indexOf", &context).unwrap();
        assert_eq!(result[0], FhirPathValue::Integer(0));

        // Test empty substring
        let arguments = vec![FhirPathValue::String("".to_string())];
        let context = create_test_context!(&input, &arguments);
        let result = dispatcher.dispatch_sync("indexOf", &context).unwrap();
        assert_eq!(result[0], FhirPathValue::Integer(0));

        // Test Unicode strings
        let input = vec![FhirPathValue::String("Héllo Wörld".to_string())];
        let arguments = vec![FhirPathValue::String("Wörld".to_string())];
        let context = create_test_context!(&input, &arguments);
        let result = dispatcher.dispatch_sync("indexOf", &context).unwrap();
        assert_eq!(result[0], FhirPathValue::Integer(6));
    }

    #[test]
    fn test_lastIndexOf_function() {
        let registry = FunctionRegistry::default();
        let dispatcher = dispatcher::FunctionDispatcher::new(registry);

        // Test lastIndexOf with multiple occurrences
        let input = vec![FhirPathValue::String("Hello World World".to_string())];
        let arguments = vec![FhirPathValue::String("World".to_string())];
        let context = create_test_context!(&input, &arguments);
        let result = dispatcher.dispatch_sync("lastIndexOf", &context).unwrap();
        assert_eq!(result[0], FhirPathValue::Integer(12));

        // Test lastIndexOf with single occurrence
        let arguments = vec![FhirPathValue::String("Hello".to_string())];
        let context = create_test_context!(&input, &arguments);
        let result = dispatcher.dispatch_sync("lastIndexOf", &context).unwrap();
        assert_eq!(result[0], FhirPathValue::Integer(0));

        // Test lastIndexOf not found
        let arguments = vec![FhirPathValue::String("Universe".to_string())];
        let context = create_test_context!(&input, &arguments);
        let result = dispatcher.dispatch_sync("lastIndexOf", &context).unwrap();
        assert_eq!(result[0], FhirPathValue::Integer(-1));
    }

    #[test]
    fn test_substring_function() {
        let registry = FunctionRegistry::default();
        let dispatcher = dispatcher::FunctionDispatcher::new(registry);

        let input = vec![FhirPathValue::String("Hello World".to_string())];

        // Test substring with start index only
        let arguments = vec![FhirPathValue::Integer(6)];
        let context = create_test_context!(&input, &arguments);
        let result = dispatcher.dispatch_sync("substring", &context).unwrap();
        assert_eq!(result[0], FhirPathValue::String("World".to_string()));

        // Test substring with start and length
        let arguments = vec![FhirPathValue::Integer(0), FhirPathValue::Integer(5)];
        let context = create_test_context!(&input, &arguments);
        let result = dispatcher.dispatch_sync("substring", &context).unwrap();
        assert_eq!(result[0], FhirPathValue::String("Hello".to_string()));

        // Test substring with start beyond string length
        let arguments = vec![FhirPathValue::Integer(20)];
        let context = create_test_context!(&input, &arguments);
        let result = dispatcher.dispatch_sync("substring", &context).unwrap();
        assert_eq!(result[0], FhirPathValue::String("".to_string()));

        // Test substring with Unicode
        let input = vec![FhirPathValue::String("Héllo Wörld".to_string())];
        let arguments = vec![FhirPathValue::Integer(6), FhirPathValue::Integer(5)];
        let context = create_test_context!(&input, &arguments);
        let result = dispatcher.dispatch_sync("substring", &context).unwrap();
        assert_eq!(result[0], FhirPathValue::String("Wörld".to_string()));

        // Test error: negative start index
        let input = vec![FhirPathValue::String("Hello".to_string())];
        let arguments = vec![FhirPathValue::Integer(-1)];
        let context = create_test_context!(&input, &arguments);
        let result = dispatcher.dispatch_sync("substring", &context);
        assert!(result.is_err());
    }

    #[test]
    fn test_startsWith_function() {
        let registry = FunctionRegistry::default();
        let dispatcher = dispatcher::FunctionDispatcher::new(registry);

        let input = vec![FhirPathValue::String("Hello World".to_string())];

        // Test successful startsWith
        let arguments = vec![FhirPathValue::String("Hello".to_string())];
        let context = create_test_context!(&input, &arguments);
        let result = dispatcher.dispatch_sync("startsWith", &context).unwrap();
        assert_eq!(result[0], FhirPathValue::Boolean(true));

        // Test unsuccessful startsWith
        let arguments = vec![FhirPathValue::String("World".to_string())];
        let context = create_test_context!(&input, &arguments);
        let result = dispatcher.dispatch_sync("startsWith", &context).unwrap();
        assert_eq!(result[0], FhirPathValue::Boolean(false));

        // Test empty prefix
        let arguments = vec![FhirPathValue::String("".to_string())];
        let context = create_test_context!(&input, &arguments);
        let result = dispatcher.dispatch_sync("startsWith", &context).unwrap();
        assert_eq!(result[0], FhirPathValue::Boolean(true));
    }

    #[test]
    fn test_endsWith_function() {
        let registry = FunctionRegistry::default();
        let dispatcher = dispatcher::FunctionDispatcher::new(registry);

        let input = vec![FhirPathValue::String("Hello World".to_string())];

        // Test successful endsWith
        let arguments = vec![FhirPathValue::String("World".to_string())];
        let context = create_test_context!(&input, &arguments);
        let result = dispatcher.dispatch_sync("endsWith", &context).unwrap();
        assert_eq!(result[0], FhirPathValue::Boolean(true));

        // Test unsuccessful endsWith
        let arguments = vec![FhirPathValue::String("Hello".to_string())];
        let context = create_test_context!(&input, &arguments);
        let result = dispatcher.dispatch_sync("endsWith", &context).unwrap();
        assert_eq!(result[0], FhirPathValue::Boolean(false));

        // Test empty suffix
        let arguments = vec![FhirPathValue::String("".to_string())];
        let context = create_test_context!(&input, &arguments);
        let result = dispatcher.dispatch_sync("endsWith", &context).unwrap();
        assert_eq!(result[0], FhirPathValue::Boolean(true));
    }

    #[test]
    fn test_upper_lower_functions() {
        let registry = FunctionRegistry::default();
        let dispatcher = dispatcher::FunctionDispatcher::new(registry);

        let input = vec![FhirPathValue::String("Hello World".to_string())];
        let context = create_test_context!(&input, &[]);

        // Test upper
        let result = dispatcher.dispatch_sync("upper", &context).unwrap();
        assert_eq!(result[0], FhirPathValue::String("HELLO WORLD".to_string()));

        // Test lower
        let result = dispatcher.dispatch_sync("lower", &context).unwrap();
        assert_eq!(result[0], FhirPathValue::String("hello world".to_string()));

        // Test Unicode case conversion
        let input = vec![FhirPathValue::String("Héllo Wörld".to_string())];
        let context = create_test_context!(&input, &[]);
        let result = dispatcher.dispatch_sync("upper", &context).unwrap();
        assert_eq!(result[0], FhirPathValue::String("HÉLLO WÖRLD".to_string()));
    }

    #[test]
    fn test_length_function() {
        let registry = FunctionRegistry::default();
        let dispatcher = dispatcher::FunctionDispatcher::new(registry);

        // Test basic length
        let input = vec![FhirPathValue::String("Hello".to_string())];
        let context = create_test_context!(&input, &[]);
        let result = dispatcher.dispatch_sync("length", &context).unwrap();
        assert_eq!(result[0], FhirPathValue::Integer(5));

        // Test empty string
        let input = vec![FhirPathValue::String("".to_string())];
        let context = create_test_context!(&input, &[]);
        let result = dispatcher.dispatch_sync("length", &context).unwrap();
        assert_eq!(result[0], FhirPathValue::Integer(0));

        // Test Unicode string length (character count, not byte count)
        let input = vec![FhirPathValue::String("Héllo".to_string())];
        let context = create_test_context!(&input, &[]);
        let result = dispatcher.dispatch_sync("length", &context).unwrap();
        assert_eq!(result[0], FhirPathValue::Integer(5));

        // Test with emojis
        let input = vec![FhirPathValue::String("Hello 👋 World 🌎".to_string())];
        let context = create_test_context!(&input, &[]);
        let result = dispatcher.dispatch_sync("length", &context).unwrap();
        assert_eq!(result[0], FhirPathValue::Integer(15)); // 15 Unicode code points
    }

    #[test]
    fn test_trim_function() {
        let registry = FunctionRegistry::default();
        let dispatcher = dispatcher::FunctionDispatcher::new(registry);

        // Test basic trim
        let input = vec![FhirPathValue::String("  hello world  ".to_string())];
        let context = create_test_context!(&input, &[]);
        let result = dispatcher.dispatch_sync("trim", &context).unwrap();
        assert_eq!(result[0], FhirPathValue::String("hello world".to_string()));

        // Test trim with no whitespace
        let input = vec![FhirPathValue::String("hello".to_string())];
        let context = create_test_context!(&input, &[]);
        let result = dispatcher.dispatch_sync("trim", &context).unwrap();
        assert_eq!(result[0], FhirPathValue::String("hello".to_string()));

        // Test trim with only whitespace
        let input = vec![FhirPathValue::String("   ".to_string())];
        let context = create_test_context!(&input, &[]);
        let result = dispatcher.dispatch_sync("trim", &context).unwrap();
        assert_eq!(result[0], FhirPathValue::String("".to_string()));
    }

    #[test]
    fn test_replace_function() {
        let registry = FunctionRegistry::default();
        let dispatcher = dispatcher::FunctionDispatcher::new(registry);

        // Test basic replace
        let input = vec![FhirPathValue::String("Hello World World".to_string())];
        let arguments = vec![
            FhirPathValue::String("World".to_string()),
            FhirPathValue::String("Universe".to_string())
        ];
        let context = create_test_context!(&input, &arguments);
        let result = dispatcher.dispatch_sync("replace", &context).unwrap();
        assert_eq!(result[0], FhirPathValue::String("Hello Universe Universe".to_string()));

        // Test replace with no match
        let arguments = vec![
            FhirPathValue::String("xyz".to_string()),
            FhirPathValue::String("abc".to_string())
        ];
        let context = create_test_context!(&input, &arguments);
        let result = dispatcher.dispatch_sync("replace", &context).unwrap();
        assert_eq!(result[0], FhirPathValue::String("Hello World World".to_string()));

        // Test replace with empty string
        let arguments = vec![
            FhirPathValue::String("World".to_string()),
            FhirPathValue::String("".to_string())
        ];
        let context = create_test_context!(&input, &arguments);
        let result = dispatcher.dispatch_sync("replace", &context).unwrap();
        assert_eq!(result[0], FhirPathValue::String("Hello  ".to_string()));
    }

    #[test]
    fn test_matches_function() {
        let registry = FunctionRegistry::default();
        let dispatcher = dispatcher::FunctionDispatcher::new(registry);

        // Test email regex match
        let input = vec![FhirPathValue::String("hello@example.com".to_string())];
        let arguments = vec![FhirPathValue::String(r"^[a-z]+@[a-z]+\.[a-z]+$".to_string())];
        let context = create_test_context!(&input, &arguments);
        let result = dispatcher.dispatch_sync("matches", &context).unwrap();
        assert_eq!(result[0], FhirPathValue::Boolean(true));

        // Test non-matching pattern
        let arguments = vec![FhirPathValue::String(r"^[0-9]+$".to_string())];
        let context = create_test_context!(&input, &arguments);
        let result = dispatcher.dispatch_sync("matches", &context).unwrap();
        assert_eq!(result[0], FhirPathValue::Boolean(false));

        // Test phone number regex
        let input = vec![FhirPathValue::String("+1-555-123-4567".to_string())];
        let arguments = vec![FhirPathValue::String(r"^\+1-[0-9]{3}-[0-9]{3}-[0-9]{4}$".to_string())];
        let context = create_test_context!(&input, &arguments);
        let result = dispatcher.dispatch_sync("matches", &context).unwrap();
        assert_eq!(result[0], FhirPathValue::Boolean(true));

        // Test invalid regex (should error)
        let input = vec![FhirPathValue::String("test".to_string())];
        let arguments = vec![FhirPathValue::String("[invalid".to_string())]; // Invalid regex
        let context = create_test_context!(&input, &arguments);
        let result = dispatcher.dispatch_sync("matches", &context);
        assert!(result.is_err());
    }

    #[test]
    fn test_replaceMatches_function() {
        let registry = FunctionRegistry::default();
        let dispatcher = dispatcher::FunctionDispatcher::new(registry);

        // Test replace numbers with XXX
        let input = vec![FhirPathValue::String("Hello 123 World 456".to_string())];
        let arguments = vec![
            FhirPathValue::String(r"[0-9]+".to_string()),
            FhirPathValue::String("XXX".to_string())
        ];
        let context = create_test_context!(&input, &arguments);
        let result = dispatcher.dispatch_sync("replaceMatches", &context).unwrap();
        assert_eq!(result[0], FhirPathValue::String("Hello XXX World XXX".to_string()));

        // Test normalize whitespace
        let input = vec![FhirPathValue::String("Hello   World\t\nTest".to_string())];
        let arguments = vec![
            FhirPathValue::String(r"\s+".to_string()),
            FhirPathValue::String(" ".to_string())
        ];
        let context = create_test_context!(&input, &arguments);
        let result = dispatcher.dispatch_sync("replaceMatches", &context).unwrap();
        assert_eq!(result[0], FhirPathValue::String("Hello World Test".to_string()));
    }

    #[test]
    fn test_split_function() {
        let registry = FunctionRegistry::default();
        let dispatcher = dispatcher::FunctionDispatcher::new(registry);

        // Test basic split
        let input = vec![FhirPathValue::String("a,b,c".to_string())];
        let arguments = vec![FhirPathValue::String(",".to_string())];
        let context = create_test_context!(&input, &arguments);
        let result = dispatcher.dispatch_sync("split", &context).unwrap();
        assert_eq!(result.len(), 3);
        assert_eq!(result[0], FhirPathValue::String("a".to_string()));
        assert_eq!(result[1], FhirPathValue::String("b".to_string()));
        assert_eq!(result[2], FhirPathValue::String("c".to_string()));

        // Test split by space
        let input = vec![FhirPathValue::String("Hello World Test".to_string())];
        let arguments = vec![FhirPathValue::String(" ".to_string())];
        let context = create_test_context!(&input, &arguments);
        let result = dispatcher.dispatch_sync("split", &context).unwrap();
        assert_eq!(result.len(), 3);
        assert_eq!(result[0], FhirPathValue::String("Hello".to_string()));
        assert_eq!(result[1], FhirPathValue::String("World".to_string()));
        assert_eq!(result[2], FhirPathValue::String("Test".to_string()));

        // Test split by empty string (should split into characters)
        let input = vec![FhirPathValue::String("abc".to_string())];
        let arguments = vec![FhirPathValue::String("".to_string())];
        let context = create_test_context!(&input, &arguments);
        let result = dispatcher.dispatch_sync("split", &context).unwrap();
        assert_eq!(result.len(), 3);
        assert_eq!(result[0], FhirPathValue::String("a".to_string()));
        assert_eq!(result[1], FhirPathValue::String("b".to_string()));
        assert_eq!(result[2], FhirPathValue::String("c".to_string()));
    }

    #[test]
    fn test_join_function() {
        let registry = FunctionRegistry::default();
        let dispatcher = dispatcher::FunctionDispatcher::new(registry);

        // Test basic join
        let input = vec![
            FhirPathValue::String("a".to_string()),
            FhirPathValue::String("b".to_string()),
            FhirPathValue::String("c".to_string()),
        ];
        let arguments = vec![FhirPathValue::String(",".to_string())];
        let context = create_test_context!(&input, &arguments);
        let result = dispatcher.dispatch_sync("join", &context).unwrap();
        assert_eq!(result[0], FhirPathValue::String("a,b,c".to_string()));

        // Test join with space
        let arguments = vec![FhirPathValue::String(" ".to_string())];
        let context = create_test_context!(&input, &arguments);
        let result = dispatcher.dispatch_sync("join", &context).unwrap();
        assert_eq!(result[0], FhirPathValue::String("a b c".to_string()));

        // Test join mixed types (should convert to strings)
        let input = vec![
            FhirPathValue::String("hello".to_string()),
            FhirPathValue::Integer(123),
            FhirPathValue::Boolean(true),
        ];
        let arguments = vec![FhirPathValue::String(" ".to_string())];
        let context = create_test_context!(&input, &arguments);
        let result = dispatcher.dispatch_sync("join", &context).unwrap();
        assert_eq!(result[0], FhirPathValue::String("hello 123 true".to_string()));

        // Test join empty collection
        let input = vec![];
        let arguments = vec![FhirPathValue::String(",".to_string())];
        let context = create_test_context!(&input, &arguments);
        let result = dispatcher.dispatch_sync("join", &context).unwrap();
        assert_eq!(result[0], FhirPathValue::String("".to_string()));
    }

    #[test]
    fn test_toChars_function() {
        let registry = FunctionRegistry::default();
        let dispatcher = dispatcher::FunctionDispatcher::new(registry);

        // Test basic toChars
        let input = vec![FhirPathValue::String("Hello".to_string())];
        let context = create_test_context!(&input, &[]);
        let result = dispatcher.dispatch_sync("toChars", &context).unwrap();
        assert_eq!(result.len(), 5);
        assert_eq!(result[0], FhirPathValue::String("H".to_string()));
        assert_eq!(result[1], FhirPathValue::String("e".to_string()));
        assert_eq!(result[2], FhirPathValue::String("l".to_string()));
        assert_eq!(result[3], FhirPathValue::String("l".to_string()));
        assert_eq!(result[4], FhirPathValue::String("o".to_string()));

        // Test empty string
        let input = vec![FhirPathValue::String("".to_string())];
        let context = create_test_context!(&input, &[]);
        let result = dispatcher.dispatch_sync("toChars", &context).unwrap();
        assert_eq!(result.len(), 0);

        // Test Unicode characters
        let input = vec![FhirPathValue::String("Hi👋".to_string())];
        let context = create_test_context!(&input, &[]);
        let result = dispatcher.dispatch_sync("toChars", &context).unwrap();
        assert_eq!(result.len(), 3);
        assert_eq!(result[0], FhirPathValue::String("H".to_string()));
        assert_eq!(result[1], FhirPathValue::String("i".to_string()));
        assert_eq!(result[2], FhirPathValue::String("👋".to_string()));
    }

    #[test]
    fn test_string_utils() {
        use super::super::string::StringUtils;

        // Test to_string_value
        assert_eq!(
            StringUtils::to_string_value(&FhirPathValue::String("test".to_string())).unwrap(),
            "test"
        );
        assert_eq!(
            StringUtils::to_string_value(&FhirPathValue::Integer(123)).unwrap(),
            "123"
        );
        assert_eq!(
            StringUtils::to_string_value(&FhirPathValue::Boolean(true)).unwrap(),
            "true"
        );

        // Test safe_substring
        assert_eq!(StringUtils::safe_substring("Hello", 0, Some(3)), "Hel");
        assert_eq!(StringUtils::safe_substring("Hello", 2, None), "llo");
        assert_eq!(StringUtils::safe_substring("Hello", 10, Some(5)), "");
        
        // Test Unicode safe_substring
        assert_eq!(StringUtils::safe_substring("Héllo", 1, Some(3)), "éll");
    }

    #[test]
    fn test_error_conditions() {
        let registry = FunctionRegistry::default();
        let dispatcher = dispatcher::FunctionDispatcher::new(registry);

        // Test multiple input values (should error for single-value functions)
        let input = vec![
            FhirPathValue::String("hello".to_string()),
            FhirPathValue::String("world".to_string())
        ];
        let context = create_test_context!(&input, &[]);
        assert!(dispatcher.dispatch_sync("length", &context).is_err());

        // Test wrong argument count
        let input = vec![FhirPathValue::String("hello".to_string())];
        let context = create_test_context!(&input, &[]);
        assert!(dispatcher.dispatch_sync("contains", &context).is_err()); // Missing argument

        // Test wrong argument types
        let arguments = vec![FhirPathValue::Integer(123)];
        let context = create_test_context!(&input, &arguments);
        assert!(dispatcher.dispatch_sync("contains", &context).is_err()); // Non-string argument
    }
}
