# Java FHIRPath Implementation Analysis

This document analyzes the official Java FHIRPath implementation from the HL7 FHIR Core library (org.hl7.fhir.r5.fhirpath package).

## Architecture Overview

The Java implementation follows a traditional interpreter pattern with these main components:

1. **Lexer** (FHIRLexer) - Tokenizes input strings
2. **Parser** (in FHIRPathEngine) - Builds AST from tokens
3. **Type Checker** (via TypeDetails) - Validates types before evaluation
4. **Evaluator** (FHIRPathEngine) - Executes the AST

## AST Structure

### ExpressionNode Class

The core AST node class with a discriminated union design:

```java
public class ExpressionNode {
    // Node identification
    private String uniqueId;           // Unique identifier for debugging
    private Kind kind;                 // Type of node (Name, Function, etc.)
    
    // Content fields (used based on kind)
    private String name;               // For Name nodes: identifier name
    private Base constant;             // For Constant nodes: literal value
    private Function function;         // For Function nodes: function type
    private ExpressionNode group;      // For Group nodes: grouped expression
    
    // Structure fields
    private List<ExpressionNode> parameters;  // Function arguments
    private ExpressionNode inner;             // Next in chain (a.b.c)
    private Operation operation;              // Binary operator type
    private ExpressionNode opNext;            // Right operand for binary ops
    
    // Metadata
    private boolean proximal;          // True if first in operation chain
    private Types types;               // Legacy type information
    private TypeDetails typesD;        // Detailed type information
    private SourceLocation start;      // Start position in source
    private SourceLocation end;        // End position in source
}
```

#### Field Relationships

- **inner**: Used for chaining - `Patient.name.given` creates nodes where each `inner` points to the next
- **operation + opNext**: Binary operations store operator and right operand
- **group**: Parentheses create Group nodes containing the grouped expression
- **parameters**: Function nodes store their arguments here

### Node Types (Kind enum)

```java
public enum Kind {
    Name,      // Variable/property reference
    Function,  // Function call
    Constant,  // Literal value
    Group,     // Parenthesized expression
    Unary      // Unary +/-
}
```

### Operations (Binary)

```java
public enum Operation {
    Equals, Equivalent, NotEquals, NotEquivalent,
    LessThan, Greater, LessOrEqual, GreaterOrEqual,
    Is, As, Union, Or, And, Xor, Implies,
    Plus, Times, Minus, Concatenate, DivideBy,
    Div, Mod, In, Contains, MemberOf
}
```

### Functions

Extensive enum with 60+ functions:

```java
public enum Function {
    // Collection operations
    Where, Select, First, Last, Tail, Take, Skip, Distinct,
    Count, Aggregate, Distinct, IsDistinct,
    
    // Logic operations  
    Empty, Not, Exists, All, AllTrue, AnyTrue, AllFalse, AnyFalse,
    SubsetOf, SupersetOf, Iif,
    
    // String operations
    Upper, Lower, ToChars, IndexOf, Substring, StartsWith, EndsWith,
    Matches, MatchesFull, ReplaceMatches, Contains, Replace, Length,
    Split, Join, Trim, Encode, Decode, Escape, Unescape,
    
    // Type operations
    OfType, Type, As, Is, ConformsTo,
    ConvertsToBoolean, ConvertsToInteger, ConvertsToString,
    ConvertsToDecimal, ConvertsToQuantity, ConvertsToDateTime,
    ConvertsToDate, ConvertsToTime,
    ToBoolean, ToInteger, ToString, ToDecimal, ToQuantity,
    ToDateTime, ToDate, ToTime,
    
    // Math operations
    Abs, Ceiling, Exp, Floor, Ln, Log, Power, Round, Sqrt, Truncate,
    
    // Date/Time operations
    Now, Today, TimeOfDay, 
    
    // Navigation
    Children, Descendants, Resolve,
    
    // FHIR-specific
    Extension, HasValue, HtmlChecks, Comparable,
    HasTemplateIdOf, MemberOf, DefineVariable, Check, Trace,
    
    // Aggregate functions
    Sum, Min, Max, Avg, 
    
    // Collection combining
    Union, Combine, Intersect, Exclude,
    
    // Other
    Item, Single, Repeat, Sort, 
    LowBoundary, HighBoundary, Precision
}
```

