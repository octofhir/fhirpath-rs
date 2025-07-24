## Operations

Operators are allowed to be used between any kind of path expressions (e.g. expr op expr). Like functions, operators will generally propagate an empty collection in any of their operands. This is true even when comparing two empty collections using the equality operators, e.g.

```
{} = {}
true > {}
{} != 'dummy'
```

all result in `{}`.

### Equality

#### = (Equals)

Returns `true` if the left collection is equal to the right collection:

As noted above, if either operand is an empty collection, the result is an empty collection. Otherwise:

If both operands are collections with a single item, they must be of the same type, and:

* For primitives:
  * `String`: comparison is based on Unicode values
  * `Integer`: values must be exactly equal
  * `Decimal`: values must be equal, trailing zeroes after the decimal are ignored
  * `Boolean`: values must be the same
  * `Date`: must be exactly the same
  * `DateTime`: must be exactly the same, respecting the timezone offset (though +00:00 = -00:00 = Z)
  * `Time`: must be exactly the same
* For complex types, equality requires all child properties to be equal, recursively.

If both operands are collections with multiple items:

* Each item must be equal
* Comparison is order dependent

Otherwise, equals returns `false`.

Note that this implies that if the collections have a different number of items to compare, the result will be `false`.

Typically, this operator is used with single fixed values as operands. This means that `Patient.telecom.system = &#39;phone&#39;` will result in an error if there is more than one `telecom` with a `use`. Typically, you'd want `Patient.telecom.where(system = 'phone')`

If one or both of the operands is the empty collection, this operation returns an empty collection.

When comparing quantities for equality, the dimensions of each quantity must be the same, but not necessarily the unit. For example, units of 'cm' and 'm' can be compared, but units of 'cm2' and  'cm' cannot. The unit of the result will be the most granular unit of either input. Attempting to operate on quantities with invalid units will result in empty (`{ }`).

Implementations are not required to fully support operations on units, but they must at least respect units, recognizing when units differ.

