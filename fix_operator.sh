#!/bin/bash

# Script to apply binary operator utils fix to multiple operators
# Usage: ./fix_operator.sh operator_file_path

if [ $# -ne 1 ]; then
    echo "Usage: $0 <operator_file_path>"
    exit 1
fi

OPERATOR_FILE="$1"

if [ ! -f "$OPERATOR_FILE" ]; then
    echo "File $OPERATOR_FILE does not exist"
    exit 1
fi

echo "Fixing operator: $OPERATOR_FILE"

# Add binary_operator_utils import
sed -i '' 's/use crate::operations::EvaluationContext;/use crate::operations::{EvaluationContext, binary_operator_utils};/' "$OPERATOR_FILE"

# Find the comparison function name
COMPARE_FUNC=$(grep -o 'pub fn compare_[a-z_]*' "$OPERATOR_FILE" | head -1 | sed 's/pub fn //')

if [ -z "$COMPARE_FUNC" ]; then
    echo "Could not find comparison function in $OPERATOR_FILE"
    exit 1
fi

echo "Found comparison function: $COMPARE_FUNC"

# Replace async evaluate method
sed -i '' "/async fn evaluate.*{/,/^    }/ {
    /async fn evaluate.*{/,/^    }/ c\\
    async fn evaluate(\&self, args: \&[FhirPathValue], _context: \&EvaluationContext) -> Result<FhirPathValue> {\\
        if args.len() != 2 {\\
            return Err(FhirPathError::InvalidArgumentCount { \\
                function_name: self.identifier().to_string(), \\
                expected: 2, \\
                actual: args.len() \\
            });\\
        }\\
\\
        binary_operator_utils::evaluate_binary_operator(\&args[0], \&args[1], Self::$COMPARE_FUNC)\\
    }
}" "$OPERATOR_FILE"

# Replace try_evaluate_sync method  
sed -i '' "/fn try_evaluate_sync.*{/,/^    }/ {
    /fn try_evaluate_sync.*{/,/^    }/ c\\
    fn try_evaluate_sync(\&self, args: \&[FhirPathValue], _context: \&EvaluationContext) -> Option<Result<FhirPathValue>> {\\
        if args.len() != 2 {\\
            return Some(Err(FhirPathError::InvalidArgumentCount { \\
                function_name: self.identifier().to_string(), \\
                expected: 2, \\
                actual: args.len() \\
            }));\\
        }\\
\\
        Some(binary_operator_utils::evaluate_binary_operator(\&args[0], \&args[1], Self::$COMPARE_FUNC))\\
    }
}" "$OPERATOR_FILE"

echo "Fixed $OPERATOR_FILE successfully"