## Lexer (FHIRLexer)

Hand-written lexer with rich token support:

### Token Types
- **Identifiers**: `[A-Za-z_][A-Za-z0-9_]*`
- **Numbers**: Integers and decimals
- **Strings**: Single quotes (standard), double quotes (optional), backticks (fixed names)
- **Date/Time**: `@` prefix (e.g., `@2023-01-01`)
- **Quantities**: Number + unit (e.g., `5 'mg'`)
- **External constants**: `%` prefix (e.g., `%ucum`)
- **Operators**: All standard operators
- **Comments**: `//` line, `/* */` block

### Special Features
- Location tracking for error reporting
- Unicode support
- Escape sequences in strings
- UCUM unit validation

## Parser

Recursive descent parser integrated into FHIRPathEngine:

### Parsing Strategy

1. **Primary expression parsing**: Constants, names, function calls, groups
2. **Postfix operations**: Member access (`.`), indexing (`[]`)
3. **Binary operations**: With precedence handling
4. **Precedence reorganization**: Via `organisePrecedence()` method

### Operator Precedence (lowest to highest)

1. `implies`
2. `or`, `xor`
3. `and`
4. `in`, `contains`, `memberOf`
5. `=`, `~`, `!=`, `!~`
6. `is`, `as`
7. `<`, `>`, `<=`, `>=`
8. `|` (union)
9. `+`, `-`, `&`
10. `*`, `/`, `div`, `mod`
11. Unary `-`, `+`
12. Postfix `.`, `()`, `[]`

### Key Parsing Methods

```java
private ExpressionNode parseExpression(FHIRLexer lexer, boolean proximal) {
    ExpressionNode node = new ExpressionNode(lexer.nextId());
    // ... parse logic
    return organisePrecedence(lexer, node);
}
```

## Parser Grammar (Deduced from Implementation)

Based on analysis of the Java implementation, here's the FHIRPath grammar in EBNF notation:

```ebnf
(* Main expression entry point *)
Expression ::= UnaryExpression (BinaryOperator UnaryExpression)*

(* Unary expressions *)
UnaryExpression ::= [UnaryOperator] PostfixExpression
UnaryOperator ::= '+' | '-'

(* Postfix expressions *)
PostfixExpression ::= PrimaryExpression (Invocation | IndexerExpression | '.' Expression)*

(* Primary expressions *)
PrimaryExpression ::= Literal
                    | Identifier
                    | FunctionCall
                    | '(' Expression ')'

(* Literals *)
Literal ::= NullLiteral
          | BooleanLiteral
          | NumberLiteral
          | DateTimeLiteral
          | StringLiteral
          | QuantityLiteral

NullLiteral ::= '{' '}'
BooleanLiteral ::= 'true' | 'false'
NumberLiteral ::= Integer | Decimal
Integer ::= ['+' | '-'] Digit+
Decimal ::= ['+' | '-'] Digit+ '.' Digit+
DateTimeLiteral ::= '@' DateTimeFormat
StringLiteral ::= '"' DoubleQuotedChar* '"'
                | '\'' SingleQuotedChar* '\''
                | '`' BacktickedChar* '`'
QuantityLiteral ::= NumberLiteral (Unit | StringLiteral)

(* Identifiers *)
Identifier ::= Alpha (Alpha | Digit | '_')*
             | '$' ('this' | 'total' | 'index')
             | '%' (Alpha (Alpha | Digit | ':' | '-' | '_')*)
             | '`' BacktickedChar* '`'

(* Function calls *)
FunctionCall ::= Identifier '(' ArgumentList? ')'
ArgumentList ::= Expression (',' Expression)*

(* Indexer *)
IndexerExpression ::= '[' Expression ']'

(* Binary operators by precedence *)
BinaryOperator ::= 'implies'                    (* Lowest precedence *)
                 | 'or' | 'xor'                 
                 | 'and'                        
                 | 'in' | 'contains' | 'memberOf'
                 | '=' | '~' | '!=' | '!~'      
                 | 'is' | 'as'                  
                 | '<' | '>' | '<=' | '>='      
                 | '|'                          
                 | '+' | '-' | '&'              
                 | '*' | '/' | 'div' | 'mod'    (* Highest precedence *)
```

