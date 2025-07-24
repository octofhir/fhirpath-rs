# FHIRPath Key Definitions

## Special Variables

### $this

**Definition**: A special variable that represents the current item being operated on in the FHIRPath expression.

**Behavior**:

1. **At expression start**: 
   - Set to the initial evaluation context (same as `%context`)
   - Represents the data instance the expression is being evaluated against

2. **In iterative functions** (`where`, `select`, `exists`, `all`, `repeat`, `aggregate`):
   - Reset to each individual item in the input collection during iteration
   - Scoped to the function's parameters/arguments only
   - Previous value is restored when exiting the function

3. **Example usage**:
   ```fhirpath
   // At root level - implicit reference to context
   name.given  // equivalent to $this.name.given
   
   // In where clause - refers to each item
   Patient.name.where($this.use = 'official')
   
   // In select - refers to each name element
   Patient.name.select($this.given + ' ' + $this.family)
   
   // Can be implicit inside iterative functions
   Patient.name.where(use = 'official')  // 'use' implicitly means '$this.use'
   ```

**Key characteristics**:
- Mutable: Changes during expression evaluation
- Context-sensitive: Value depends on current evaluation context
- Always defined: Never null or undefined
- Collection-valued: Like all FHIRPath values, $this is always a collection (may be single-valued)

**Relationship to other variables**:
- Initially equals `%context` at expression start
- Different from `%context` which never changes during evaluation
- Related to proposed `$focus` variable (not yet in spec)

### %context

**Definition**: The immutable reference to the original node/resource that was passed to the FHIRPath evaluation engine.

**Behavior**:
- Set once at the beginning of expression evaluation
- Never changes during evaluation
- Useful for referring back to the original context from within nested expressions
- Often used in validation invariants

**Example**:
```fhirpath
// In a complex expression, refer back to original context
Patient.contact.where(relationship.exists() and %context.deceased.not())
```

### $index

**Definition**: The zero-based index of the current item being processed in iterative functions.

**Behavior**:
- Set to 0 at expression start
- Updated to the current item's index during iteration in functions like `where`, `select`, etc.
- Only meaningful within iterative function parameters
- Restored to previous value when exiting the function

**Example**:
```fhirpath
// Select first 3 items
Patient.name.where($index < 3)
```

### $total

**Definition**: Special variable available only within the `aggregate` function that holds the running total/accumulator.

**Behavior**:
- Only exists within `aggregate` function parameters
- Represents the accumulated value during iteration
- Final value becomes the function result

**Example**:
```fhirpath
// Sum all values
value.aggregate($total + $this, 0)
```

## Focus vs $this

**Focus**: The conceptual "current input collection" that flows through the expression chain. Not directly accessible as a variable in current FHIRPath spec.

**Key distinction**:
- Focus = what the function operates on (the input collection)
- $this = the current item when iterating over that collection

In most cases they have the same value, but conceptually:
- Focus flows through the fluent chain: `Patient.name.given`
- $this is set during iteration: `name.where($this.use = 'official')`

## Iterative Functions

Functions that set `$this` and `$index` for each item in their input collection:

1. **where(criteria)** - Filters based on criteria
2. **select(projection)** - Transforms each item
3. **exists(criteria)** - Tests if any item matches
4. **all(criteria)** - Tests if all items match
5. **repeat(projection)** - Recursively applies projection
6. **aggregate(aggregator, init)** - Reduces collection to single value

These functions:
- Evaluate their expression parameters for each item in the input
- Set `$this` to the current item
- Set `$index` to the current position
- Restore previous values when complete

## Non-Iterative Functions with Expression Parameters

Functions like `iif`, `trace`, and `defineVariable` take expression parameters but do NOT set `$this` or `$index`. They operate on the input collection as a whole.