Implementations that do support units SHALL do so as specified by [UCUM](#UCUM).

> Note: Although [UCUM](#UCUM) identifies 'a' as 365.25 days, and 'mo' as 1/12 of a year, calculations involving durations shall round using calendar semantics as specified in [ISO8601](#ISO8601). For comparisons involving durations (where no anchor to a calendar is available), the duration of a year is 365 days, and the duration of a month is 30 days.
>

For `Date`, `DateTime` and `Time` equality, the comparison is performed by considering each precision in order, beginning with years (or hours for time values), and respecting timezone offsets. If the values are the same, comparison proceeds to the next precision; if the values are different, the comparison stops and the result is `false`. If one input has a value for the precision and the other does not, the comparison stops and the result is empty (`{ }`); if neither input has a value for the precision, or the last precision has been reached, the comparison stops and the result is `true`. For the purposes of comparison, seconds and milliseconds are considered a single precision using a decimal, with decimal equality semantics.

For example:

```
@2012 = @2012 // returns true
@2012 = @2013 // returns false
@2012-01 = @2012 // returns empty ({ })
@2012-01-01T10:30 = @2012-01-01T10:30 // returns true
@2012-01-01T10:30 = @2012-01-01T10:31 // returns false
@2012-01-01T10:30:31 = @2012-01-01T10:30 // returns empty ({ })
@2012-01-01T10:30:31.0 = @2012-01-01T10:30:31 // returns true
@2012-01-01T10:30:31.1 = @2012-01-01T10:30:31 // returns false
```

For `DateTime` values that do not have a timezone offsets, whether or not to provide a default timezone offset is a policy decision. In the simplest case, no default timezone offset is provided, but some implementations may use the client's or the evaluating system's timezone offset.

To support comparison of DateTime values, either both values have no timezone offset specified, or both values are converted to a common timezone offset. The timezone offset to use is an implementation decision. In the simplest case, it's the timezone offset of the local server. The following examples illustrate expected behavior:

```
@2017-11-05T01:30:00.0-04:00 > @2017-11-05T01:15:00.0-05:00 // false
@2017-11-05T01:30:00.0-04:00 < @2017-11-05T01:15:00.0-05:00 // true
@2017-11-05T01:30:00.0-04:00 = @2017-11-05T01:15:00.0-05:00 // false
@2017-11-05T01:30:00.0-04:00 = @2017-11-05T00:30:00.0-05:00 // true
```

Additional functions to support more sophisticated timezone offset comparison (such as .toUTC()) may be defined in a future version.

#### ~ (Equivalent)

Returns `true` if the collections are the same. In particular, comparing empty collections for equivalence `{ } ~ { }` will result in `true`.

If both operands are collections with a single item, they must be of the same type, and:

* For primitives
  * `String`: the strings must be the same, ignoring case and locale, and normalizing whitespace (see [string-equivalence](#string-equivalence) for more details).
  * `Integer`: exactly equal
  * `Decimal`: values must be equal, comparison is done on values rounded to the precision of the least precise operand. Trailing zeroes after the decimal are ignored in determining precision.
  * `Date`, `DateTime` and `Time`: values must be equal, except that if the input values have different levels of precision, the comparison returns `false`, not empty (`{ }`).
  * `Boolean`: the values must be the same
* For complex types, equivalence requires all child properties to be equivalent, recursively.

If both operands are collections with multiple items:

* Each item must be equivalent
* Comparison is not order dependent

Note that this implies that if the collections have a different number of items to compare, or if one input is a value and the other is empty (`{ }`), the result will be `false`.

When comparing quantities for equivalence, the dimensions of each quantity must be the same, but not necessarily the unit. For example, units of 'cm' and 'm' can be compared, but units of 'cm2' and  'cm' cannot. The unit of the result will be the most granular unit of either input. Attempting to operate on quantities with invalid units will result in `false`.

Implementations are not required to fully support operations on units, but they must at least respect units, recognizing when units differ.

Implementations that do support units SHALL do so as specified by [UCUM](#UCUM).

> Note: Although [UCUM](#UCUM) identifies 'a' as 365.25 days, and 'mo' as 1/12 of a year, calculations involving durations shall round using calendar semantics as specified in [ISO8601](#ISO8601). For comparisons involving durations (where no anchor to a calendar is available), the duration of a year is 365 days, and the duration of a month is 30 days.
>

For `Date`, `DateTime` and `Time` equivalence, the comparison is the same as for equality, with the exception that if the input values have different levels of precision, the result is `false`, rather than empty (`{ }`). As with equality, the second and millisecond precisions are considered a single precision using a decimal, with decimal equivalence semantics.

For example:

```
@2012 ~ @2012 // returns true
@2012 ~ @2013 // returns false
@2012-01 ~ @2012 // returns false as well
@2012-01-01T10:30 ~ @2012-01-01T10:30 // returns true
@2012-01-01T10:30 ~ @2012-01-01T10:31 // returns false
@2012-01-01T10:30:31 ~ @2012-01-01T10:30 // returns false as well
@2012-01-01T10:30:31.0 ~ @2012-01-01T10:30:31 // returns true
@2012-01-01T10:30:31.1 ~ @2012-01-01T10:30:31 // returns false
```

##### String Equivalence

For strings, equivalence returns true if the strings are the same value while ignoring case and locale, and normalizing whitespace. Normalizing whitespace means that all whitespace characters are treated as equivalent, with whitespace characters as defined in the [Whitespace](#whitespace) lexical category.

#### != (Not Equals)

The converse of the equals operator.

#### !~ (Not Equivalent)

The converse of the equivalent operator.

### Comparison

* The comparison operators are defined for strings, integers, decimals, quantities, dates, datetimes and times.
* If one or both of the arguments is an empty collection, a comparison operator will return an empty collection.
* Both arguments must be collections with single values, and the evaluator will throw an error if either collection has more than one item.
* Both arguments must be of the same type, and the evaluator will throw an error if the types differ.
* When comparing integers and decimals, the integer will be converted to a decimal to make comparison possible.
* String ordering is strictly lexical and is based on the Unicode value of the individual characters.

When comparing quantities, the dimensions of each quantity must be the same, but not necessarily the unit. For example, units of 'cm' and 'm' can be compared, but units of 'cm2' and  'cm' cannot. The unit of the result will be the most granular unit of either input. Attempting to operate on quantities with invalid units will result in empty (`{ }`).

Implementations are not required to fully support operations on units, but they must at least respect units, recognizing when units differ.

Implementations that do support units SHALL do so as specified by [UCUM](#UCUM).

For partial Date, DateTime, and Time values, the comparison is performed by comparing the values at each precision, beginning with years, and proceeding to the finest precision specified in either input, and respecting timezone offsets. If one value is specified to a different level of precision than the other, the result is empty (`{ }`) to indicate that the result of the comparison is unknown. As with equality and equivalence, the second and millisecond precisions are considered a single precision using a decimal, with decimal comparison semantics.

See the [Equals](#equals) operator for discussion on respecting timezone offsets in comparison operations.

#### &gt; (Greater Than)

The greater than operator (`>`) returns true if the first operand is strictly greater than the second. The operands must be of the same type, or convertible to the same type using an implicit conversion.

```
10 > 5 // true
10 > 5.0 // true; note the 10 is converted to a decimal to perform the comparison
'abc' > 'ABC' // true
4 'm' > 4 'cm' // true (or { } if the implementation does not support unit conversion)
@2018-03-01 > @2018-01-01 // true
@2018-03 > @2018-03-01 // empty ({ })
@2018-03-01T10:30:00 > @2018-03-01T10:00:00 // true
@2018-03-01T10 > @2018-03-01T10:30 // empty ({ })
@2018-03-01T10:30:00 > @2018-03-01T10:30:00.0 // false
@T10:30:00 > @T10:00:00 // true
@T10 > @T10:30 // empty ({ })
@T10:30:00 > @T10:30:00.0 // false
```

#### &lt; (Less Than)

The less than operator (`<`) returns true if the first operand is strictly less than the second. The operands must be of the same type, or convertible to the same type using implicit conversion.

```
10 < 5 // false
10 < 5.0 // false; note the 10 is converted to a decimal to perform the comparison
'abc' < 'ABC' // false
4 'm' < 4 'cm' // false (or { } if the implementation does not support unit conversion)
@2018-03-01 < @2018-01-01 // false
@2018-03 < @2018-03-01 // empty ({ })
@2018-03-01T10:30:00 < @2018-03-01T10:00:00 // false
@2018-03-01T10 < @2018-03-01T10:30 // empty ({ })
@2018-03-01T10:30:00 < @2018-03-01T10:30:00.0 // false
@T10:30:00 < @T10:00:00 // false
@T10 < @T10:30 // empty ({ })
@T10:30:00 < @T10:30:00.0 // false
```

#### &lt;= (Less or Equal)

The less or equal operator (`\<=`) returns true if the first operand is less than or equal to the second. The operands must be of the same type, or convertible to the same type using implicit conversion.

```
10 <= 5 // true
10 <= 5.0 // true; note the 10 is converted to a decimal to perform the comparison
'abc' <= 'ABC' // true
4 'm' <= 4 'cm' // false (or { } if the implementation does not support unit conversion)
@2018-03-01 <= @2018-01-01 // false
@2018-03 <= @2018-03-01 // empty ({ })
@2018-03-01T10:30:00 <= @2018-03-01T10:00:00 // false
@2018-03-01T10 <= @2018-03-01T10:30 // empty ({ })
@2018-03-01T10:30:00 <= @2018-03-01T10:30:00.0 // true
@T10:30:00 <= @T10:00:00 // false
@T10 <= @T10:30 // empty ({ })
@T10:30:00 <= @T10:30:00.0 // true
```

#### &gt;= (Greater or Equal)

The greater or equal operator (`>=`) returns true if the first operand is greater than or equal to the second. The operands must be of the same type, or convertible to the same type using implicit conversion.

```
10 >= 5 // false
10 >= 5.0 // false; note the 10 is converted to a decimal to perform the comparison
'abc' >= 'ABC' // false
4 'm' >= 4 'cm' // true (or { } if the implementation does not support unit conversion)
@2018-03-01 >= @2018-01-01 // true
@2018-03 >= @2018-03-01 // empty ({ })
@2018-03-01T10:30:00 >= @2018-03-01T10:00:00 // true
@2018-03-01T10 >= @2018-03-01T10:30 // empty ({ })
@2018-03-01T10:30:00 >= @2018-03-01T10:30:00.0 // true
@T10:30:00 >= @T10:00:00 // true
@T10 >= @T10:30 // empty ({ })
@T10:30:00 >= @T10:30:00.0 // true
```

### Types

#### is _type specifier_

If the left operand is a collection with a single item and the second operand is a type identifier, this operator returns `true` if the type of the left operand is the type specified in the second operand, or a subclass thereof. If the identifier cannot be resolved to a valid type identifier, the evaluator will throw an error. If the input collections contains more than one item, the evaluator will throw an error. In all other cases this operator returns the empty collection.

A _type specifier_ is an identifier that must resolve to the name of a type in a model. Type specifiers can have qualifiers, e.g. `FHIR.Patient`, where the qualifier is the name of the model.

```
Patient.contained.all($this is Patient implies age > 10)
```

This example returns true if for all the contained resources, if the contained resource is of type `Patient`, then the `age` is greater than ten.

#### is(type : TypeInfo)

The `is()` function is supported for backwards compatibility with previous implementations of FHIRPath. Just as with the `is` keyword, the `type` argument is an identifier that must resolve to the name of a type in a model. For implementations with compile-time typing, this requires special-case handling when processing the argument to treat is a type specifier rather than an identifier expression:

```
Patient.contained.all($this.is(Patient) implies age > 10)
```

> Note: The `is()` function is defined for backwards compatibility only and may be deprecated in a future release.
>

#### as _type specifier_

If the left operand is a collection with a single item and the second operand is an identifier, this operator returns the value of the left operand if it is of the type specified in the second operand, or a subclass thereof. If the identifier cannot be resolved to a valid type identifier, the evaluator will throw an error. If there is more than one item in the input collection, the evaluator will throw an error. Otherwise, this operator returns the empty collection.

A _type specifier_ is an identifier that must resolve to the name of a type in a model. Type specifiers can have qualifiers, e.g. `FHIR.Patient`, where the qualifier is the name of the model.

```
Observation.component.where(value as Quantity > 30 'mg')
```

#### as(type : TypeInfo)

The `as()` function is supported for backwards compatibility with previous implementations of FHIRPath. Just as with the `as` keyword, the `type` argument is an identifier that must resolve to the name of a type in a model. For implementations with compile-time typing, this requires special-case handling when processing the argument to treat is a type specifier rather than an identifier expression:

```
Observation.component.where(value.as(Quantity) > 30 'mg')
```

> Note: The `as()` function is defined for backwards compatibility only and may be deprecated in a future release.
>

### Collections

#### | (union collections)
Merge the two collections into a single collection, eliminating any duplicate values (using [equals](#equals) (`=`)) to determine equality). Unioning an empty collection to a non-empty collection will return the non-empty collection with duplicates eliminated. There is no expectation of order in the resulting collection.

#### in (membership)
If the left operand is a collection with a single item, this operator returns true if the item is in the right operand using equality semantics. If the left-hand side of the operator is empty, the result is empty, if the right-hand side is empty, the result is false. If the left operand has multiple items, an exception is thrown.

The following example returns true if 'Joe' is in the list of given names for the Patient:

```
'Joe' in Patient.name.given
```

#### contains (containership)
If the right operand is a collection with a single item, this operator returns true if the item is in the left operand using equality semantics. This is the converse operation of in.

The following example returns true if the list of given names for the Patient has 'Joe' in it:

```
Patient.name.given contains 'Joe'
```

### Boolean logic
For all boolean operators, the collections passed as operands are first evaluated as Booleans (as described in [Singleton Evaluation of Collections](#singleton-evaluation-of-collections)). The operators then use three-valued logic to propagate empty operands.

> Note: To ensure that FHIRPath expressions can be freely rewritten by underlying implementations, there is no expectation that an implementation respect short-circuit evaluation. With regard to performance, implementations may use short-circuit evaluation to reduce computation, but authors should not rely on such behavior, and implementations must not change semantics with short-circuit evaluation. If short-circuit evaluation is needed to avoid effects (e.g. runtime exceptions), use the `iff()` function.
>

#### and

Returns `true` if both operands evaluate to `true`, `false` if either operand evaluates to `false`, and the empty collection (`{ }`) otherwise.

| and | true | false | empty |
| --- | --- | --- | --- |
| **true** | `true` | `false` | empty (`{ }`) |
| **false** | `false` | `false` | `false` |
| **empty** | empty (`{ }`) | `false` | empty (`{ }`) |

#### or

Returns `false` if both operands evaluate to `false`, `true` if either operand evaluates to `true`, and empty (`{ }`) otherwise:

| or | true | false | empty |
| --- | --- | --- | --- |
| **true** | `true` | `true` | `true` |
| **false** | `true` | `false` | empty (`{ }`) |
| **empty** | `true` | empty (`{ }`) | empty (`{ }`) |

#### not() : Boolean

Returns `true` if the input collection evaluates to `false`, and `false` if it evaluates to `true`. Otherwise, the result is empty (`{ }`):

| not |  |
| --- | --- |
| **true** | `false` |
| **false** | `true` |
| **empty** | empty (`{ }`) |

#### xor

Returns `true` if exactly one of the operands evaluates to `true`, `false` if either both operands evaluate to `true` or both operands evaluate to `false`, and the empty collection (`{ }`) otherwise:

| xor | true | false | empty |
| --- | --- | --- | --- |
| **true** | `false` | `true` | empty (`{ }`) |
| **false** | `true` | `false` | empty (`{ }`) |
| **empty** | empty (`{ }`) | empty (`{ }`) | empty (`{ }`) |

#### implies

If the left operand evaluates to `true`, this operator returns the boolean evaluation of the right operand. If the left operand evaluates to `false`, this operator returns `true`. Otherwise, this operator returns `true` if the right operand evaluates to `true`, and the empty collection (`{ }`) otherwise.

| implies | true | false | empty |
| --- | --- | --- | --- |
| **true** | `true` | `false` | empty (`{ }`) |
| **false** | `true` | `true` | `true` |
| **empty** | `true` | empty (`{ }`) | empty (`{ }`) |

The implies operator is useful for testing conditionals. For example, if a given name is present, then a family name must be as well:

```
Patient.name.given.exists() implies Patient.name.family.exists()
```

Note that implies may use short-circuit evaluation in the case that the first operand evaluates to false.

### Math

The math operators require each operand to be a single element. Both operands must be of the same type, or of compatible types according to the rules for implicit conversion. Each operator below specifies which types are supported.

If there is more than one item, or an incompatible item, the evaluation of the expression will end and signal an error to the calling environment.

As with the other operators, the math operators will return an empty collection if one or both of the operands are empty.

When operating on quantities, the dimensions of each quantity must be the same, but not necessarily the unit. For example, units of 'cm' and 'm' can be compared, but units of 'cm2' and  'cm' cannot. The unit of the result will be the most granular unit of either input. Attempting to operate on quantities with invalid units will result in empty (`{ }`).

Implementations are not required to fully support operations on units, but they must at least respect units, recognizing when units differ.

Implementations that do support units SHALL do so as specified by [UCUM](#UCUM).

Operations that cause arithmetic overflow or underflow will result in empty (`{ }`).

#### * (multiplication)

Multiplies both arguments (supported for Integer, Decimal, and Quantity). For multiplication involving quantities, the resulting quantity will have the appropriate unit:

```
12 'cm' * 3 'cm' // 36 'cm2'
3 'cm' * 12 'cm2' // 36 'cm3'
```

#### / (division)

Divides the left operand by the right operand (supported for Integer, Decimal, and Quantity). The result of a division is always Decimal, even if the inputs are both Integer. For integer division, use the `div` operator.

If an attempt is made to divide by zero, the result is empty.

For division involving quantities, the resulting quantity will have the appropriate unit:

```
12 'cm2' / 3 'cm' // 4.0 'cm'
12 / 0 // empty ({ })
```

#### + (addition)

For Integer, Decimal, and quantity, adds the operands. For strings, concatenates the right operand to the left operand.

When adding quantities, the dimensions of each quantity must be the same, but not necessarily the unit.

```
3 'm' + 3 'cm' // 303 'cm'
```

#### - (subtraction)

Subtracts the right operand from the left operand (supported for Integer, Decimal, and Quantity).

When subtracting quantities, the dimensions of each quantity must be the same, but not necessarily the unit.

```
3 'm' - 3 'cm' // 297 'cm'
```

#### div

Performs truncated division of the left operand by the right operand (supported for Integer and Decimal). In other words, the division that ignores any remainder:

```
5 div 2 // 2
5.5 div 0.7 // 7
5 div 0 // empty ({ })
```

#### mod

Computes the remainder of the truncated division of its arguments (supported for Integer and Decimal).

```
5 mod 2 // 1
5.5 mod 0.7 // 0.6
5 mod 0 // empty ({ })
```

#### &amp; (String concatenation)

For strings, will concatenate the strings, where an empty operand is taken to be the empty string. This differs from `+` on two strings, which will result in an empty collection when one of the operands is empty. This operator is specifically included to simplify treating an empty collection as an empty string, a common use case in string manipulation.

```
'ABC' + 'DEF' // 'ABCDEF'
'ABC' + { } + 'DEF' // { }
'ABC' & 'DEF' // 'ABCDEF'
'ABC' & { } & 'DEF' // 'ABCDEF'
```

### Date/Time Arithmetic

Date and time arithmetic operators are used to add time-valued quantities to date/time values. The left operand must be a `Date`, `DateTime`, or `Time` value, and the right operand must be a `Quantity` with a time-valued unit:

* `year`, `years`, or `&#39;a&#39;`
* `month`, `months`, or `&#39;mo&#39;`
* `week`, `weeks` or `&#39;wk&#39;`
* `day`, `days`, or `&#39;d&#39;`
* `hour`, `hours`, or `&#39;h&#39;`
* `minute`, `minutes`, or `&#39;min&#39;`
* `second`, `seconds`, or `&#39;s&#39;`
* `millisecond`, `milliseconds`, or `&#39;ms&#39;`

If there is more than one item, or an item of an incompatible type, the evaluation of the expression will end and signal an error to the calling environment.

If either or both arguments are empty (`{ }`), the result is empty (`{ }`).

#### + (addition)

Returns the value of the given `Date`, `DateTime`, or `Time`, incremented by the time-valued quantity, respecting variable length periods for calendar years and months.

For `Date` values, the quantity unit must be one of: `years`, `months`, `weeks`, or `days`

For `DateTime` values, the quantity unit must be one of: `years`, `months`, `weeks`, `days`, `hours`, `minutes`, `seconds`, or `milliseconds` (or an equivalent unit), or an error is raised.

For `Time` values, the quantity unit must be one of: `hours`, `minutes`, `seconds`, or `milliseconds` (or an equivalent unit), or an error is raised.

For partial date/time values, the operation is performed by converting the time-valued quantity to the highest precision in the partial (removing any decimal value off) and then adding to the date/time value. For example:

```
@2014 + 24 months
```

This expression will evaluate to the value `@2016` even though the date/time value is not specified to the level of precision of the time-valued quantity.

Calculations involving weeks are equivalent to multiplying the number of weeks by 7 and performing the calculation for the resulting number of days.

> Note: Although [UCUM](#UCUM) identifies 'a' as 365.25 days, and 'mo' as 1/12 of a year, calculations involving durations shall round using calendar semantics as specified in [ISO8601](#ISO8601).
>

#### - (subtraction)

Returns the value of the given `Date`, `DateTime`, or `Time`, decremented by the time-valued quantity, respecting variable length periods for calendar years and months.

For `Date` values, the quantity unit must be one of: `years`, `months`, `weeks`, or `days`

For `DateTime` values, the quantity unit must be one of: `years`, `months`, `weeks`, `days`, `hours`, `minutes`, `seconds`, `milliseconds` (or an equivalent unit), or an error is raised.

For `Time` values, the quantity unit must be one of: `hours`, `minutes`, `seconds`, or `milliseconds` (or an equivalent unit), or an error is raised.

For partial date/time values, the operation is performed by converting the time-valued quantity to the highest precision in the partial (removing any decimal value off) and then subtracting from the date/time value. For example:

```
@2014 - 24 months
```

This expression will evaluate to the value `@2012` even though the date/time value is not specified to the level of precision of the time-valued quantity.

Calculations involving weeks are equivalent to multiplying the number of weeks by 7 and performing the calculation for the resulting number of days.

> Note: Although [UCUM](#UCUM) identifies 'a' as 365.25 days, and 'mo' as 1/12 of a year, calculations involving durations shall round using calendar semantics as specified in [ISO8601](#ISO8601).
>

### Operator precedence

Precedence of operations, in order from high to low:

```
#01 . (path/function invocation)
#02 [] (indexer)
#03 unary + and -
#04: *, /, div, mod
#05: +, -, &
#06: is, as
#07: |
#08: >, <, >=, <=
#09: =, ~, !=, !~
#10: in, contains
#11: and
#12: xor, or
#13: implies
```

As customary, expressions may be grouped by parenthesis (`()`).