### Grammar Notes

1. **Expression Chaining**: The grammar supports chaining through postfix operations (`.` for member access, `[]` for indexing, `()` for function calls).

2. **Operator Precedence**: The `organisePrecedence` method reorganizes the flat parsed structure into a properly nested tree based on operator precedence.

3. **Special Identifiers**:
   - `$this`, `$total`, `$index` - Special variables
   - `%name` - External constants
   - Backtick-quoted identifiers for names with special characters

4. **Quantity Literals**: Numbers can be followed by units (either predefined units or string literals).

5. **Comments**: Not shown in the grammar but supported: `//` for line comments and `/* */` for block comments.

## Grammar Comparison with Official Specification

### Key Differences Between Java Implementation and Official Grammar

The Java implementation's parser differs from the official FHIRPath grammar (fhirpath.g4) in several important ways:

#### 1. **Grammar Structure**
- **Official**: Uses ANTLR4 grammar with labeled alternatives and explicit precedence through rule hierarchy
- **Java**: Hand-written recursive descent parser with precedence handled via `organisePrecedence()` method

#### 2. **Expression Hierarchy**
- **Official**: Defines precedence through grammar rules (termExpression → polarityExpression → multiplicativeExpression → etc.)
- **Java**: Parses flat then reorganizes based on operator precedence

#### 3. **Operator Precedence Order**
Both follow similar precedence, but with slight differences:

**Official Grammar** (highest to lowest):
1. Term (literals, invocations, parentheses)
2. Postfix (`.`, `[]`)
3. Unary (`+`, `-`)
4. Multiplicative (`*`, `/`, `div`, `mod`)
5. Additive (`+`, `-`, `&`)
6. Type (`is`, `as`)
7. Union (`|`)
8. Inequality (`<`, `>`, `<=`, `>=`)
9. Equality (`=`, `~`, `!=`, `!~`)
10. Membership (`in`, `contains`)
11. And (`and`)
12. Or/Xor (`or`, `xor`)
13. Implies (`implies`)

**Java Implementation** (highest to lowest):
1. Multiplication/Division (`*`, `/`, `div`, `mod`)
2. Addition/Subtraction/Concatenation (`+`, `-`, `&`)
3. Union (`|`)
4. Comparison (`<`, `>`, `<=`, `>=`)
5. Type operators (`is`, `as`)
6. Equality (`=`, `~`, `!=`, `!~`)
7. Membership (`in`, `contains`, `memberOf`)
8. Logical AND (`and`)
9. Logical XOR/OR (`xor`, `or`)
10. Implies (`implies`)

#### 4. **Additional Features in Java**
- **`memberOf` operator**: Not in official grammar, added for FHIR terminology support
- **Double-quoted strings**: Optional support beyond single quotes
- **Special parsing modes**: Metadata format (`///`), liquid mode (`||`)

#### 5. **Term Structure**
- **Official**: `term` can be invocation, literal, externalConstant, or parenthesized expression
- **Java**: Primary expressions include names directly (not just through invocation)

#### 6. **Invocation Handling**
- **Official**: Invocations include identifiers, functions, and special variables as separate productions
- **Java**: Treats property access and function calls more uniformly in parsing

#### 7. **Type Specifier**
- **Official**: Supports qualified identifiers for types (e.g., `FHIR.Patient`)
- **Java**: Handles this but not explicitly shown in deduced grammar

#### 8. **Missing from Deduced Grammar**
- Lambda expressions (commented out in official grammar)
- Explicit `qualifiedIdentifier` production
- Separate `dateTimePrecision` and `pluralDateTimePrecision` rules

#### 9. **Lexical Differences**
- **Official**: `NUMBER` allows leading zeros
- **Java**: More restrictive number parsing
- **Official**: `IDENTIFIER` includes underscore
- **Java**: Same, but also handles reserved words differently

#### 10. **Implementation-Specific Behavior**
The Java parser includes several behaviors not specified in the grammar:
- Automatic semicolon insertion in some contexts
- Special handling for FHIR-specific constructs
- More lenient string parsing with multiple quote styles
- Context-sensitive keyword handling

