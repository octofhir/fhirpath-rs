## Expressions

FHIRPath expressions can consist of _paths_, _literals_, _operators_, and _function invocations_, and these elements can be chained together, so that the output of one operation or function is the input to the next. This is the core of the _fluent_ syntactic style and allows complex paths and expressions to be built up from simpler components.

### Literals

In addition to paths, FHIRPath expressions may contain _literals_, _operators_, and _function invocations_. FHIRPath supports the following types of literals:

```
Boolean: true, false
String: 'test string', 'urn:oid:3.4.5.6.7.8'
Integer: 0, 45
Decimal: 0.0, 3.14159265
Date: @2015-02-04 (@ followed by ISO8601 compliant date)
DateTime: @2015-02-04T14:34:28+09:00 (@ followed by ISO8601 compliant date/time)
Time: @T14:34:28 (@ followed by ISO8601 compliant time beginning with T, no timezone offset)
Quantity: 10 'mg', 4 days
```

For each type of literal, FHIRPath defines a named system type to allow operations and functions to be defined. For example, the multiplication operator (`*`) is defined for the numeric types Integer and Decimal, as well as the Quantity type. See the discussion on [Models](#models) for a more detailed discussion of how these types are used within evaluation contexts.

#### Boolean

The `Boolean` type represents the logical Boolean values `true` and `false`. These values are used as the result of comparisons, and can be combined using logical operators such as `and` and `or`.

```
true
false
```

#### String

The `String` type represents string values up to 2^31^-1 characters in length. String literals are surrounded by single-quotes and may use `\`-escapes to escape quotes and represent Unicode characters:

| Escape | Character |
| --- | --- |
| `\'` | Single-quote |
| `\"` | Double-quote |
| `\`` | Backtick |
| `\r` | Carriage Return |
| `\n` | Line Feed |
| `\t` | Tab |
| `\f` | Form Feed |
| `\\` | Backslash |
| `\uXXXX` | Unicode character, where XXXX is the hexadecimal representation of the character |

Note that Unicode is supported in both string literals and delimited identifiers. 

```
'test string'
'urn:oid:3.4.5.6.7.8'
```

#### Integer

The `Integer` type represents whole numbers in the range -2^31^ to 2^31^-1.

```
0
45
-5
```

Note that the minus sign (`-`) in the representation of a negative integer is not part of the literal, it is the unary negation operator defined as part of FHIRPath syntax.

#### Decimal

The `Decimal` type represents real values in the range (-10^28^+1)/10^8^ to (10^28^-1)/10^8^ with a step size of 10^-8^. This range is defined based on a survey of decimal-value implementations and is based on the most useful lowest common denominator. Implementations can provide support for larger decimals and higher precision, but must provide at least the range and precision defined here. In addition, implementations should use fixed-precision decimal formats to ensure that decimal values are accurately represented.

```
0.0
3.14159265
```

Note that decimal literals cannot use exponential notation. There is enough additional complexity associated with enabling exponential notation that this is outside the scope of what FHIRPath is intended to support (namely graph traversal).

#### Date

The `Date` type represents date and partial date values in the range @0001-01-01 to @9999-12-31 with a 1 day step size.

The `Date` literal is a subset of [ISO8601](#ISO8601):

* A date literal begins with an `@`
* It uses the `YYYY-MM-DD` format, though month and day parts are optional
* Week dates and ordinal dates are not allowed
* Years must be present (`-MM-DD` is not a valid Date in FHIRPath)
* Months must be present if a day is present
* To specify a date and time together, see the description of `DateTime` below

The following examples illustrate the use of the `Date` literal:

```
@2014-01-25
@2014-01
@2014
```

Consult the formal grammar for more details.

#### Time

The `Time` type represents time-of-day and partial time-of-day values in the range @T00:00:00.0 to @T23:59:59.999 with a step size of 1 millisecond. This range is defined based on a survey of time implementations and is based on the most useful lowest common denominator. Implementations can provide support for higher precision, but must provide at least the range and precision defined here. Time values in FHIRPath do not have a timezone or timezone offset.

The `Time` literal uses a subset of [ISO8601](#ISO8601):

* A time begins with a `@T`
* It uses the `Thh:mm:ss.fff` format

The following examples illustrate the use of the `Time` literal:

```
@T12:00
@T14:30:14.559
```

Consult the formal grammar for more details.

#### DateTime

The `DateTime` type represents date/time and partial date/time values in the range `@0001-01-01T00:00:00.0 to @9999-12-31T23:59:59.999` with a 1 millisecond step size. This range is defined based on a survey of datetime implementations and is based on the most useful lowest common denominator. Implementations can provide support for larger ranges and higher precision, but must provide at least the range and precision defined here.

The `DateTime` literal combines the `Date` and `Time` literals and is a subset of [ISO8601](#ISO8601):

* A datetime literal begins with an `@`
* It uses the `YYYY-MM-DDThh:mm:ss.fff±hh:mm` format
* Timezone offset is optional, but if present the notation `±hh:mm` is used (so must include both minutes and hours)
* `Z` is allowed as a synonym for the zero (+00:00) UTC offset.
* A `T` can be used at the end of any date (year, year-month, or year-month-day) to indicate a partial DateTime.

The following example illustrates the use of the `DateTime` literal:

```
@2014-01-25T14:30:14.559
@2014-01-25T14:30 // A partial DateTime with year, month, day, hour, and minute
@2014-03-25T // A partial DateTime with year, month, and day
@2014-01T // A partial DateTime with year and month
@2014T // A partial DateTime with only the year
```

The suffix `T` is allowed after a year, year-month, or year-month-day literal because without it, there would be no way to specify a partial DateTime with only a year, month, or day; the literal would always result in a Date value.

Consult the formal grammar for more details.

#### Quantity

The `Quantity` type represents quantities with a specified unit, where the `value` component is defined as a `Decimal`, and the `unit` element is represented as a `String` that is required to be a valid Unified Code for Units of Measure [UCUM](#UCUM) unit.

The `Quantity` literal is a number (integer or decimal), followed by a (single-quoted) string representing a valid Unified Code for Units of Measure [UCUM](#UCUM) unit. If the value literal is an Integer, it will be implicitly converted to a Decimal in the resulting Quantity value:

```
4.5 'mg'
100 '[degF]'
```

> Note: When using [UCUM](#UCUM) units within FHIRPath, implementations shall use case-sensitive comparisons.
>

For date/time units, an alternative representation may be used (note that both a plural and singular version exist):

* `year`/`years`, `month`/`months`, `week`/`weeks`, `day`/`days`, `hour`/`hours`, `minute`/`minutes`, `second`/`seconds`, `millisecond`/`milliseconds`

```
1 year
4 days
```

> Note: Although [UCUM](#UCUM) identifies 'a' as 365.25 days, and 'mo' as 1/12 of a year, calculations involving durations shall round using calendar semantics as specified in [ISO8601](#ISO8601).
>

### Operators

Expressions can also contain _operators_, like those for mathematical operations and boolean logic:

```
Appointment.minutesDuration / 60 > 5
MedicationAdministration.wasNotGiven implies MedicationAdministration.reasonNotGiven.exists()
name.given | name.family // union of given and family names
'sir ' + name.given
```

Operators available in FHIRPath are covered in detail in the [Operations](#operations) section.

### Function Invocations

Finally, FHIRPath supports the notion of functions, which all take a collection of values as input and produce another collection as output and may take parameters. For example:

```
(name.given | name.family).substring(0,4)
identifier.where(use = 'official')
```

Since all functions work on collections, constants will first be converted to a collection when functions are invoked on constants:

```
(4+5).count()
```

will return `1`, since this is implicitly a collection with one constant number `9`.

In general, functions in FHIRPath take collections as input and produce collections as output. This property, combined with the syntactic style of _dot invocation_ enables functions to be chained together, creating a _fluent_-style syntax:

```
Patient.telecom.where(use = 'official').union(Patient.contact.telecom.where(use = 'official')).exists().not()
```

Throughout the function documentation, this _input_ parameter is implicitly assumed, rather than explicitly documented in the function signature like the other arguments. For a complete listing of the functions defined in FHIRPath, refer to the [Functions](#functions) section.

### Null and empty

There is no concept of `null` in FHIRPath. This means that when, in an underlying data object a member is null or missing, there will simply be no corresponding node for that member in the tree, e.g. `Patient.name` will return an empty collection (not null) if there are no name elements in the instance.

In expressions, the empty collection is represented as `{}`.

#### Propagation of empty results in expressions

FHIRPath functions and operators both propagate empty results, but the behavior is in general different when the argument to the function or operator expects a collection (e.g. `select()`, `where()` and `|` (union)) versus when the argument to the function or operator takes a single value as input (e.g. `+` and `substring()`).

For functions or operators that take a single values as input, this means in general if the input is empty, then the result will be empty as well. More specifically:

* If a single-input operator or function operates on an empty collection, the result is an empty collection
* If a single-input operator or function is passed an empty collection as an argument, the result is an empty collection
* If any operand to a single-input operator or function is an empty collection, the result is an empty collection.

For operator or function arguments that expect collections, in general the empty collection is treated as any other collection would be. For example, the union (`|`) of an empty collection with some non-empty collection is that non-empty collection.

When functions or operators behave differently from these general principles, (for example the `count()` and `empty()` functions), this is clearly documented in the next sections.

### Singleton Evaluation of Collections

In general, when a collection is passed as an argument to a function or operator that expects a single item as input, the collection is implicitly converted to a singleton as follows:

```
IF the collection contains a single node AND the node's value can be converted to the expected input type THEN
  The collection evaluates to the value of that single node
ELSE IF the collection contains a single node AND the expected input type is Boolean THEN
  The collection evaluates to true
ELSE IF the collection is empty THEN
  The collection evaluates to an empty collection
ELSE
  An error is raised
```

For example:

```
Patient.name.family + ', ' + Patient.name.given
```

If the `Patient` instance has a single `name`, and that name has a single `given`, then this will evaluate without any issues. However, if the `Patient` has multiple `name` elements, or the single name has multiple `given` elements, then it's ambiguous which of the elements should be used as the input to the `+` operator, and the result is an error.

As another example:

```
Patient.active and Patient.gender and Patient.telecom
```

Assuming the `Patient` instance has an `active` value of `true`, a `gender` of `female` and a single `telecom` element, this expression will result in true. However, consider a different instance of `Patient` that has an `active` value of `true`, a `gender` of `male`, and multiple `telecom` elements, then this expression will result in an error because of the multiple telecom elements.