### Summary
While the Java implementation follows the spirit of the official grammar, it takes a more pragmatic approach with a hand-written parser that handles precedence through reorganization rather than grammar structure. The core language features are preserved, but with FHIR-specific extensions and some variations in precedence handling.

## AST Examples

Here are examples showing how different FHIRPath expressions are represented in the AST:

### Example 1: Simple Property Access
**FHIRPath**: `Patient.name`
```java
ExpressionNode {
  kind: Name
  name: "Patient"
  inner: ExpressionNode {
    kind: Name
    name: "name"
  }
}
```

### Example 2: Chained Navigation
**FHIRPath**: `Patient.name.given.first()`
```java
ExpressionNode {
  kind: Name
  name: "Patient"
  inner: ExpressionNode {
    kind: Name
    name: "name"
    inner: ExpressionNode {
      kind: Name
      name: "given"
      inner: ExpressionNode {
        kind: Function
        function: First
        parameters: []
      }
    }
  }
}
```

### Example 3: Function with Predicate
**FHIRPath**: `name.where(use = 'official')`
```java
ExpressionNode {
  kind: Name
  name: "name"
  inner: ExpressionNode {
    kind: Function
    function: Where
    parameters: [
      ExpressionNode {
        kind: Name
        name: "use"
        operation: Equals
        opNext: ExpressionNode {
          kind: Constant
          constant: StringType { value: "official" }
        }
      }
    ]
  }
}
```

### Example 4: Binary Operations with Precedence
**FHIRPath**: `a + b * c`
```java
// After precedence reorganization:
ExpressionNode {
  kind: Name
  name: "a"
  operation: Plus
  opNext: ExpressionNode {
    kind: Name
    name: "b"
    operation: Times
    opNext: ExpressionNode {
      kind: Name
      name: "c"
    }
  }
}
```

### Example 5: Complex Expression
**FHIRPath**: `(age > 18) and active.exists()`
```java
ExpressionNode {
  kind: Group
  group: ExpressionNode {
    kind: Name
    name: "age"
    operation: Greater
    opNext: ExpressionNode {
      kind: Constant
      constant: IntegerType { value: 18 }
    }
  }
  operation: And
  opNext: ExpressionNode {
    kind: Name
    name: "active"
    inner: ExpressionNode {
      kind: Function
      function: Exists
      parameters: []
    }
  }
}
```

### Example 6: Union Operation
**FHIRPath**: `Patient.name | Patient.contact.name`
```java
ExpressionNode {
  kind: Name
  name: "Patient"
  inner: ExpressionNode {
    kind: Name
    name: "name"
  }
  operation: Union
  opNext: ExpressionNode {
    kind: Name
    name: "Patient"
    inner: ExpressionNode {
      kind: Name
      name: "contact"
      inner: ExpressionNode {
        kind: Name
        name: "name"
      }
    }
  }
}
```

### Example 7: Indexer and Type Check
**FHIRPath**: `name[0] is HumanName`
```java
ExpressionNode {
  kind: Name
  name: "name"
  inner: ExpressionNode {
    kind: Function
    function: Item  // Indexing is represented as Item function
    parameters: [
      ExpressionNode {
        kind: Constant
        constant: IntegerType { value: 0 }
      }
    ]
  }
  operation: Is
  opNext: ExpressionNode {
    kind: Name
    name: "HumanName"
  }
}
```

### Example 8: Conditional Expression
**FHIRPath**: `iif(gender = 'male', 'Mr.', 'Ms.')`
```java
ExpressionNode {
  kind: Function
  function: Iif
  parameters: [
    ExpressionNode {  // Condition
      kind: Name
      name: "gender"
      operation: Equals
      opNext: ExpressionNode {
        kind: Constant
        constant: StringType { value: "male" }
      }
    },
    ExpressionNode {  // Then branch
      kind: Constant
      constant: StringType { value: "Mr." }
    },
    ExpressionNode {  // Else branch
      kind: Constant
      constant: StringType { value: "Ms." }
    }
  ]
}
```

## Type System (TypeDetails)

### Collection Cardinality

```java
public enum CollectionStatus {
    SINGLETON,   // Exactly one value
    ORDERED,     // Ordered collection (list)
    UNORDERED    // Unordered collection (set)
}
```

### Type Representation

- Uses URIs to identify types
- System types: `http://hl7.org/fhirpath/System.{Type}`
- FHIR types: `http://hl7.org/fhir/StructureDefinition/{Type}`
- Supports profiled types with constraints
- Tracks reference targets for Reference types

### Type Checking

Separate pass before evaluation:
- Validates function parameters
- Propagates types through expressions
- Checks operator compatibility
- Handles polymorphic types

## Evaluation Engine

### Execution Context

```java
public class ExecutionContext {
    private Object appInfo;
    private Base focusResource;
    private Base rootResource;
    private Base context;
    private Base thisItem;
    private List<Base> total;
    private Map<String, List<Base>> definedVariables;
}
```

### Evaluation Strategy

1. **List-based**: All values are lists of `Base` objects
2. **Recursive**: Direct AST interpretation
3. **Short-circuiting**: For logical operators
4. **Lazy evaluation**: For certain constructs

### Key Evaluation Methods

```java
private List<Base> execute(ExecutionContext context, 
                          List<Base> focus, 
                          ExpressionNode exp, 
                          boolean atEntry) {
    switch (exp.getKind()) {
        case Name: return executeContextType(context, exp.getName(), exp);
        case Function: return executeFunction(context, focus, exp);
        case Constant: return resolveConstant(context, exp.getConstant(), exp);
        case Group: return execute(context, focus, exp.getGroup(), atEntry);
    }
}
```

## Detailed Evaluation Process

### 1. Core Evaluation Flow

The evaluation follows a recursive pattern through the AST:

```java
public List<Base> evaluate(Base base, ExpressionNode expression) {
    // Create execution context
    ExecutionContext context = new ExecutionContext(appInfo, base, base);
    
    // Start evaluation with base as focus
    List<Base> focus = new ArrayList<>();
    focus.add(base);
    
    // Execute recursively
    return execute(context, focus, expression, true);
}
```

### 2. Navigation Through the AST

Each node type has specific evaluation logic:

#### Name Node Evaluation
```java
private List<Base> executeContextType(ExecutionContext context, String name, ExpressionNode exp) {
    // Handle special variables
    if (name.equals("$this")) return context.getThisItem();
    if (name.equals("$index")) return makeBoolean(context.getIndex());
    if (name.equals("$total")) return context.getTotal();
    
    // Handle defined variables
    if (context.hasDefinedVariable(name))
        return context.getDefinedVariable(name);
    
    // Navigate to property
    return getChildrenByName(focus, name);
}
```

#### Function Evaluation Dispatch
```java
private List<Base> executeFunction(ExecutionContext context, List<Base> focus, ExpressionNode exp) {
    switch (exp.getFunction()) {
        case Where: return funcWhere(context, focus, exp);
        case Select: return funcSelect(context, focus, exp);
        case First: return funcFirst(context, focus, exp);
        case Exists: return funcExists(context, focus, exp);
        // ... many more functions
    }
}
```

### 3. Collection-Based Operations

All operations work on collections (List<Base>):

#### Where Function Implementation
```java
private List<Base> funcWhere(ExecutionContext context, List<Base> focus, ExpressionNode exp) {
    List<Base> result = new ArrayList<>();
    
    for (Base item : focus) {
        // Set $this to current item
        context.setThisItem(item);
        
        // Evaluate predicate
        List<Base> verdict = execute(context, makeList(item), exp.getParameters().get(0), false);
        
        // Add if predicate is true
        if (convertToBoolean(verdict)) {
            result.add(item);
        }
    }
    
    return result;
}
```

#### Select Function Implementation
```java
private List<Base> funcSelect(ExecutionContext context, List<Base> focus, ExpressionNode exp) {
    List<Base> result = new ArrayList<>();
    
    for (Base item : focus) {
        context.setThisItem(item);
        
        // Evaluate projection expression
        List<Base> selected = execute(context, makeList(item), exp.getParameters().get(0), false);
        
        // Flatten results
        result.addAll(selected);
    }
    
    return result;
}
```

### 4. Binary Operation Evaluation

Binary operations follow after evaluating the left side:

```java
private List<Base> execute(ExecutionContext context, List<Base> focus, ExpressionNode exp, boolean atEntry) {
    List<Base> work = evaluateLeftSide(context, focus, exp, atEntry);
    
    if (exp.getOperation() != null) {
        // Check for short-circuit
        ExpressionNode next = exp.getOpNext();
        ExpressionNode last = exp;
        
        while (next != null) {
            // Short-circuit evaluation
            if (preOperate(work, last.getOperation())) {
                return work;
            }
            
            // Evaluate right side
            List<Base> work2 = evaluateRightSide(context, focus, next);
            
            // Apply operation
            work = operate(work, last.getOperation(), work2);
            
            last = next;
            next = next.getOpNext();
        }
    }
    
    return work;
}
```

### 5. Short-Circuit Evaluation

```java
private boolean preOperate(List<Base> left, Operation operation) {
    switch (operation) {
        case And:
            return !convertToBoolean(left);  // False and X = False
        case Or:
            return convertToBoolean(left);    // True or X = True
        case Implies:
            return !convertToBoolean(left);   // False implies X = True
        default:
            return false;
    }
}
```

### 6. Property Navigation

```java
private List<Base> getChildrenByName(List<Base> focus, String name) {
    List<Base> result = new ArrayList<>();
    
    for (Base item : focus) {
        // Handle polymorphic properties (e.g., value[x])
        if (name.endsWith("[x]")) {
            String prefix = name.substring(0, name.length() - 3);
            // Find all properties starting with prefix
            for (Property prop : item.children()) {
                if (prop.getName().startsWith(prefix)) {
                    result.addAll(prop.getValues());
                }
            }
        } else {
            // Normal property access
            Base[] children = item.listChildrenByName(name);
            if (children != null) {
                result.addAll(Arrays.asList(children));
            }
        }
    }
    
    return result;
}
```

### 7. Type Operations

```java
private List<Base> funcIs(ExecutionContext context, List<Base> focus, ExpressionNode exp) {
    if (focus.size() != 1) {
        return makeBoolean(false);
    }
    
    String type = exp.getParameters().get(0).getName();
    return makeBoolean(focus.get(0).isType(type));
}

private List<Base> funcAs(ExecutionContext context, List<Base> focus, ExpressionNode exp) {
    String type = exp.getParameters().get(0).getName();
    List<Base> result = new ArrayList<>();
    
    for (Base item : focus) {
        if (item.isType(type)) {
            result.add(item);
        }
    }
    
    return result;
}
```

## Evaluation Examples

### Example 1: Simple Navigation
**Expression**: `Patient.name.given`

```java
// Step 1: Evaluate "Patient"
focus = [PatientResource]
executeContextType("Patient") -> filters to Patient type

// Step 2: Navigate to "name" (via inner)
focus = [PatientResource]
getChildrenByName(focus, "name") -> [HumanName1, HumanName2]

// Step 3: Navigate to "given" (via inner)
focus = [HumanName1, HumanName2]
getChildrenByName(focus, "given") -> ["John", "J.", "Jane"]
```

### Example 2: Where Clause
**Expression**: `Patient.name.where(use = 'official')`

```java
// Step 1: Get all names
focus = [HumanName1{use:'official'}, HumanName2{use:'nickname'}]

// Step 2: Execute where function
for each name in focus:
    context.$this = name
    evaluate "use = 'official'":
        - get use -> ['official'] or ['nickname']
        - compare with 'official'
        - return [true] or [false]
    if true, add to result

// Result: [HumanName1{use:'official'}]
```

### Example 3: Chained Operations
**Expression**: `Patient.telecom.where(system = 'phone').value`

```java
// Step 1: Get telecoms
focus = [ContactPoint1{system:'phone', value:'555-1234'}, 
         ContactPoint2{system:'email', value:'john@example.com'}]

// Step 2: Filter by system
funcWhere:
    ContactPoint1: system='phone' -> true -> keep
    ContactPoint2: system='email' -> false -> skip
intermediate = [ContactPoint1]

// Step 3: Get values
getChildrenByName(intermediate, "value") -> ["555-1234"]
```

### Example 4: Logical Short-Circuit
**Expression**: `Patient.deceased.exists() or Patient.active`

```java
// Step 1: Evaluate left side
Patient.deceased.exists() -> [false]

// Step 2: Check short-circuit
preOperate([false], Or) -> false (no short-circuit)

// Step 3: Evaluate right side
Patient.active -> [true]

// Step 4: Apply OR operation
operate([false], Or, [true]) -> [true]
```

### Example 5: Complex Expression
**Expression**: `Observation.where(code.coding.where(system = 'http://loinc.org' and code = '8867-4').exists())`

```java
// For each Observation:
//   For each coding in code.coding:
//     Check if system = 'http://loinc.org' AND code = '8867-4'
//   If any coding matches, include the Observation

// Step-by-step for one Observation:
context.$this = Observation1
code.coding -> [Coding1, Coding2]

// Inner where on codings:
for Coding1:
    system = 'http://loinc.org' -> true
    code = '8867-4' -> true
    AND -> true
for Coding2:
    system = 'http://snomed.info/sct' -> false
    AND -> false (short-circuit)

// exists() on results: [true, false].exists() -> true
// Observation1 is included in final result
```

## Key Implementation Insights

1. **Everything is a List**: Simplifies operations but creates many temporary collections
2. **Context Threading**: ExecutionContext carries state through recursive calls
3. **Special Variables**: $this, $index, $total are managed by context
4. **Short-Circuit Logic**: Prevents unnecessary evaluation
5. **Type Safety**: Runtime type checking with graceful empty returns
6. **FHIR Integration**: Deep knowledge of FHIR types and polymorphic properties

## Notable Implementation Features

### 1. Polymorphic Name Resolution
Handles FHIR's polymorphic properties (e.g., `value[x]` matches `valueString`, `valueInteger`, etc.)

### 2. Quantity Arithmetic
Full UCUM support for unit conversions and calculations

### 3. Date/Time Handling
- Partial dates supported
- Timezone handling
- Arithmetic operations
- Duration calculations

### 4. Reference Resolution
Can resolve FHIR references during evaluation using host services

### 5. Extension Support
Special handling for FHIR extensions with convenient access methods

### 6. External Functions
Extensibility through `IEvaluationContext` interface:

```java
public interface IEvaluationContext {
    List<Base> resolveConstant(String name);
    List<Base> executeFunction(String name, List<List<Base>> parameters);
    Base resolveReference(String url);
    boolean conformsToProfile(Base item, String url);
    ValueSet resolveValueSet(String url);
}
```

## Performance Characteristics

- **No compilation**: Direct AST interpretation
- **AST reuse**: Parse once, evaluate many times
- **Type caching**: Type checking results stored in AST
- **Memory efficient**: Streaming evaluation where possible
- **No optimization**: No constant folding or common subexpression elimination

## Design Patterns

1. **Visitor-like pattern**: For node evaluation
2. **Strategy pattern**: For function execution
3. **Builder pattern**: For AST construction
4. **Immutable AST**: Nodes not modified after parsing
5. **Separation of concerns**: Lexing, parsing, type checking, and evaluation are distinct phases

## Error Handling

- Custom exception hierarchy
- Location tracking for all errors
- Detailed error messages
- Type errors caught before evaluation

## Key Differences from Spec

1. **Single AST node type**: Uses discriminated union rather than inheritance hierarchy
2. **Integrated parser**: Parser is part of the engine, not separate
3. **FHIR-specific features**: Deep integration with FHIR types and resources
4. **Extended function set**: Many functions beyond the base spec

## Strengths

1. **Complete implementation**: All FHIRPath features supported
2. **Strong type safety**: Comprehensive type checking
3. **Good error reporting**: With location information
4. **Extensible**: Through host services interface
5. **Well-tested**: Extensive test suite
6. **FHIR integration**: Seamless working with FHIR resources

## Potential Improvements

1. **Large classes**: FHIRPathEngine is ~6000 lines
2. **No optimization**: Could benefit from AST optimization passes
3. **No compilation**: JIT compilation could improve performance
4. **Memory usage**: Creates many intermediate lists
5. **Complex precedence handling**: Could be simplified with a precedence climbing parser

## Conclusion

The Java implementation provides a robust, feature-complete FHIRPath evaluator with excellent FHIR integration. While the architecture is traditional and the main classes are large, the code is well-structured and maintainable. The separation of lexing, parsing, type checking, and evaluation provides clear boundaries between concerns.