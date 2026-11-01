---
standards-status: normative
---

# FHIRPath (Continuous Build)

FHIRPath is a path based navigation and extraction language, somewhat like XPath. Operations are expressed in terms of the logical content of hierarchical data models, and support traversal, selection and filtering of data. Its design was influenced by the needs for path navigation, selection and formulation of invariants in both HL7 Fast Healthcare Interoperability Resources ([FHIR](http://hl7.org/fhir)) and HL7 Clinical Quality Language ([CQL](http://cql.hl7.org/03-developersguide.html#using-fhirpath)).

Looking for implementations? See [FHIRPath Implementations on the HL7 confluence](https://confluence.hl7.org/display/FHIRI/FHIRPath+Implementations){:target="_blank"}


> **Note:** The following sections of this specification have not received significant implementation experience and are marked for Standard for Trial Use (STU):
> 
> * [Aggregates](#aggregates)
> * [Literals - Long](#long)
> * [Conversions - toLong](#tolong--long)
> * [Functions - repeatAll](#fn-repeatall)
> * [Functions - sort](#fn-sort)
> * [Functions - coalesce](#coalesce)
> * [Functions - String (lastIndexOf)](#lastindexofsubstring--string--integer)
> * [Functions - String (matchesFull)](#matchesfullregex--string-flags--string--boolean)
> * [Functions - String (trim, split, join)](#trim--string)
> * [Functions - String (encode, decode, escape, unescape)](#additional-string-functions)
> * [Functions - Math](#math)
> * [Functions - Utility (defineVariable, lowBoundary, highBoundary)](#definevariable)
> * [Functions - Utility (precision)](#precision--integer)
> * [Functions - pathname](#fn-pathname)
> * [Functions - Extract Date/DateTime/Time components](#extract-datedatetimetime-components)
> * [Functions - Date and Time Interval Functions (duration, difference)](#date-and-time-interval-functions)
> * [Operations - Comparison (comparable)](#fn-comparable)
> * [Instance Selector/Object Creation](#instance-selector)
> * [Types - Reflection](#reflection)
> 
> These sections have undergone extensive clarification:
> * [Functions - documentation of functions with scoped arguments](#scoped-functions)
> * [Functions - combine](#fn-combine) - additional optional parameter `preserveOrder`
> * [Date Conversion Functions](#date-conversion-functions) - inclusion of string format codes and `format` parameter
> * [Functions - String (matches)](#fn-matches) - additional optional parameter `flags`
> * [Operations - Quantity Equality / Equivalence (time-valued)](#quantity-equality) - refined cross-system calendar/UCUM comparisons
> * [Operations - Math](#math-1) - refined handling of quantity operations - particularly with time based quanties
> * [Unary Operators](#unary-operators) - this is a new section, however they have been implied as defined since the normative version via other parts of the specification and grammar
>
> In addition, the appendices are included as additional documentation and are informative content.
>
> All content that appears with this pink stripe to the left is marked as Standard for Trial Use (STU) 
{: .stu-note }


## Background

In Information Systems in general, and Healthcare Information Systems in particular, the need for formal representation of logic is both pervasive and critical. From low-level technical specifications, through intermediate logical architectures, up to the high-level conceptual descriptions of requirements and behavior, the ability to formally represent knowledge in terms of expressions and information models is essential to the specification and implementation of these systems.

### Requirements

Of particular importance is the ability to easily and precisely express conditions of basic logic, such as those found in requirements constraints (e.g. Patients must have a name), decision support (e.g. if the patient has diabetes and has not had a recent comprehensive foot exam), cohort definitions (e.g. All male patients aged 60-75), protocol descriptions (e.g. if the specimen has tested positive for the presence of sodium), and numerous other environments.

Precisely because the need for such expressions is so pervasive, there is no shortage of existing languages for representing them. However, these languages tend to be tightly coupled to the data structures, and even the information models on which they operate, XPath being a typical example. To ensure that the knowledge captured by the representation of these expressions can survive technological drift, a representation that can be used independent of any underlying physical implementation is required.

Languages meeting these additional requirements also exist, such as Object Constraint Language (OCL), Java, JavaScript, C#, and others. However, these languages are both tightly coupled to the platforms in which they operate, and, because they are general-purpose development languages, come with much heavier tooling and technology dependencies than is warranted or desirable. Even constraining one of these grammars would be insufficient, resulting in the need to extend, defeating the purpose of basing it on an existing language in the first place.

Given these constraints, and the lack of a specific language that meets all of these requirements, there is a need for a simple, lightweight, platform- and structure-independent graph traversal language. FHIRPath meets these requirements, and can be used within various environments to provide for simple but effective formal representation of expressions.

### Features

* Graph-traversal: FHIRPath is a graph-traversal language; authors can clearly and concisely express graph traversal on hierarchical information models (e.g. Health Level 7 - Version 3 (HL7 V3), Fast Healthcare Interoperability Resources (FHIR), virtual Medical Record (vMR), Clinical Information Modeling Initiative (CIMI), and Quality Data Model (QDM)).
* Fluent: FHIRPath has a syntax based on the [Fluent Interface](https://en.wikipedia.org/wiki/Fluent_interface) pattern
* Collection-centric: FHIRPath deals with all values as collections, allowing it to easily deal with information models with repeating elements.
* Platform-independent: FHIRPath is a conceptual and logical specification that can be implemented in any platform.
* Model-independent: FHIRPath deals with data as an abstract model, allowing it to be used with any information model.

### Usage

In Fast Healthcare Interoperability Resources ([FHIR](http://hl7.org/fhir)), FHIRPath is used within the specification to provide formal definitions for conditions such as validation invariants, search parameter paths, etc. Within Clinical Quality Language ([CQL](http://cql.hl7.org)), FHIRPath is used to simplify graph-traversal for hierarchical information models.

In both FHIR and CQL, the model independence of FHIRPath means that expressions can be written that deal with the contents of the resources and data types as described in the Logical views, or the UML diagrams, rather than against the physical representation of those resources. JSON and XML specific features are not visible to the FHIRPath language (such as comments and the split representation of primitives (i.e. `value[x]`)).

The expressions can in theory be converted to equivalent expressions in XPath, OCL, or another similarly expressive language.

FHIRPath can be used against many other graphs as well. For example, [Use of FHIRPath on HL7 Version 2 messages](appendices.html#hl7v2) describes how FHIRPath is used in HL7 V2.

### Conventions

Throughout this documentation, `monospace font` is used to delineate expressions of FHIRPath.

Optional parameters to functions are enclosed in square brackets in the definition of a function. Note that the brackets are only used to indicate optionality in the signature, they are not part of the actual syntax of FHIRPath.

All operations and functions return a collection, but if the operation or function will always produce a collection containing a single item of a predefined type, the description of the operation or function will specify its output type explicitly, instead of just stating `collection`, e.g. `all(...) : Boolean`

Throughout this specification, formatting patterns for Date, Time, and DateTime values are described using an informal description with the following markers:

* `yyyy`{:.formatted} - A full four digit year (0001..9999), padded with leading zeroes if necessary
* `MM`{:.formatted} - A full two digit month value (01..12), padded with leading zeroes if necessary
* `DD`{:.formatted} - A full two digit day value (01..31), padded with leading zeroes if necessary
* `hh`{:.formatted} - A full two digit hour value (00..24), padded with leading zeroes if necessary
* `mm`{:.formatted} - A full two digit minute value (00..59), padded with leading zeroes if necessary
* `ss`{:.formatted} - A full two digit second value (00..59), padded with leading zeroes if necessary
* `fff`{:.formatted} - A fractional millisecond value (0..999)

and formatting patterns for numeric types are described using an informal description with these markers:
* `0`{:.formatted} - Any digit shall appear at this location in the format string
* `#`{:.formatted} - Any digits may appear at this location in the format string
* `?`{:.formatted} - The immediately preceding pattern is optional
* `( )`{:.formatted} - Used to group patterns
* `« »`{:.formatted} - Used indicate some specific named content is included. e.g `'«unit»'`{:.formatted} to indicate that a quantities unit would be surrounded by single quotes.
* `|`{:.formatted} - Used to combine choices of patterns (e.g. `+|-`{:.formatted} means a **`+`** or **`-`** will appear at this location)

Any other character in a format string indicates that the character must appear at that location (unless it has an optional indicator, or is part of an optional group), noting that named content inside `« »` isn't explicit text.

These formatting patterns are set in `bold`{:.formatted} to distinguish them typographically from literals or code and to make clear that they are not intended to be formally interpreted as regex patterns.

#### Conformance Language

<div data-fhir="generated" markdown="1">
This specification uses the conformance verbs SHALL, SHOULD, and MAY as defined in [RFC 2119](https://www.ietf.org/rfc/rfc2119.txt). Unlike RFC 2119, however, this specification allows that different applications might not be able to interoperate because of how they use optional features. In particular:

* **SHALL**: An absolute requirement for all implementations
* **SHALL NOT**: An absolute prohibition against inclusion for all implementations
* **SHOULD/SHOULD NOT**: A best practice or recommendation to be considered by implementers within the context of their particular implementation; there may be valid reasons to ignore an item, but the full implications must be understood and carefully weighed before choosing a different course
* **MAY**: This is truly optional language for an implementation; can be included or omitted as the implementer decides with no implications.
</div>

### Credits
This implementation guide represents significant person-hours of discussion, development work, and testing. The following are some of the key participants in the work, but many more have assisted through connectathons testing, asking questions and participating in conference calls.

| Role | Contributors |
| ---- | ------------ |
| *Editors:* | **Bryn Rhodes** (Smile Digital Health)<br/>**Grahame Grieve** (Health Intersections)<br/>**Ewout Kramer** (Firely)<br/>**Brian Postlethwaite** (Microsoft Research) |
| *Key Contributors:* | **Paul Lynch** (U.S. National Library of Medicine - NLM)<br/> **Bas van den Heuvel** (Philips Healthcare)<br/> **Chris Moesel** (MITRE)<br/> **Nikolai Ryzhikov** (Health Samurai)<br/> **Brian Kaney** (Vermonster)<br/> **Yury Sedinkin** (US NLM) |

This list inevitably fails to mention many of the individuals who contributed their time and energy to developing this specification and making it better. Thanks to all of you!


## Navigation model

FHIRPath navigates and selects nodes from a tree that abstracts away and is independent of the actual underlying implementation of the source against which the FHIRPath query is run. This way, FHIRPath can be used on in-memory Plain Old Java Objects (POJOs), XML data or any other physical representation, so long as that representation can be viewed as classes that have properties. In somewhat more formal terms, FHIRPath operates on a directed acyclic graph of classes as defined by a Meta Object Facility (MOF)-equivalent [\[MOF\]](#MOF) type system. In this specification, the structures on which FHIRPath operates are referred to as the Object Model.

Data are represented as a tree of labelled nodes, where each node may optionally carry a primitive value and have child nodes. Nodes need not have a unique label, and leaf nodes must carry a primitive value. For example, a (partial) representation of a FHIR Patient resource in this model looks like this:

![Tree representation of a Patient](treestructure.png){: height="375px" width="500px" style="float: unset; margin-bottom:0;" }

The diagram shows a tree with a repeating `name` node, representing repeating elements of the FHIR Object Model. Leaf nodes such as `use` and `family` carry a (string) value. It is also possible for internal nodes to carry a value, as is the case for the node labelled `active`: this allows the tree to represent FHIR "primitives", which may still have child extension data.

FHIRPath expressions are then _evaluated_ with respect to a specific instance, such as the Patient one described above. This instance is referred to as the _context_ (also called the _root_) and paths within the expression are evaluated in terms of this instance.

## Path selection

FHIRPath allows navigation through the tree by composing a path of concatenated labels, e.g.

``` fhirpath
name.given
```

This would result in a collection of nodes, one with the value `'Wouter'` and one with the value `'Gert'`. In fact, each step in such a path results in a collection of nodes by selecting nodes with the given label from the step before it. The input collection at the beginning of the evaluation contained all elements from Patient, and the path `name` selected just those named `name`. Since the `name` element repeats, the next step `given` along the path, will contain all nodes labeled `given` from all nodes `name` in the preceding step.

The path may start with the type of the root node (which otherwise does not have a name), but this is optional. To illustrate this point, the path `name.given` above can be evaluated as an expression on a set of data of any type. However the expression may be prefixed with the name of the type of the root:

``` fhirpath
Patient.name.given
```

The two expressions have the same outcome, but when evaluating the second, the evaluation will only produce results when used on data of type `Patient`.

When resolving an identifier that is also the root of a FHIRPath expression, it is resolved as a type name first. If it resolves to a type and that type is the type of the context (or a supertype), then the evaluation proceeds with the context unchanged, i.e. in the example above proceeds to evaluate the `name` element.
Otherwise the identifier is resolved as a path on the context, which if not found returns an empty collection.

This could also be written using [`ofType()`](#fn-oftype), which would have the same behaviour:
``` fhirpath
ofType(Patient).name.given
```

Syntactically, FHIRPath defines identifiers as any sequence of characters consisting only of letters, digits, and underscores, beginning with a letter or underscore. Paths may use other characters by using [delimited identifiers](#identifiers) - surrounding with backticks and using FHIRPath escaping as needed. This approach can also be used to encode nodes with names that are keywords. e.g.:

``` fhirpath
Message.`PID-1` // Identifier delimiting is required as the subtraction operator is part of the element's name
`as` as string  // Identifier delimiting is also required if an element's name is also a fhirpath keyword
```

### Collections

Collections are fundamental to FHIRPath, in that the result of every expression is a collection, even if that expression only results in a single item. This approach allows paths to be specified without having to care about the cardinality of any particular element, and is therefore ideally suited to graph traversal.

Within FHIRPath, a collection is:

* Ordered - The order of items in the collection is important and is preserved through operations as much as possible. Operators and functions that do not preserve order will note that in their documentation.
* Non-Unique - Duplicate values are allowed within a collection. Some operations and functions, such as `distinct()` and the union operator `|` produce collections of unique values, but in general, duplicate values are allowed.
* Indexed - Each item in a collection can be addressed by its index, i.e. ordinal position within the collection (e.g. `a[2]`).
* Unless specified otherwise by the underlying Object Model, the first item in a collection has index 0. Note that if the underlying model specifies that a collection is 1-based (the only reasonable alternative to 0-based collections), _any collections generated from operations on the 1-based list are 0-based_.
* Countable - The number of items in a given collection can always be determined using the `count()` function

Note that the outcome of functions like `children()` and `descendants()` cannot be assumed to be in any meaningful order, and `first()`, `last()`, `tail()`, `skip()` and `take()` should not be used on collections derived from these paths. Note that some implementations may follow the logical order implied by the object model, and some may not, and some may be different depending on the underlying source. Implementations may decide to return an error if an attempt is made to perform an order-dependent operation on a list whose order is undefined.

### Paths and polymorphic items

In the underlying representation of data, nodes may be typed and represent polymorphic items. Paths may either ignore the type of a node, and continue along the path or may be explicit about the expected node and filter the set of nodes by type before navigating down child nodes:

``` fhirpath
Observation.value.unit // all kinds of value
Observation.value.ofType(Quantity).unit // only values that are of type Quantity
```

The `is` operator can be used to determine whether or not a given value is of a given type:

``` fhirpath
Observation.value is Quantity // returns true if the value is of type Quantity
```

The `as` operator can be used to treat a value as a specific type:

``` fhirpath
Observation.value as Quantity // returns value as a Quantity if it is of type Quantity, and an empty result otherwise
```

The list of available types that can be passed as an argument to the [`ofType()`](#fn-oftype) function and `is` and `as` operators is determined by the underlying object model. Within FHIRPath, they are just identifiers, either delimited or simple.

## Expressions

FHIRPath expressions can consist of _paths_, _literals_, _operators_, and _function invocations_. These can be chained together, so that the output of one operation or function is the input to the next. This is the core of the _fluent_ [\[Fluent\]](#fluent) syntactic style and allows complex paths and expressions to be built up from simpler components.

### Literals

In addition to paths, FHIRPath expressions may contain _literals_, _operators_, and _function invocations_. 

Examples of the supported FHIRPath literals:
``` txt
Boolean: true, false
String: 'test string', 'urn:oid:3.4.5.6.7.8'
Integer: 0, 45
Long: 0L, 45L    // Long is defined as STU
Decimal: 0.0, 3.14159265
Date: @2015-02-04 (@ followed by ISO8601 compliant date)
DateTime: @2015-02-04T14:34:28+09:00 (@ followed by ISO8601 compliant date/time)
Time: @T14:34:28 (@ followed by ISO8601 compliant time beginning with T, no timezone offset)
Quantity: 10 'mg', 4 days
```

For each type of literal, FHIRPath defines a named system type to allow operations and functions to be defined, as well as an ultimate root type, `System.Any`. For example, the multiplication operator (`*`) is defined for the numeric types Integer and Decimal, as well as the Quantity type. See the discussion on [Models](#models) for a more detailed discussion of how these types are used within evaluation contexts.

#### Boolean

The `Boolean` type represents the logical Boolean values `true` and `false`. These values are used as the result of comparisons, and can be combined using logical operators such as `and` and `or`.

``` fhirpath
true
false
```

#### String
The `String` type represents string values up to 2<sup>31</sup>-1 characters in length and is UTF-8 encoded.

When parsing string literals, `\uXXXX` escape sequences represent [UTF-16](https://www.unicode.org/versions/Unicode16.0.0/core-spec/chapter-3/#G31699) code units; valid surrogate pairs are combined to form a single Unicode scalar value.
After processing escapes, a FHIRPath string is defined as a sequence of Unicode scalar values, which are referred to elsewhere in this specification as `characters` (for example, in [toChars()](#tochars--collection)).<br/>
This behaviour is consistent with the Unicode Standard and with the string literal processing rules used by most modern programming languages.
{:.stu}

String literals are surrounded by single-quotes and may use `\`-escapes to escape quotes and represent Unicode characters:

| Escape | Character |
| - | - |
| `\'` | Single-quote |
| `\"` | Double-quote |
| `` \` `` | Backtick |
| `\r` | Carriage Return |
| `\n` | Line Feed |
| `\t` | Tab |
| `\f` | Form Feed |
| `\\` | Backslash |
| `\uXXXX` | Unicode character escape, where XXXX is a four‑digit hexadecimal value representing a 16‑bit Unicode code unit.<br/><span class="stu">If the escape sequence represents a surrogate code unit, it must be paired with a corresponding high or low surrogate to form a valid Unicode scalar value.</span> |
{: .grid}

No other escape sequences besides those listed above are recognized.

Note that Unicode is supported in both string literals and delimited [Identifiers](#identifiers).

For example:
``` fhirpath
'test string'
'urn:oid:3.4.5.6.7.8'
```

If a `\` is used at the beginning of a non-escape sequence, it will be ignored and will not appear in the sequence:
``` fhirpath
'\p' // 'p'
'\\p' // '\p' ; normal escaped backslash
'\3' // '3'
'\u005' // 'u005' ; unicode escape sequences have 4 characters, not 3, hence not detected as an escaped sequence
'\' // '' ; an empty string
```

##### Unicode and String Operations
{:.stu}

String functions such as [`length()`](#length--integer), [`indexOf()`](#indexofsubstring--string--integer), [`substring()`](#substringstart--integer--length--integer--string), and [`toChars()`](#tochars--collection) operate on characters (Unicode scalar values), not on visual characters (grapheme clusters). This distinction matters when strings contain combining characters:
{:.stu}

| Visual Representation | Encoded String Literal | Unicode Scalar Value(s) | length() | Description |
| - | - | - | - | - |
| `é` (precomposed) | `'\u00E9'` | U+00E9 | 1 | Single character |
| `é` (combining) | `'\u0065\u0301'` | U+0065 U+0301 | 2 | Two characters: 'e' + combining acute accent |
| `🔥` (flame emoji) | `'\uD83D\uDD25'` | U+1F525 | 1 | Single character: A scalar value encoded using a UTF‑16 surrogate pair |
{: .grid}
{:.stu}

Both precomposed and combining representations are valid and may appear in data. Authors should be aware that the same visual character may have different lengths depending on its Unicode representation. When consistent string length behavior is required, data should be normalized (e.g., to NFC or NFD form) before processing, though such normalization is outside the scope of FHIRPath.
{:.stu}


#### Integer

The `Integer` type represents whole numbers in the range -2<sup>31</sup> to 2<sup>31</sup>-1.

``` fhirpath
0
45
-5
```

> Note that the minus sign (`-`) in the representation of a negative integer is not part of the literal, it is the unary negation operator defined as part of FHIRPath syntax.

##### Long
{:.stu}
> **Note:** The contents of this section are Standard for Trial Use (STU)
{: .stu-note }

The `Long` type represents whole numbers in the range -2<sup>63</sup> to 2<sup>63</sup>-1.
{:.stu}
``` fhirpath
0L
45L
-5L
```
{:.stu}

#### Decimal

The `Decimal` type represents real values in the range (-10<sup>28</sup>+1)/10<sup>8</sup> to (10<sup>28</sup>-1)/10<sup>8</sup> with a step size of 10<sup>-8</sup>. This range is defined based on a survey of decimal-value implementations and is based on the most useful lowest common denominator. §dec1:Implementations can provide support for larger decimals and higher precision, but SHALL provide at least the range and precision defined here.§ §dec2:In addition, implementations SHOULD use [fixed-precision decimal](https://en.wikipedia.org/wiki/Fixed-point_arithmetic) formats to ensure that decimal values are accurately represented.§

``` fhirpath
0.0
3.14159265
```

> **Note:** Decimal literals cannot use exponential notation. There is enough additional complexity associated with enabling exponential notation that this is outside the scope of what FHIRPath is intended to support (namely graph traversal).

#### Date

The `Date` type represents date and partial date values in the range @0001-01-01 to @9999-12-31 with a 1 day step size.

The `Date` literal is a subset of [\[ISO8601\]](#ISO8601):

* A date literal begins with an `@`
* It uses the `yyyy-MM-DD`{:.formatted} format, though month and day parts are optional, and a separator is required between provided components
* Week dates and ordinal dates are not allowed
* Years must be present (e.g. `@-10-20` is not a valid Date in FHIRPath)
* Months must be present if a day is present
* To specify a date and time together, see the description of `DateTime` below

The following examples illustrate the use of the `Date` literal:

``` fhirpath
@2014-01-25
@2014-01 // partial date (year-month)
@2014    // partial date (year only)
```

Consult the [formal grammar](grammar.html) for more details.

#### Time

The `Time` type represents time-of-day and partial time-of-day values in the range @T00:00:00.000 to @T23:59:59.999 with a step size of 1 millisecond. This range is defined based on a survey of time implementations and is based on the most useful lowest common denominator. §tm1:Implementations can provide support for higher precision, but SHALL provide at least the range and precision defined here.§ Time values in FHIRPath do not have a timezone or timezone offset.

The `Time` literal uses a subset of [\[ISO8601\]](#ISO8601):

* A time begins with a `@T`
* It uses the `Thh:mm:ss.fff`{:.formatted} format

The following examples illustrate the use of the `Time` literal:

``` fhirpath
@T14:30:14.559 // full Time including milliseconds
@T12:00        // partial time (hour and minute)
```

Consult the [formal grammar](grammar.html) for more details.

#### DateTime

The `DateTime` type represents date/time and partial date/time values in the range `@0001-01-01T00:00:00.000 to @9999-12-31T23:59:59.999` with a 1 millisecond step size. This range is defined based on a survey of datetime implementations and is based on the most useful lowest common denominator. §dt1:Implementations can provide support for larger ranges and higher precision, but SHALL provide at least the range and precision defined here.§

The `DateTime` literal combines the `Date` and `Time` literals and is a subset of [\[ISO8601\]](#ISO8601):

* A datetime literal begins with an `@`
* It uses the `yyyy-MM-DDThh:mm:ss.fff(+|-)hh:mm`{:.formatted} format
* Timezone offset is optional, but if present the notation `(+|-)hh:mm`{:.formatted} is used (so must include both minutes and hours)
* **Z** is allowed as a synonym for the zero (+00:00) UTC offset.
* A `T` can be used at the end of any date (year, year-month, or year-month-day) to indicate a partial DateTime.

The following example illustrates the use of the `DateTime` literal:

``` fhirpath
@2014-01-25T14:30:14.559
@2014-01-25T14:30:14.559Z // A date time with UTC timezone offset
@2014-01-25T14:30 // A partial DateTime with year, month, day, hour, and minute
@2014-03-25T // A partial DateTime with year, month, and day
@2014-01T // A partial DateTime with year and month
@2014T // A partial DateTime with only the year
```

The suffix `T` is allowed after a year, year-month, or year-month-day literal because without it, there would be no way to specify a partial DateTime with only a year, month, or day; the literal would always result in a Date value.

Consult the [formal grammar](grammar.html) for more details.

#### Quantity

The `Quantity` type represents quantities with a specified unit, where the `value` component is defined as a `Decimal`, and the `unit` is represented as a `String` that is required to be either a valid Unified Code for Units of Measure [\[UCUM\]](#UCUM) unit or one of the calendar duration keywords, singular or plural.

The `Quantity` literal is a number (integer or decimal), followed by a (single-quoted) **case-sensitive** string representing a valid Unified Code for Units of Measure [\[UCUM\]](#UCUM) unit or calendar duration keyword.
If the value literal is an Integer, it will be implicitly converted to a Decimal in the resulting Quantity value:


``` fhirpath
4.5 'mg'      // UCUM milligrams
100 '[degF]'  // UCUM temperature in Fahrenheit
2 years       // Calendar units
```

§q1:Implementations SHALL support [equality](#quantity-equality), [equivalence](#quantity-equivalence), [comparison](#comparison) and [arithmetic](#math-1) operations on quantities with units where the units are the same value, case-sensitively.§

§q2:Implementations that do NOT support UCUM unit conversion MAY return empty (`{ }`) for calculations involving quantities with different units, except for [equivalence](#quantity-equivalence), which returns `false` when a required conversion is not supported.§

§q3:Implementations that DO support UCUM conversions SHALL support implicit conversions where operations between quantities with different units are performed, and also explicitly through [toQuantity(unit)](#fn-toquantity).§

For Implementations that DO support UCUM conversion, if an operation other than [equivalence](#quantity-equivalence) is performed with conflicting units (for example, adding meters and grams), the evaluation will end and signal an error to the calling environment.

> **Note:** The UCUM specification defines what unit codes are valid, how units are composed/decomposed when performing multiplication and division, 
> and, where applicable, scalar conversion factors between commensurable units (e.g. between a single dimension such as length).
>
> UCUM does not specify how precision, rounding or comparisons are evaluated, FHIRPath specifies this in each section where required.<br>
> UCUM does not specify *which* operand should be converted when an operation has 2 operands of different units.


##### Time-valued Quantities

For time-valued quantities, in addition to the definite duration UCUM units, FHIRPath defines calendar duration keywords for calendar duration units:

| Calendar Duration | Unit Representation | Relationship to Definite Duration UCUM Unit |
| - | - | - |
| `year`/`years` | `'year'` | `~ 1 'a'` *(mean Julian year)* |
| `month`/`months` | `'month'` | `~ 1 'mo'` *(mean Julian month)* |
| `week`/`weeks` | `'week'` | `= 1 'wk'` |
| `day`/`days` | `'day'` | `= 1 'd'` |
| `hour`/`hours` | `'hour'` | `= 1 'h'` |
| `minute`/`minutes` | `'minute'` | `= 1 'min'` |
| `second`/`seconds` | `'second'` | `= 1 's'` |
| `millisecond`/`milliseconds` | `'millisecond'` | `= 1 'ms'` |
{: .grid}

The table above defines the equality/equivalence relationship between calendar and definite duration quantities.
For example, `1 year` is not [equal](#-equals) to `1 'a'` (i.e. results in empty), but it is [equivalent](#-equivalent) to `1 'a'`.<br/>
See the [Date/Time Arithmetic](#datetime-arithmetic) section for details on addition and subtraction of time-valued quantities to date/time values.

> **Note:** A calendar `day` is not technically equal/equivalent to a UCUM 'd', due to the possibility  of daylight saving time. 
> However for practical reasons it considers them to be equal.
> Authors should take care to ensure calendar days are used in applications that are sensitive to this difference.

UCUM defines the conversion factors between UCUM units, and FHIRPath defines [conversion factors](#fn-toquantity-conversion-factors) for calendar units.

For example, the following quantities are _calendar duration_ quantities:

``` fhirpath
1 year
4 days
```

Whereas the following quantities are _definite duration_ quantities:

``` fhirpath
1 'a'
4 'd'
```

> **Note:** UCUM is represented with the system `http://unitsofmeasure.org` and Calendar Units with the system [http://hl7.org/fhirpath/CodeSystem/calendar-units](CodeSystem-calendar-units.html).
> These systems may be used in contexts that interact with FHIRPath Quantities, such as the FHIR `Quantity` datatype. 
> Note that the FHIR [`Duration`](http://hl7.org/fhir/datatypes.html#Duration) datatype is an exception; it only supports UCUM units.
{:.stu}

### Operators

Expressions can also contain _operators_, like those for mathematical operations and boolean logic:

``` fhirpath
Appointment.minutesDuration / 60 > 5
MedicationAdministration.wasNotGiven implies MedicationAdministration.reasonNotGiven.exists()
name.given | name.family // union of given and family names
'sir ' + name.given
```

Operators available in FHIRPath are covered in detail in the [Operations](#operations) section.

### Function Invocations

Finally, FHIRPath supports the notion of functions, which operate on a collection of values (referred to as the _input collection_), optionally taking arguments, and return another collection (referred to as the _output collection_). For example:

``` fhirpath
name.given.substring(0,4)
identifier.where(use = 'official')
```

Since all functions work on input collections, constants will first be converted to a collection when functions are invoked on constants:

``` fhirpath
(4+5).count()
```

will return `1`, since the input collection is implicitly a collection with one constant number `9`.

In general, functions in FHIRPath operate on collections and return new collections. This property, combined with the syntactic style of _dot invocation_ enables functions to be chained together, creating a _fluent_-style syntax:

``` fhirpath
Patient.telecom.where(use = 'official').union(Patient.contact.telecom.where(use = 'official')).exists().not()
```

However, not all functions support multiple items in the input collection, some expect only a single item and will be explicitly documented. Further details are available in the ["singleton evaluation of collections"](#singleton-evaluation-of-collections) section.

Singleton only functions can be run on collections by using the function inside a [`select(...)`](#fn-select) to evaluate the function for each item in the collection.

For a complete listing of the functions defined in FHIRPath, refer to the [Functions](#functions) section.

### Null and empty

There is no literal representation for _null_ in FHIRPath. This means that when, in an underlying data object (i.e. they physical data on which the implementation is operating) an element is null or missing, there will simply be no corresponding node for that element in the tree, e.g. `Patient.name`{:.fhirpath} will return an empty collection (not null) if there are no name elements in the instance.

In expressions, the empty collection is represented as `{ }`.

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

``` txt
IF the collection contains a single node AND the node's value can be implicitly converted to the expected input type THEN
  The collection evaluates to the value of that single node
ELSE IF the collection contains a single node AND the expected input type is Boolean THEN
  The collection evaluates to true
ELSE IF the collection is empty THEN
  The collection evaluates to an empty collection
ELSE
  The evaluation will end and signal an error to the calling environment
```

For example:

``` fhirpath
Patient.name.family + ', ' + Patient.name.given
```

If the `Patient` instance has a single `name`, and that name has a single `given`, then this will evaluate without any issues. However, if the `Patient` has multiple `name` elements, or the single name has multiple `given` elements, then it's ambiguous which of the elements should be used as the input to the `+` operator, and the result is an error.

As another example:

``` fhirpath
Patient.active and Patient.gender and Patient.telecom
```

Assuming the `Patient` instance has an `active` value of `true`, a `gender` of `female` and a single `telecom` element, this expression will result in `true`. However, consider a different instance of `Patient` that has an `active` value of `true`, a `gender` of `male`, and multiple `telecom` elements, then this expression will result in an error because of the multiple telecom elements.

Note that for repeating elements like `telecom` in the above example, the logic _looks_ like an existence check. To avoid confusion and reduce unintended errors, authors should use the explicit form of these checks when appropriate. For example, a more explicit rendering of the same logic that more clearly indicates the actual intent and avoids the run-time error is:

``` fhirpath
Patient.active and Patient.gender and Patient.telecom.count() = 1
```

## Functions

Functions are distinguished from path navigation names by the fact that they are followed by `()` with zero or more arguments. Throughout this specification, the word _parameter_ is used to refer to the definition of a parameter as part of the function definition, while the word _argument_ is used to refer to the values passed as part of a function invocation. With a few minor exceptions (e.g. [current date and time functions](#current-date-and-time-functions)), functions in FHIRPath operate on a collection of values (referred to as the _input collection_) and produce another collection as output (referred to as the _output collection_). However, for many functions, passing an input collection with more than one item is defined as an error condition. Each function defines its behavior for input collections of any cardinality (0, 1, or many).

This approach allows functions to be chained, successively operating on the results of the previous function in order to produce the desired final result.

The following sections describe the functions supported in FHIRPath, detailing the expected types of parameters, when and how they are evaluated, and the type of output collections returned by the function:

* Although the function parameters are defined with a specific type, they are expressed as fhirpath expressions that will return the specific type (where a type is defined)<br/> - often the expression is a simple constant
* If a function expects the argument passed to a parameter to be a single value (e.g. `startsWith(prefix: String)`) and it is passed an argument that evaluates to a collection with multiple items, or to a collection with an item that is not of the required type (or cannot be converted to the required type), the evaluation of the expression will end and an error will be signaled to the calling environment.
* Square bracket notation `[]` is used in function signatures to indicate optional parameters.<br/> (e.g. `toQuantity([unit : String]) : Quantity`)
* If a parameter doesn't require a specific type, and supports collections, then the parameter type will be defined as `collection`
* If a parameter doesn't require a specific type, but does not support collections, then the parameter type will be defined as `any`
* Some parameters passed to a scoped function (documented below) will set and use some special variables when the parameter expression is evaluated.<br/>
  This is only intended to provide clarity on the usage of the expression, not how it is actually written.<br/>
  For example, the `where` function's definition is `where(criteria : ($this, $index) => Boolean)` indicating that when the `criteria` expression is evaluated, the special variables `$this` and `$index` will be set in the evaluation context according to the definition of the function.
  When you actually use it, you would write something like `rules.where($index mod 2 = 0)` (which will filter out every second rule from the collection). Usage in this case is the same as if the parameter was a simple `Boolean` parameter in the definition.

Note that although all functions return collections, if a given function is defined to return a single item, the return type is simplified to just the type of the single item, rather than the list type.

#### Scoped Functions
Some functions are marked as **scoped** functions. This type of function creates special variables (such as `$this`) that are available when evaluating arguments to the function.
Most of these functions iterate through the items in the input collection, allowing the current item to be accessed.

For example:
``` fhirpath
(1 | 2 | 3 | 4 | 5).where($this > 3) // filter the input collection to values that are greater than 3
```
The documentation for each scoped function defines what variables are available and how/when they are used as part of evaluating the function arguments.

> **Note:** Variables introduced by scoped functions can only be accessed within the specified parameters of the scoped function. This behavior is preserved through nesting, meaning specifically that once a nested scope function evaluation is complete, the variables in the outer scoped function are again available.

These are the fhirpath defined scoped functions: *(argument processing only, refer to each function for full details of its functionality)*

| Scoped Function | Argument Processing Logic |
| --------------- | ------------------------- |
| [`exists`](#fn-exists) | The `criteria` argument is evaluated for each item (setting `$this` and `$index` before each iteration); if any return `true` then the function returns `true`, otherwise `false`. |
| [`all`](#fn-all) | The `criteria` argument is evaluated for each item (setting `$this` and `$index` before each iteration); if all return `true` then the function returns `true`, otherwise `false`. An empty input collection returns `true`. |
| [`where`](#fn-where) | The `criteria` argument is evaluated for each item (setting `$this` and `$index` before each iteration); those that return `true` are included in the output collection. |
| [`select`](#fn-select) | The `projection` argument is evaluated for each item (setting `$this` and `$index` before each iteration); and the results are included in the output collection. |
| [`sort`](#fn-sort) | Each `keySelector` argument is evaluated for each item being compared (setting `$this` to the item for each evaluation). The results are compared to determine sort order. If there are multiple `keySelector` arguments, subsequent selectors are only evaluated for items where the previous `keySelector` comparison resulted in equality (i.e., the sort order hasn't been determined yet). This allows for multi-level sorting with minimal evaluations. <br/>As this function is used to modify the order of the collection the `$index` variable is not defined in this context, it could be anywhere during any evaluation depending on algorithms selected. |
| [`repeat`](#fn-repeat) | The `projection` argument is evaluated for each item (setting `$this` before each iteration); and the results are included in the output collection. The function is then evaluated on the new items from the previous iteration, repeating until no new items are added.<br/>Note: As the function iterates on itself, the meaning of `$index` is undefined and not set here. |
| [`repeatAll`](#fn-repeatall) | The `projection` argument is evaluated for each item (setting `$this` before each iteration); and the results are included in the output collection. The function is then evaluated on the new items from the previous iteration, repeating until no new items are added.<br/>Note: As the function iterates on itself, the meaning of `$index` is undefined and not set here. |
| [`iif`](#iif) | The `criterion` argument is evaluated once (with `$this` set to the input value).<br/> If it returns `true`, then the `true-result` argument is evaluated (with `$this` set to the input value) and returned,<br/> otherwise the `otherwise-result` argument is evaluated (with `$this` set to the input value) and returned. |
| [`trace`](#fn-trace) | If no `projection` argument is provided, the input collection is logged without the need for scoping. If the `projection` argument is provided, it is evaluated for each item (setting `$this` and `$index` before each iteration) and the result logged. The input collection is returned as the result of the function. |
| [`aggregate`](#aggregate) | The `init` argument is evaluated once at the start to initialize the `$total` variable.<br/> The `aggregator` argument is then evaluated for each item (setting `$this`and `$index` for each), and has access to the current value of `$total` available. The result of the evaluation is then assigned to `$total`.<br/> The final value of `$total` is returned as the result of the function.<br/> The `init` argument is evaluated once before setting `$this` and `$index`, so will be evaluated on the outer context, and will have access to outer `$this` values. |
{:.list}

For example (some expressions using scoped functions and accessing the special `$this` variable):
```fhirpath
// Retrieve a list of formatted names for the patient
// (with no unwanted padding white-space)
Patient.name.select(given.join(' ').combine($this.family, true).join(', '))

// Observation values that are outside a specific range
Observation.value.where($this < 90 or $this > 110)
```

> **Note:** Scoped functions are similar to lambda functions, closures, or callbacks in other languages.
> They allow passing expressions as parameters that are evaluated in a specific context with special variables like `$this` and `$index`.

#### Special variables

| Variable | Description |
| - | - |
| `$this` | Set at the beginning of execution of an expression as the initial context *(See `%context` below)*<br/> Re-set to the current item being processed in [scoped functions](#scoped-functions).<br/> *Refer to each scoped function for specific details* |
| `$index` | Set at the beginning of execution of an expression to 0<br/> Re-set to the index of the current item being processed in [scoped functions](#scoped-functions).<br/> *Refer to each scoped function for specific details*<br/> Its value is undefined while evaluating [`sort`](#fn-sort) `keySelector` parameters |
| `$total` | Only available inside the parameters of the [`aggregate`](#aggregate) function. Holds the running total during processing, and at the end will be the result returned by the function. |
| `%context` | The entry/starting point for execution of the fhirpath expression.<br/> Often used in fhirpath invariants.<br/> *(Does not change during execution)* |
| `%resource` | (Defined in FHIR) The current resource being processed (that contains the element in focus)<br/> When passing through `resolve()` or into a contained resource will be changed to the new resource context. |
| `%rootResource` | (Defined in FHIR) The top level fhir resource. Usually a Bundle, or resource that has contained resources (or Parameters resource).<br/> Though processing on regular fhir resources this is also the same as %resource.<br/> *(Does not change during execution)* |
{:.list}

> **Note:** §v1:Other specifications MAY introduce their own variables§

### Existence

#### empty() : Boolean

Returns `true` if the input collection is empty (`{ }`) and `false` otherwise.

The following example returns `true` when the Patient has no link elements:
``` fhirpath
Patient.link.empty()
```

<a name="fn-exists"></a>
#### exists([criteria : ($this, $index) => Boolean]) : Boolean

> This is a [scoped function](#scoped-functions): The `criteria` argument is evaluated for each item (setting `$this` and `$index` before each iteration); if any return `true` then the function returns `true`, otherwise `false`.

Returns `true` if the input collection has any items (optionally filtered by the criteria), and `false` otherwise.
This is the opposite of `empty()`, and as such is a shorthand for `empty().not()`. If the input collection is empty (`{ }`), the result is `false`.

Using the optional criteria can be considered a shorthand for `where(criteria).exists()`.

Note that a common term for this function is _any_.

The following examples illustrate some potential uses of the `exists()` function:

``` fhirpath
Patient.name.exists()
Patient.identifier.exists(use = 'official')
Patient.telecom.exists(system = 'phone' and use = 'mobile')
Patient.generalPractitioner.exists(resolve() is Practitioner)
```

The first example returns `true` if the `Patient` has any `name` elements.

The second example returns `true` if the `Patient` has any `identifier` elements that have a `use` element equal to `'official'`.

The third example returns `true` if the `Patient` has any `telecom` elements that have a `system` element equal to `'phone'` and a `use` element equal to `'mobile'`.

And finally, the fourth example returns `true` if the `Patient` has any `generalPractitioner` elements of type `Practitioner`.

<a name="fn-all"></a>
#### all(criteria : ($this, $index) => Boolean) : Boolean

> This is a [scoped function](#scoped-functions): The `criteria` argument is evaluated for each item (setting `$this` and `$index` before each iteration); if all return `true` then the function returns `true`, otherwise `false`. An empty input collection returns `true`.

Returns `true` if for every item in the input collection, `criteria` evaluates to `true`. Otherwise, the result is `false`. If the input collection is empty (`{ }`), the result is `true`.

The following example returns `true` if all of the `generalPractitioner` elements are of type `Practitioner`:
``` fhirpath
generalPractitioner.all($this.resolve() is Practitioner)
```

#### allTrue() : Boolean

Takes a collection of Boolean values and returns `true` if all the items are `true`. If any items are `false`, the result is `false`. If the input is empty (`{ }`), the result is `true`.

The following example returns `true` if all of the components of the Observation have a value greater than 90 mm[Hg]:

``` fhirpath
Observation.component.select(value > 90 'mm[Hg]').allTrue()
```

#### anyTrue() : Boolean

Takes a collection of Boolean values and returns `true` if any of the items are `true`. If all the items are `false`, or if the input is empty (`{ }`), the result is `false`.

The following example returns `true` if any of the components of the Observation have a value greater than 90 mm[Hg]:

``` fhirpath
Observation.component.select(value > 90 'mm[Hg]').anyTrue()
```

#### allFalse() : Boolean

Takes a collection of Boolean values and returns `true` if all the items are `false`. If any items are `true`, the result is `false`. If the input is empty (`{ }`), the result is `true`.

The following example returns `true` if none of the components of the Observation have a value greater than 90 mm[Hg]:

``` fhirpath
Observation.component.select(value > 90 'mm[Hg]').allFalse()
```

#### anyFalse() : Boolean

Takes a collection of Boolean values and returns `true` if any of the items are `false`. If all the items are `true`, or if the input is empty (`{ }`), the result is `false`.

The following example returns `true` if any of the components of the Observation have a value that is not greater than 90 mm[Hg]:

``` fhirpath
Observation.component.select(value > 90 'mm[Hg]').anyFalse()
```

#### subsetOf(other : collection) : Boolean

Returns `true` if all items in the input collection are members of the collection passed as the `other` argument.
Membership is determined when the [equals](#equals) (`=`) operator returns `true` *(i.e. `false` and empty both indicate that an item is not a member)*.

Conceptually, this function is evaluated by testing each item in the input collection for membership in the `other` collection, with a default of `true`. This means that if the input collection is empty (`{ }`), the result is `true`, otherwise if the `other` collection is empty (`{ }`), the result is `false`.

The following example returns `true` if the tags defined in any contained resource are a subset of the tags defined in the MedicationRequest resource:

``` fhirpath
MedicationRequest.contained.meta.tag.subsetOf(MedicationRequest.meta.tag)
```

#### supersetOf(other : collection) : Boolean

Returns `true` if all items in the collection passed as the `other` argument are members of the input collection.
Membership is determined when the [equals](#equals) (`=`) operator returns `true` *(i.e. `false` and empty both indicate that an item is not a member)*.

Conceptually, this function is evaluated by testing each item in the `other` collection for membership in the input collection, with a default of `true`. This means that if the `other` collection is empty (`{ }`), the result is `true`, otherwise if the input collection is empty (`{ }`), the result is `false`.

The following example returns `true` if the tags defined in any contained resource are a superset of the tags defined in the MedicationRequest resource:

``` fhirpath
MedicationRequest.contained.meta.tag.supersetOf(MedicationRequest.meta.tag)
```

#### count() : Integer

Returns the integer count of the number of items in the input collection. Returns 0 when the input collection is empty.

The following example returns the number of names on the Patient:
``` fhirpath
Patient.name.count()
```

#### distinct() : collection

Returns a collection containing only the unique items in the input collection.
Items are considered to be equal when the [equals](#equals) (`=`) operator returns `true` *(i.e. `false` and empty both indicate that the items are distinct)*.

If the input collection is empty (`{ }`), the result is empty.

Note that the order of items in the input collection is not guaranteed to be preserved in the result.

The following example returns the distinct list of tags on the given Patient:

``` fhirpath
Patient.meta.tag.distinct()
```

As the equality operator for quantities can support unit conversions, this also applies here, so for example the following expression would return a collection with a single item (either `1 'm'` or `100 'cm'`, depending on the implementation), as they are considered equal:
``` fhirpath
1 'm' | 100 'cm' // 1 'm' (or could be 100 'cm')
```

#### isDistinct() : Boolean

Returns `true` if all the items in the input collection are distinct.
Items are considered to be equal when the [equals](#equals) (`=`) operator returns `true` *(i.e. `false` and empty both indicate that the items are distinct)*.

Conceptually, this function is shorthand for a comparison of the `count()` of the input collection against the `count()` of the `distinct()` of the input collection:

``` fhirpath
X.count() = X.distinct().count()
```

This means that if the input collection is empty (`{ }`), the result is `true`.

For example:
``` fhirpath
1.combine(2).combine(3).isDistinct() // true
1.combine(2).combine(3).combine(2).isDistinct() // false ; the 2 appears twice in the input collection
1 'm'.combine(100 'cm').isDistinct() // false ; both quantities are the same (via unit conversion)
```

### Filtering and projection

<a name="fn-where"></a>
#### where(criteria : ($this, $index) => Boolean) : collection

> This is a [scoped function](#scoped-functions): The `criteria` argument is evaluated for each item (setting `$this` and `$index` before each iteration); those that return `true` are included in the output collection.

Returns a collection containing only those items in the input collection for which the stated `criteria` expression evaluates to `true`. Items for which the expression evaluates to `false` or empty (`{ }`) are not included in the result.

If the input collection is empty (`{ }`), the result is empty.

If the result of evaluating the condition is other than a single boolean value, the evaluation will end and signal an error to the calling environment, consistent with singleton evaluation of collections behavior.

The following example returns the list of `telecom` elements that have a `use` element with the value of `'official'`:

``` fhirpath
Patient.telecom.where(use = 'official')
```

<a name="fn-select"></a>
#### select(projection: ($this, $index) => any) : collection

> This is a [scoped function](#scoped-functions): The `projection` argument is evaluated for each item (setting `$this` and `$index` before each iteration); and the results are included in the output collection.

Evaluates the `projection` expression for each item in the input collection. The result of each evaluation is added to the output collection. If the evaluation results in a collection with multiple items, all items are added to the output collection (collections resulting from evaluation of `projection` are _flattened_). This means that if the evaluation for an item results in the empty collection (`{ }`), no item is added to the result, and that if the input collection is empty (`{ }`), the result is empty as well.

The following example results in a collection with only the patient resources from the bundle:
``` fhirpath
Bundle.entry.select(resource as Patient)
```

The following example results in a collection with all the telecom elements with system of `phone` for all the patients in the bundle:
``` fhirpath
Bundle.entry.select((resource as Patient).telecom.where(system = 'phone'))
```

The following example returns a collection containing the concatenation of the first given and family names for each of the Patient's "usual" names:
``` fhirpath
Patient.name.where(use = 'usual').select(given.first() & ' ' & family)
```

<a name="instance-selector"></a>
#### Instance Selector/Object Creation
{:.stu}
> **Note:** The contents of this section are Standard for Trial Use (STU)
{: .stu-note }

*Although the instance selector is not a function, it is closely related to the `select` function in that this statement is
used to convert the input collection into a different output collection. Its sub-expression format is:*
{:.stu}

> ```
> [«typename»] { element : value, element : value, ...  } : «typename»
> ```
> `«typename»` is the optional name of the type to create (optionally prefixed with a namespace)<br/>
> `element` is the name *(identifier)* of an element of the type being created<br/>
> `value` is any fhirpath expression (including literals) to set to the associated `element`<br/>
{:.stu}

Creates a new object of type `«typename»` and returns that to the output collection.
§ic1:If the `«typename»` is omitted, an implementation MAY use a type appropriate to the context, or an anonymous type.§<br/> 
Any elements listed within the parentheses are set as children on the newly created object.
{:.stu}

If any of child element's `value` is evaluated to an empty collection, then that element will not be added to the object.
{:.stu}

To create an empty object requires the use of `{ : }` to differentiate it from the empty set.
{:.stu}

§ic2:If the input collection contains multiple items, the evaluation of the expression will end and signal an error to the calling environment.
However element selectors can return multiples where the object's structure supports it for that element. 
If not the engine MAY throw an error.§
{:.stu}

If the input collection is empty, the result is empty.
{:.stu}

Some examples of creating data using static instance selectors:
{:.stu}
``` fhirpath
// create a static coding
Coding { system : 'http://example.org/demo', code : 'c1' }

// create an simple identifier, explicitly defining the FHIR namespace
FHIR.Identifier { system : 'http://example.org/demo', value : 'N0001231' }

// create an MRN identifier similar to the one in the patient example at https://hl7.org/fhir/patient-example.json.html
// (no explicit types on the child properties in the creation, there's only 1 valid type for each context here - so can be derived)
Identifier { 
  type : { coding: { system: 'http://terminology.hl7.org/CodeSystem/v2-0203', code: 'MR' } },
  system : 'urn:oid:1.2.36.146.595.217.0.1',
  value : '12345',
  period : { start: @2001-05-06 }
}

// create an empty Period
Period {:}
```
{:.stu}

Instance Selectors are usually used to manipulate existing elements into another datatype:
{:.stu}

``` fhirpath
// Convert the patient gender from a code into a coding
Patient.select( 
  Coding { system: 'http://terminology.hl7.org/CodeSystem/v2-0203', code: gender }
)

// Convert a set of concepts from a code system into Codings
CodeSystem.concept.select(Coding { system: %resource.url, code: code, display: display })

// Create an anonymous type with 2 properties
{ foo: 'bar', mol: 42 }
```
{:.stu}

**Note:** Primitive types can be created and set using a special element name `value` if specifically required.
The evaluating fhirpath engine is responsible for performing any type conversions from fhirpath primitives to the target object/type system as required.
{:.stu}
``` fhirpath
code { value: 'final' }
```
{:.stu}

Collection elements can be populated with multiple values using `|`, `combine` or an expression that results in multiple items:
{:.stu}
```fhirpath
// Set the codings via an expression that returns multiple codings (using |)
CodeableConcept {
   coding: Coding { system: 'http://terminology.hl7.org/CodeSystem/v2-0203', code: 'MR' }
         | Coding { system: 'http://example.org/id-types', code: 'mr' }
}

// create a codeable concept that contains all of the codes selected
// multiple codings come from the answers in the questionnaire
QuestionnaireResponse.item.where(linkId='coded-q')
.select( CodeableConcept { coding: answer.value.ofType(Coding) } )
```
{:.stu}

<a name="fn-sort"></a>
#### sort([keySelector: ($this) => any [asc | desc] [, keySelector: ($this) => any [asc | desc], ...]]) : collection
{:.stu}
> **Note:** The contents of this section are Standard for Trial Use (STU)
{: .stu-note }

> This is a [scoped function](#scoped-functions): Each `keySelector` argument is evaluated for each item being compared (setting `$this` to the item for each evaluation). The results are compared to determine sort order. If there are multiple `keySelector` arguments, subsequent selectors are only evaluated for items where the previous `keySelector` comparison resulted in equality (i.e., the sort order hasn't been determined yet). This allows for multi-level sorting with minimal evaluations. <br/>As this function is used to modify the order of the collection the `$index` variable is undefined in this context, it could be anywhere during any evaluation depending on algorithms selected.
{:.stu}

Returns a collection containing the items in the input collection, sorted according to the specified key selector expressions. The function takes a variable number of key selector parameters, each of which can be optionally qualified with `asc` (ascending) or `desc` (descending). If no qualifier is provided, `asc` is the default.
{:.stu}

If no key selector parameters are provided, the sort uses the default ordering for the type of data in the input collection, using the same comparison semantics as defined for the equals (`=`) and comparison operators (`>`, `>=`, `<`, `<=`).
{:.stu}

Each key selector expression is evaluated for each item in the input collection using singleton evaluation semantics. If the key selector expression evaluates to a collection with more than one item, the evaluation will end and signal an error to the calling environment.
{:.stu}

comparing values returned by the no key selector using the same comparison semantics as defined for the equals (`=`) and comparison operators (`>`, `>=`, `<`, `<=`).
{:.stu}

An empty value is considered lower than all other values, meaning they will appear before others when sorted ascending.
{:.stu}

When comparing two items, if the values for the first key selector are equal, the comparison proceeds to the next key selector, and so on until all key selectors have been evaluated or a difference is found.
Items are considered equal if and only if the [equals](#equals) (`=`) operator returns `true`. *(i.e. `false` and empty both indicate that the items are not equal).*
{:.stu}

Attempting to sort items with incompatible types will result in an error. Values that would result in comparison errors must be filtered from the collection prior to sorting.
{:.stu}

If the input collection is empty (`{ }`), the result is empty.
{:.stu}

The following examples illustrate the use of the `sort()` function:
{:.stu}

``` fhirpath
(3 | 1 | 2).sort() // 1, 2, 3 ; natural numeric ordering
(3 | 1 | 2).sort($this) // 1, 2, 3 ; explicit ascending
(3 | 1 | 2).sort($this desc) // 3, 2, 1 ; descending
('c' | 'a' | 'b').sort() // 'a', 'b', 'c' ; default string ordering
('c' | 'a' | 'b').sort($this desc) // 'c', 'b', 'a' ; descending
('3' | '1' | '10').sort() // '1', '10', '3' ; default string ordering (not numeric)
Patient.name.sort(family desc, given.first()) // sort by family name descending, then by first given name ascending
Patient.telecom.sort(system, use desc) // sort by system ascending, then by use descending
```
{:.stu}

<a name="fn-repeat"></a>
#### repeat(projection: ($this) => collection) : collection

> This is a [scoped function](#scoped-functions): The `projection` argument is evaluated for each item (setting `$this` before each iteration); and the results are included in the output collection. The function is then evaluated on new items from the previous iteration, repeating until no new items are added.<br/>Note: As the function iterates on itself, the meaning of `$index` is undefined and not set here.

A version of `select` that will repeat the `projection` and add items to the output collection only if they are not already in the output collection as determined by the [equals](#equals) (`=`) operator returning `true` *(i.e. `false` and empty both indicate that the values are not equal and thus added to the output)*.

This can be evaluated by adding all items in the input collection to an input queue, then for each item in the input queue evaluate the repeat expression. If the result of the repeat expression is not in the output collection, add it to both the output collection and also the input queue. Processing continues until the input queue is empty.

This function can be used to traverse a tree and selecting only specific children:

``` fhirpath
ValueSet.expansion.repeat(contains)
```

Will repeat finding children called `contains`, until no new items are found.

``` fhirpath
Questionnaire.repeat(item)
```

Will repeat finding children called `item`, until no new items are found.

Note that this is slightly different from:

``` fhirpath
Questionnaire.descendants().select(item)
```

which would find *any* descendants called `item`, not just the ones nested inside other `item` elements.

The order of items returned by the `repeat()` function is undefined.

> §r1:Implementations SHOULD include safety mechanisms to prevent infinite loops.§ §r2:An implementation MAY impose a limit on the number of iterations, or MAY statically analyze the expression to ensure it navigates to child elements within the hierarchical structure.§
> §r3:If an infinite loop is detected, or considered likely, the evaluation MAY end and signal an error to the calling environment.§
{:.dragon}


<a name="fn-repeatall"></a>
#### repeatAll(projection: ($this) => collection) : collection
{:.stu}
> **Note:** The contents of this section are Standard for Trial Use (STU)
{: .stu-note }

> This is a [scoped function](#scoped-functions): The `projection` argument is evaluated for each item (setting `$this` before each iteration); and the results are included in the output collection. The function is then evaluated on the new items from the previous iteration, repeating until no new items are added.<br/>Note: As the function iterates on itself, the meaning of `$index` is undefined and not set here.
{:.stu}

A version of `repeat` that allows duplicate items in the output collection. Unlike `repeat`, this function does not check whether items are already present in the output collection before adding them.
{:.stu}

This can be evaluated by adding all items in the input collection to an input queue, then for each item in the input queue evaluate the expression. The results are added to the output collection and also to a new iteration queue, regardless of whether they already exist in either collection. The input queue is then replaced by the new iteration queue and processing continues until there are no more items in the input queue to process.
{:.stu}

This function provides better performance than `repeat` by eliminating the equality comparisons required to check for duplicates, while still providing more targeted traversal than `descendants()`.
{:.stu}

The order of items returned by the `repeatAll()` function is undefined.
{:.stu}

> §r4:Implementations SHOULD include safety mechanisms to prevent infinite loops. An implementation MAY impose a limit on the number of iterations, or MAY statically analyze the expression to ensure it navigates to child elements within the hierarchical structure.§
> §r5:If an infinite loop is detected, or considered likely, the evaluation MAY end and signal an error to the calling environment.§
{:.stu .dragon}

Safe usage typically relies on the hierarchical structure of the input data. Expressions that reference children of the input collection's items will naturally terminate when no more child elements are found.
{:.stu}

Some safe expressions:
{:.stu}
``` fhirpath
ValueSet.expansion.repeatAll(contains)
Questionnaire.repeatAll(item)
```
{:.stu}

Some unsafe expressions:
{:.stu}
``` fhirpath
Questionnaire.repeatAll('item') // this is a common mistake where the element name is accidentally provided as a string, not an identifier, so each iteration re-evaluates the same string constant and the function never terminates.
'abc'.repeatAll(replace('a', 'A')) // the projection always returns a value (after the first iteration it produces 'Abc'), and since repeatAll does not remove duplicates, the same item keeps being re-added and the function never terminates.
```
{:.stu}

<a name="fn-oftype"></a>
#### ofType(type : _type specifier_) : collection

Returns a collection that contains all items in the input collection that are of the given type or a subclass thereof. If the input collection is empty (`{ }`), the result is empty.
The `type` argument is an identifier that must resolve to the name of a type in a model.
For implementations with compile-time typing, this requires special-case handling when processing the argument to treat it as type specifier rather than an identifier expression.

In the following example, the symbol `Patient` must be treated as a type identifier rather than a reference to a Patient in context:
``` fhirpath
Bundle.entry.resource.ofType(Patient)
```

<a name="coalesce"></a>
#### coalesce(value : collection, [value : collection, ...]) : collection
{:.stu}
> **Note:** The contents of this section are Standard for Trial Use (STU)
{: .stu-note }
The `coalesce` function takes a variable number of arguments, each of which is a collection. It returns the first non-empty collection from the arguments. If all arguments are empty collections, the result is an empty collection.
{:.stu}

Note that short-circuit behaviour is expected in this function. In other words, arguments after the first non-empty argument are not evaluated. For implementations, this means delaying evaluation of the arguments (as is done with `iif`).
{:.stu}

```fhirpath
Patient.coalesce(name.where(use='official'), name.where(use='usual'), name.first()).text // preferentially select name via use
Patient.name.select(coalesce(family & ' ' & given.join(', '), text, 'unknown')) // select is required to process each name separately
coalesce(Patient.identifier.where(system = 'http://example.org/identifier').value.first(), 'unknown')
```
{:.stu}

Note that this function is useful for providing fallback options, and is more concise than using `iif` to check each collection in turn.
{:.stu}

Such as selecting specific telecom elements based on their use, and falling back to the first available telecom element if none of the specific uses are present:
{:.stu}
```fhirpath
iif( telecom.where(use='mobile').exists(), telecom.where(use='mobile'),
    iif( telecom.where(use='home').exists(), telecom.where(use='home'),
        iif( telecom.where(use='work').exists(), telecom.where(use='work'),
          telecom))).first()
// could equivalently be written as:
coalesce(telecom.where(use='mobile'), telecom.where(use='home'), telecom.where(use='work'), telecom).first()
```
{:.stu}

Another common case is to select a specific coding in a CodeableConcept if it is available, otherwise whatever coding is available.
{:.stu}
``` fhirpath
iif( code.coding.where(system='http://snomed.info/sct').exists(),
        code.coding.where(system='http://snomed.info/sct'),
            code.coding)
    .first().code
// could equivalently be written as:
coalesce(code.coding.where(system='http://snomed.info/sct'), code.coding).first().code
```
{:.stu}


### Subsetting

#### [ index : Integer ] : any

The indexer operation returns a collection with only the `index`-th item (0-based index). If the input collection is empty (`{ }`), or the index lies outside the boundaries of the input collection, an empty collection is returned.

> **Note:** Unless specified otherwise by the underlying Object Model, the first item in a collection has index 0. Note that if the underlying model specifies that a collection is 1-based (the only reasonable alternative to 0-based collections), _any collections generated from operations on the 1-based list are 0-based_.

The following example returns the item in the `name` collection of the Patient at index 0:

``` fhirpath
Patient.name[0]
```

#### single() : any

Will return the single item in the input if there is just one item. If the input collection is empty (`{ }`), the result is empty. If there are multiple items, an error is signaled to the evaluation environment. This function is useful for ensuring that an error is returned if an assumption about cardinality is violated at run-time.

The following example returns the name of the Patient if there is one. If there are no names, an empty collection, and if there are multiple names, an error is signaled to the evaluation environment:

``` fhirpath
Patient.name.single()
```

#### first() : any

Returns a collection containing only the first item in the input collection. This function is equivalent to `item[0]`, so it will return an empty collection if the input collection has no items.

The following example returns the first name element, equivalent to `Patient.name[0]`:
``` fhirpath
Patient.name.first()
```

#### last() : any

Returns a collection containing only the last item in the input collection. Will return an empty collection if the input collection has no items.

The following example returns the last name element in the collection:
``` fhirpath
Patient.name.last()
```

#### tail() : collection

Returns a collection containing all but the first item in the input collection. Will return an empty collection if the input collection has no items, or only one item.

For example:
``` fhirpath
(0 | 1 | 2).tail() // 1, 2
```

#### skip(num : Integer) : collection

Returns a collection containing all but the first `num` items in the input collection. Will return an empty collection if there are no items remaining after the indicated number of items have been skipped, or if the input collection is empty. If `num` is less than or equal to zero, the input collection is simply returned.

For example:
``` fhirpath
(0 | 1 | 2).skip(1) // 1, 2
(0 | 1 | 2).skip(2) // 2
```

#### take(num : Integer) : collection

Returns a collection containing the first `num` items in the input collection, or less if there are less than `num` items. If num is less than or equal to 0, or if the input collection is empty (`{ }`), `take` returns an empty collection.

For example:
``` fhirpath
(0 | 1 | 2).take(2) // 0, 1
(0 | 1 | 2).take(5) // 0, 1, 2
```

#### intersect(other: collection) : collection

Returns the set of items that are in both collections. Duplicate items will be eliminated by this function. Order of items is not guaranteed to be preserved in the result of this function.

For example:
``` fhirpath
(1 | 2 | 3).intersect(2 | 4) // 2
```

#### exclude(other: collection) : collection

Returns the set of items that are not in the `other` collection. Duplicate items will not be eliminated by this function, and order will be preserved.

For example:
``` fhirpath
(1 | 2 | 3).exclude(2) // 1, 3
```

### Combining

#### <a name="unionother-collection"></a>union(other : collection) : collection

Merge the two collections into a single collection, eliminating any duplicate values *(items are considered duplicates if and only if the [equals](#equals) (`=`) operator returns `true`. i.e. `false` and empty both indicate that the items are not duplicates, and thus appear in the output collection)*.

There is no expectation of order in the resulting collection.

In other words, this function returns the distinct list of items from both inputs.

For example, consider two lists of integers `A: 1, 1, 2, 3` and `B: 2, 3`:
``` fhirpath
A.union( B )   // 1, 2, 3
A.union( { } ) // 1, 2, 3
```

This function can also be invoked using the [`|`](#-union-collections) operator.

e.g. `x.union(y)`{:.fhirpath} is synonymous with `x | y`{:.fhirpath}

e.g. `name.select(use.union(given))`{:.fhirpath} is the same as `name.select(use | given)`{:.fhirpath}, noting that the union function does not introduce an iteration context, in this example the select introduces the iteration context on the name elements.

<a name="fn-combine"></a>
#### combine(other : collection, [preserveOrder : Boolean]) : collection

Merge the input and other collections into a single collection without eliminating duplicate values. Combining an empty collection with a non-empty collection will return the non-empty collection.

> **Note:** The contents of this section are Standard for Trial Use (STU)
{: .stu-note }
When `preserveOrder` is `false`, or not supplied, there is no expectation of order. When `preserveOrder` is `true`, the items of the other collection are appended to the items of the input collection, preserving the order of items in both collections.
{: .stu}

For example, considering the same two lists of integers used in the union example `A: 1, 1, 2, 3` and `B: 2, 3`:
{: .stu}
```fhirpath
A.combine(B)       // 1, 1, 2, 2, 3, 3 ; order is not guaranteed to be preserved (could be in any order)
A.combine(B, true) // 1, 1, 2, 3, 2, 3 ; The order is preserved using the `preserveOrder` argument
A.combine( {} )    // 1, 1, 2, 3       ; combining an empty collection with a non-empty collection returns the non-empty collection
```
{: .stu}
Note that the duplicate `1`s are not removed from the collection using combine, where using `union` or `|` they would have been.
{: .stu}

### Conversion

FHIRPath defines both _implicit_ and _explicit_ conversion. Implicit conversions occur automatically, as opposed to explicit conversions that require the function be called explicitly. Implicit conversion is performed when an operator or function is used with a compatible type.

For example:
``` fhirpath
5 + 10.0
```

In the above expression, the addition operator expects either two Integers, or two Decimals, so implicit conversion is used to convert the integer to a decimal, resulting in decimal addition.

The following table lists the possible conversions supported, and whether the conversion is implicit or explicit:

|From\To |Boolean |Integer |Long *(STU)*{:.stu-bg} |Decimal |Quantity |String |Date |DateTime |Time |
|- |- |- |- |- |- |- |- |- | - |
|**Boolean** |N/A |Explicit | *Explicit*{:.stu-bg} |Explicit |*Explicit*{:.stu-bg} |Explicit |- |- |- |
|**Integer** |Explicit |N/A | *Implicit*{:.stu-bg} |Implicit |Implicit |Explicit |- |- |- |
|**Long** *(STU)*{:.stu-bg} |*Explicit*{:.stu-bg} |*Explicit*{:.stu-bg} | *N/A*{:.stu-bg} |*Implicit*{:.stu-bg} |*Implicit*{:.stu-bg} |*Explicit*{:.stu-bg} |*-*{:.stu-bg} |*-*{:.stu-bg} |*-*{:.stu-bg} |
|**Decimal** |Explicit |- | *-*{:.stu-bg} |N/A |Implicit |Explicit |- |- |- |
|**Quantity** |- |- | *-*{:.stu-bg} |- |N/A |Explicit |- |- |- |
|**String** |Explicit |Explicit | *Explicit*{:.stu-bg} |Explicit |Explicit |N/A |Explicit |Explicit |Explicit |
|**Date** |- |- | *-*{:.stu-bg} |- |- |Explicit |N/A |Implicit |- |
|**DateTime** |- |- | *-*{:.stu-bg} |- |- |Explicit |Explicit |N/A |- |
|**Time** |- |- | *-*{:.stu-bg} |- |- |Explicit |- |- |N/A |
{: .grid}

* Implicit - Values of the type in the From column will be implicitly converted to values of the type in the To column when necessary
* Explicit - Values of the type in the From column can be explicitly converted using a function defined in this section
* N/A - Not applicable
* \- No conversion is defined

**Note:** When an integer, long or decimal is implicitly converted to a Quantity, the resulting quantity will have the UCUM default unit ('1').

The functions in this section operate on collections with a single item. If there is more than one item, the evaluation of the expression will end and signal an error to the calling environment.

<a name="iif"></a>
#### iif(criterion: ($this) => Boolean, true-result: ($this) => collection [, otherwise-result: ($this) => collection]) : collection

> This is a [scoped function](#scoped-functions): The `criterion` argument is evaluated once (with `$this` set to the input value).<br/>
> If it returns `true`, then the `true-result` argument is evaluated (with `$this` set to the input value) and returned,<br/>
> otherwise the `otherwise-result` argument is evaluated (with `$this` set to the input value) and returned.<br/>
> *Unlike any other scoped function, the `$index` variable is not set during evaluation of the arguments, so the value of `$index` would be that of an outer context.*

The `iif` function in FHIRPath is an _immediate if_, also known as a conditional operator (such as the C programming language's `? :` operator).

Unlike most other functions it can be called with no context (using the expression's evaluation input as the context), or with a single item context.

If the input collection contains multiple items, the evaluation of the expression will end and signal an error to the calling environment.

§if1:The `criterion` expression SHALL evaluate to a Boolean, consistent with [singleton evaluation of collections](#singleton-evaluation-of-collections).§

If `criterion` evaluates to `true`, the function returns the value of the `true-result` argument, even if the input collection is empty.

If `criterion` evaluates to either `false` or an empty collection, the function returns `otherwise-result`, unless the optional `otherwise-result` is not given, in which case the function returns an empty collection.

Note that short-circuit behavior is expected in this function. 
§if2:In other words, `true-result` SHOULD only be evaluated if the `criterion` evaluates to `true`, and `otherwise-result` SHOULD only be evaluated otherwise.§
The `criterion` is always evaluated, even when the input collection is empty.
For implementations, this means delaying evaluation of the output arguments (specifically true-result and otherwise-result) to remove the chance that their evaluation throws an error and terminates the expression early.

For example:
``` fhirpath
// call with no context
iif(true, 'It is true', 'It is false') // returns 'It is true'
iif(false, 'It is true', 'It is false') // returns 'It is false'
iif({}, 'It is true', 'It is false') // returns 'It is false'

// call with input context
// (example use on an identifier to read a specific identifier and perform some checks on it - could be a form field pre-population rule)
Patient.identifier.where(system = 'http://example.org/special-id').first()
  .iif(value.startsWith('555-').not(), value, value.substring(4))
  .iif(exists(), $this, '(no special id)')

// call with empty input context (and compare to using select)
{}.iif(true, 'It is true', 'It is false')         // returns 'It is true'
{}.select(iif(true, 'It is true', 'It is false')) // empty result, the iif is never evaluated

// several ways to return the patient's birthDate, or '(unknown)' if no birthDate is present
iif(birthDate.exists(), birthDate.toString(), '(unknown)') // $this is the Patient, so need to access the birthDate element
birthDate.iif(exists(), $this.toString(), '(unknown)')     // $this is the birthDate
birthDate.iif(exists(), toString(), '(unknown)')           // most concise form of the expression

// examples showing $this context (evaluating on a Patient)
Patient.birthDate.iif(true, $this)  // returns the birthdate value, which would also return empty if there was no birthDate
iif(true, $this)                    // will return the evaluation context (e.g. the Patient)
```

Note that [singleton evaluation of collections](#singleton-evaluation-of-collections) applies to the `criterion` parameter:
``` fhirpath
iif(1, 'true', 'false')      // returns 'true' (not due to conversion to boolean)
iif(0, 'true', 'false')      // returns 'true' (0 is a non-empty single item, not a Boolean false)
iif('hi', 'true', 'false')   // returns 'true'
```

#### Boolean Conversion Functions

##### toBoolean() : Boolean

If the input collection contains a single item, this function will return a single boolean if:

* the item is a Boolean
* the item is an Integer and is equal to one of the **possible** integer **representations** of Boolean values
* the item is a Decimal that is equal to one of the **possible** decimal **representations** of Boolean values
* the item is a String that is equal to one of the **possible** string **representations** of Boolean values

If the item is not one the above types, or the item is a String, Integer, or Decimal, but is not equal to one of the possible values convertible to a Boolean, the result is empty.

The following table describes the **possible** type **representations**:

| Type | Representation | Result |
| -| - | - |
| **String** | `'true'`, `'t'`, `'yes'`, `'y'`, `'1'`, `'1.0'` | `true` |
| | `'false'`, `'f'`, `'no'`, `'n'`, `'0'`, `'0.0'` | `false` |
| **Integer** | `1` | `true` |
| | `0` | `false` |
| **Decimal** | `1.0` *(or decimal equivalent)* |`true` |
| | `0.0` *(or decimal equivalent)* | `false` |
{: .grid}

Note for the purposes of string representations, case is ignored (so that both `'T'` and `'t'` are considered `true`).

If the input collection contains multiple items, the evaluation of the expression will end and signal an error to the calling environment.

If the input collection is empty, the result is empty.

For example:
``` fhirpath
'true'.toBoolean()  // true
1.toBoolean()       // true
'hello'.toBoolean() // empty ({ }) ; not a recognized boolean representation
```

##### convertsToBoolean() : Boolean

If the input collection contains a single item, this function will return `true` if:

* the item is a Boolean
* the item is an Integer that is equal to one of the possible integer representations of Boolean values
* the item is a Decimal that is equal to one of the possible decimal representations of Boolean values
* the item is a String that is equal to one of the possible string representations of Boolean values

If the item is not one of the above types, or the item is a String, Integer, or Decimal, but is not equal to one of the possible values convertible to a Boolean, the result is `false`.

Possible values for Integer, Decimal, and String are described in the toBoolean() function.

If the input collection contains multiple items, the evaluation of the expression will end and signal an error to the calling environment.

If the input collection is empty, the result is empty.

For example:
``` fhirpath
'true'.convertsToBoolean() // true
'abc'.convertsToBoolean()  // false ; not a recognized boolean representation
```

#### Integer Conversion Functions

##### toInteger() : Integer

If the input collection contains a single item, this function will return a single integer if:

* the item is an Integer or Long
* the item is a String and is convertible to an integer
* the item is a Boolean, where `true` results in a 1 and `false` results in a 0.

If the item is not one the above types, the result is empty.

If the item is a String, but the string is not convertible to an integer (using the regex format `(\+|-)?\d+`), the result is empty.

If the input collection contains multiple items, the evaluation of the expression will end and signal an error to the calling environment.

If the input collection is empty, the result is empty.

For example:
``` fhirpath
'1'.toInteger()  // 1
'st'.toInteger() // empty ({ }) ; not convertible to an integer
42L.toInteger()  // 42 ; as an integer
```

##### convertsToInteger() : Boolean

If the input collection contains a single item, this function will return `true` if:

* the item is an Integer or Long
* the item is a String and is convertible to an Integer
* the item is a Boolean

If the item is not one of the above types, or the item is a String, but is not convertible to an Integer (using the regex format `(\+|-)?\d+`), the result is `false`.

If the input collection contains multiple items, the evaluation of the expression will end and signal an error to the calling environment.

If the input collection is empty, the result is empty.

For example:
``` fhirpath
'1'.convertsToInteger()   // true
'1.0'.convertsToInteger() // false ; decimal strings are not convertible to integer
```

##### toLong() : Long
{:.stu}
> **Note:** The contents of this section are Standard for Trial Use (STU)
{: .stu-note }

If the input collection contains a single item, this function will return a single integer if:
{:.stu}
* the item is an Integer or Long
* the item is a String and is convertible to a 64 bit integer
* the item is a Boolean, where `true` results in a 1 and `false` results in a 0.
{:.stu}

If the item is not one the above types, the result is empty.
{:.stu}

If the item is a String, but the string is not convertible to a 64 bit integer (using the regex format `(\+|-)?\d+`), the result is empty.
{:.stu}

If the input collection contains multiple items, the evaluation of the expression will end and signal an error to the calling environment.
{:.stu}

If the input collection is empty, the result is empty.
{:.stu}

For example:
{:.stu}
``` fhirpath
42.toLong()    // 42
'123'.toLong() // 123
'abc'.toLong() // empty ({ }) ; not convertible to a long
```
{:.stu}

##### convertsToLong() : Boolean
{:.stu}

If the input collection contains a single item, this function will return `true` if:
{:.stu}

* the item is an Integer or Long
* the item is a String and is convertible to a Long
* the item is a Boolean
{:.stu}

If the item is not one of the above types, or the item is a String, but is not convertible to an Integer (using the regex format `(\+|-)?\d+`), the result is `false`.
{:.stu}

If the input collection contains multiple items, the evaluation of the expression will end and signal an error to the calling environment.
{:.stu}

If the input collection is empty, the result is empty.
{:.stu}

For example:
{:.stu}
``` fhirpath
'123'.convertsToLong() // true
'abc'.convertsToLong() // false
```
{:.stu}

#### Date Conversion Functions
<a name="format-codes"></a>
##### Date/DateTime String Format Codes
{:.stu}
Parsing textual content into date/time values is a complex task, given the wide variety of formats in use in the real world. The `toDate()` and `toDateTime()` functions defined below provide a way to parse a wide variety of date/time formats into FHIRPath date and datetime types. Unfortunately, there is no single standard for date/time formats across the contexts where FHIRPath is used (e.g., most programming languages have their own date/time formats), so the this specification defines the following format codes below.
{:.stu}
Note that:
{:.stu}
* Format codes are case-sensitive.
* Any character in the format string parameter not represented by a format code is treated as a literal and must match exactly in the input string.
* Only a subset of the codes are required, implementations may vary.
{:.stu}

| Format Code	| Support	| Description |
| ----------- | ------- | ----------- |
| yy	 | optional	| 2-digit year (e.g., 80 for 1980)<br/>Implementations have discretion on how to interpret the century for 2-digit years (e.g., based on contextual knowledge).<br/>A common approach is to interpret values 00-49 as 2000-2049 and 50-99 as 1950-1999.<br/>Note that this format code is discouraged due to the ambiguity it introduces.
| yyyy | required	| 4-digit year (e.g., 2024)
| M    | optional	| 1- or 2-digit month of year (1=January, etc.)
| MM   | required	| 2-digit month of year (01=January, etc.)
| MMM  | optional	| The localized abbreviated name of the month (e.g., 'Jun' for en-US, 'juin' for fr-FR)
| MMMM | optional	| The localized full name of the month (e.g., 'June' for en-US, 'juni' for da-DK)
| d    | optional	| 1- or 2-digit day of month (1 through 31)
| dd   | required	| 2-digit day of month (01 through 31)
| h    | optional	| 1- or 2-digit hour of AM/PM (1 through 12)
| hh   | required	| 2-digit hour of AM/PM (01 through 12)
| H    | optional	| 1- or 2-digit hour of day (00 through 23)
| HH   | required	| 2-digit hour of day (00 through 23)
| m    | optional	| 1- or 2-digit minute of hour (0 through 59)
| mm   | required	| 2-digit minute of hour (00 through 59)
| s    | optional	| 1- or 2-digit second of minute (0 through 59)
| ss   | required	| 2-digit second of minute (00 through 59)
| S[+] | required	| 1-digit fraction of second (0 through 9)<br/>Note that consecutive fractional seconds are grouped together, e.g. SSS for milliseconds.<br/>§fmt1:Implementations SHALL support at least 3 digits (milliseconds); support for additional digits is optional.§
| a    | required	| 1- or 2-letter localized AM/PM specifier.<br/>e.g., en-US: A, AM, P, etc.<br/>e.g., ja-JP: 午, 午前, etc.
| z    | optional	| Time zone literal (name or id). E.g., America/Los_Angeles, Pacific Standard Time, or PST.
| Z    | required	| Time zone offset from UTC (e.g., +0200, -0500) or Z literal for UTC
{:.grid}
{:.stu}

##### toDate([format : string]) : Date

If the input collection contains a single item, this function will return a single date if:

* the item is a Date
* the item is a DateTime, in which case the year, month, and day components
are extracted directly without timezone conversion/normalization
* the item is a String and is convertible to a Date

If the item is not one of the above types, the result is empty.

If the item is a String, but the string is not convertible to a Date (using the default format `yyyy-MM-DD`{:.formatted}), the result is empty.

When the optional format parameter is provided, it is used as a [template](#format-codes) instead of the default format.
If the input is not a string, the format parameter it ignored.
{:.stu}

If the input collection contains multiple items, the evaluation of the expression will end and signal an error to the calling environment.

If the input collection is empty, the result is empty.

For example: 
```fhirpath
@2024-01-15T23:30:00-05:00.toDate() // returns @2024-01-15
'2024-01-15'.toDate() // returns @2024-01-15
'150124'.toDate('ddMMyy') // returns @2024-01-15 
'15-01-2024'.toDate('dd-MM-yyyy') // returns @2024-01-15
'12-27'.toDate('MM-yy') // returns @2027-12 (a partial date with just year and month entered)
```

##### convertsToDate([format : string]) : Boolean

If the input collection contains a single item, this function will return `true` if:

* the item is a Date
* the item is a DateTime
* the item is a String and is convertible to a Date

If the item is not one of the above types, or is not convertible to a Date (using the default format `yyyy-MM-DD`{:.formatted}), the result is `false`.

When the optional format parameter is provided, it is used as a [template](#format-codes) instead of the default format.
If the input is not a string, the format parameter it ignored.
{:.stu}

If the item contains a partial date (e.g. `'2012-01'`), the result is a partial date.

If the input collection contains multiple items, the evaluation of the expression will end and signal an error to the calling environment.

If the input collection is empty, the result is empty.

For example:
``` fhirpath
'2015-02-04'.convertsToDate() // true
'not-a-date'.convertsToDate() // false
```

#### DateTime Conversion Functions

##### toDateTime([format : string]) : DateTime

If the input collection contains a single item, this function will return a single datetime if:

* the item is a DateTime
* the item is a Date, in which case the result is a DateTime with the year, month, and day of the Date, and the time components empty (not set to zero)
* the item is a String and is convertible to a DateTime

If the item is not one of the above types, the result is empty.

If the item is a String, but the string is not convertible to a DateTime (using the default format `yyyy-MM-DDThh:mm:ss.fff(+|-)hh:mm`{:.formatted}), the result is empty.

When the optional format parameter is provided, it is used as a [template](#format-codes) instead of the default format.
If the input is not a string, the format parameter it ignored.
{:.stu}

If the item contains a partial datetime (e.g. `'2012-01-01T10:00'`), the result is a partial datetime.

If the input collection contains multiple items, the evaluation of the expression will end and signal an error to the calling environment.

If the input collection is empty, the result is empty.

For example:
``` fhirpath
'2015-02-04T14:34:28Z'.toDateTime() // @2015-02-04T14:34:28Z
'2012-01-01T10:00'.toDateTime() // @2012-01-01T10:00 ; partial datetime is preserved
```

##### convertsToDateTime([format : string]) : Boolean

If the input collection contains a single item, this function will return `true` if:

* the item is a DateTime
* the item is a Date
* the item is a String and is convertible to a DateTime

If the item is not one of the above types, or is not convertible to a DateTime (using the default format `yyyy-MM-DDThh:mm:ss.fff(+|-)hh:mm`{:.formatted}), the result is `false`.

When the optional format parameter is provided, it is used as a [template](#format-codes) instead of the default format.
If the input is not a string, the format parameter it ignored.
{:.stu}

If the input collection contains multiple items, the evaluation of the expression will end and signal an error to the calling environment.

If the input collection is empty, the result is empty.

For example:
``` fhirpath
'2015-02-04T14:34:28'.convertsToDateTime() // true
'not a datetime'.convertsToDateTime() // false
```

#### Decimal Conversion Functions

##### toDecimal() : Decimal

If the input collection contains a single item, this function will return a single decimal if:

* the item is an Integer, Long or Decimal
* the item is a String and is convertible to a Decimal
* the item is a Boolean, where `true` results in a `1.0` and `false` results in a `0.0`.

If the item is not one of the above types, the result is empty.

If the item is a String, but the string is not convertible to a Decimal (using the regex format `(\+|-)?\d+(\.\d+)?`), the result is empty.

If the input collection contains multiple items, the evaluation of the expression will end and signal an error to the calling environment.

If the input collection is empty, the result is empty.

For example:
``` fhirpath
'1.1'.toDecimal() // 1.1
'42'.toDecimal() // 42 (as a decimal value)
'42L'.toDecimal() // 42 (as a decimal value)
'st'.toDecimal() // empty ({ }) ; not convertible to a decimal
```

##### convertsToDecimal() : Boolean

If the input collection contains a single item, this function will `true` if:

* the item is an Integerm Long or Decimal
* the item is a String and is convertible to a Decimal
* the item is a Boolean

If the item is not one of the above types, or is not convertible to a Decimal (using the regex format `(\+|-)?\d+(\.\d+)?`), the result is `false`.

If the input collection contains multiple items, the evaluation of the expression will end and signal an error to the calling environment.

If the input collection is empty, the result is empty.

For example:
``` fhirpath
'1.0'.convertsToDecimal() // true
'42'.convertsToDecimal()  // true
'42L'.convertsToDecimal()  // true
'1.a'.convertsToDecimal() // false
```

#### Quantity Conversion Functions

<a name="fn-toquantity"></a>
##### toQuantity([unit : String]) : Quantity

If the input collection contains a single item, this function will return a single quantity if:

* the item is an Integer, Long or Decimal, where the resulting quantity will have the UCUM default unit (`'1'`)
* the item is a Quantity
* the item is a String and is convertible to a Quantity using the regex format:
``` regex
(?'value'(\+|-)?\d+(\.\d+)?)\s*('(?'unit'[^']+)'|(?'time'[a-zA-Z]+))?
```
As with integer, long, and decimal, where the resulting quantity has no unit, it will have the UCUM default unit (`'1'`)
* the item is a Boolean, where `true` results in the quantity `1.0 '1'`, and `false` results in the quantity `0.0 '1'`

If the item is not one of the above, the result is empty.

If the input collection contains multiple items, the evaluation of the expression will end and signal an error to the calling environment.

If the input collection is empty, the result is empty.

For example, the following are valid quantity strings used with toQuantity:
``` fhirpath
'4 days'.toQuantity()
'10 \'mm[Hg]\''.toQuantity()
```

§q4:If the `unit` argument is provided, it SHALL be the string representation of a UCUM code (or a FHIRPath calendar duration keyword).§

If the `unit` argument is provided and the input can be [**converted**](#unit-conversions), the result is the converted quantity, otherwise the result is empty.

For example:
``` fhirpath
52 'cm'.toQuantity('m') // 0.52 'm'
45.toQuantity('m')      // { } empty ; There is no conversion from the UCUM default unit to meters.
                        //             Can instead use multiplication 45 * 1 'm'
q.toQuantity('g')       // returns the value of q converted to grams according to UCUM conversion rules
24 'm'.toQuantity('kg') // empty ; there is no conversion between these units in UCUM
1 'a'.toQuantity('d')   // 365.25 'd' ; UCUM conversion for definite durations
1 'wk'.toQuantity('d')  // 7 'd' ; UCUM conversion for definite durations
42.toQuantity()         // 42 '1' ; A quantity with the UCUM default unit
```

##### Unit Conversions
Unit conversion can impact operations on quantity values including: [equality](#quantity-equality), [equivalence](#quantity-equivalence), [comparison](#comparison) and [arithmetic](#math-1).

Unit conversions between UCUM units is defined by the [\[UCUM\] specification](#UCUM) for commensurable units (i.e. between a single dimension such as length).
This is often a simple fixed conversion factor applied to convert between specific units, though may require multiple calculations.
If units are not commensurable, the result of conversion is empty (`{ }`).

> ***Most/Least Granular Unit:***<br/>
> §uc1:When an operation requires one of its arguments to be converted to the "most granular" unit, implementations SHALL select the smaller unit of measurement.§
> For example if selecting between `mm` and `cm`, then `mm` is the most granular unit.<br/>
> This can be generically evaluated by selecting the conversion factor that is less than 1 when converting from one unit to the other.
> If the conversion factor is greater than 1, then the other unit is more granular.<br/>
> e.g. for our example above, the conversion factor from `mm` to `cm` is 0.1, which is less than 1, so `mm` is the most granular unit.<br/>
> Conversely, the conversion factor from `in_i` to `cm` is 2.54, which is greater than 1, so `cm` is the most granular of those two units.<br/>
> This also applies to complex composite units such as velocities (`m/s`) or other ratios (`J/s`) where the rules of the UCUM specification define how to calculate the conversion factor.<br/>
> The "least granular" is the opposite of this.<br/>
> If the conversion factors are 1 *(the units are equal)*, then choose the unit of the operator's left argument.<br/>
> *This is required for both [quantity addition](#-addition) (most granular) and [quantity equivalence](#quantity-equivalence) (least granular)*.
>
> ***Note:*** *Implementers should refer to the [UCUM specification](https://ucum.org/ucum#baseunits) for guidance, and also to the [NLM Unit Conversion](https://ucum.nlm.nih.gov/ucum-service.html#toBaseUnits) service for more information.*
{:.stu}

There is no expectation to perform rounding on the result of applying the UCUM conversion factor to the input value, and thus the output value may have more significant figures that then input value.
Applying rounding too early can result in un-intended inaccuracies and should be explicitly applied when desired.
{:.stu}

> **Note:** Implementations are not required to support a UCUM implementation, and may return empty (`{ }`) when the units are different, or not handled. For [equivalence](#quantity-equivalence), an empty conversion result instead produces `false`.
{:.stu}

###### Time-valued unit conversions
The relationship between Calendar units and the Definite Duration UCUM Units is documented in the [Time-valued Quantities](#time-valued-quantities) section, which can be used to match a UCUM definite duration unit to/from a calendar unit, noting that the ucum `a` and `mo` units are not equal to their calendar unit counterparts.
{:.stu}

<a name="fn-toquantity-conversion-factors"></a>
FHIRPath defines the following conversion factors for calendar durations. For comparison the associated UCUM definite duration conversion factors are also included in the table:

| Calendar duration | Conversion factor | *UCUM Conversion factor* |
| - | -| - |
| `1 year` | `12 months` or `365 days` | *`1 'a'` or `365.25 'd'`* |
| `1 month` | `30 days` | *`30.4375 'd'` or `1 'mo'`* |
| `1 week` | `7 days` | *`7 'd'` or `1 'wk'`* |
| `1 day` | `24 hours` | *`1 'd'` or `24 'h'`* |
| `1 hour` | `60 minutes` | *`60 'min'`* |
| `1 minute` | `60 seconds` | *`60 's'`* |
| `1 second` | | `1 's'` |
{: .grid}

These conversion factors (apart from years/months) are the same as UCUM, so can be used interchangeably.
{:.stu}

When explicitly converting between UCUM definite durations and calendar units of differing magnitudes (e.g. days and weeks), perform the conversion within the unit system of the source,
then change the unit to the corresponding target unit:
{:.stu}
```fhirpath
7 days.toQuantity('wk')  // 7 days → 1 week → 1 'wk'
182.5 days.toQuantity('a') // 182.5 days → 0.5 year → 0.5 'a'
182.5 'd'.toQuantity('a') // 0.4996577686516085 'a' ; UCUM conversion result
```
{:.stu}
§uc2:If converting to/from years or months you SHALL use the shortest conversion chain possible (i.e. don't convert from days to months to years when you can go from direct days to years).§
§uc3:Implementers SHOULD produce a warning if this type of conversion is performed.§ 
{:.stu}

When implicitly converting quantities across UCUM definite duration and calendar units, convert the UCUM value to the UCUM unit that matches the calendar unit.
The operation that is processing the result of the implicit conversion will define the appropriate behavior (e.g. 'a' != year, 'a' ~ year, 1 day > 23 'h', 1 year + 12 'mo' )
{:.stu}

> **Note:** Unit conversion is not required for [Date/Time arithmetic](#datetime-arithmetic) except when adding/subtracting from partial dates and the 
> level of granularity is not present in the value. e.g. `@2026 + 24 months`.
{:.stu}

##### convertsToQuantity([unit : String]) : Boolean

If the input collection contains a single item, this function will return `true` if:

* the item is an Integer, Long, Decimal, or Quantity
* the item is a String that is convertible to a Quantity using the regex format:
``` regex
(?'value'(\+|-)?\d+(\.\d+)?)\s*('(?'unit'[^']+)'|(?'time'[a-zA-Z]+))?
```
* the item is a Boolean

If the input collection is empty, the result is empty.

If the item is not one of the above, the result is `false`.

If the input collection contains multiple items, the evaluation of the expression will end and signal an error to the calling environment.

§q5:If the `unit` argument is provided, it SHOULD be handled in the same way it is in the [toQuantity()](#fn-toquantity) function above, except that where an empty result would result, false is returned.§

For example:
``` fhirpath
'1 day'.convertsToQuantity() // true
10 'mg'.convertsToQuantity() // true
2 '[in_i]'.convertsToQuantity('cm') // true ; can convert inches to centimeters (commensurate units)
10 'Cel'.convertsToQuantity('[degF]') // true ; ucum "special" units are also commensurate
5 'm',convertsToQuantity('kg') // false ; the units meters and kilograms are not commensurate
```

#### String Conversion Functions

##### toString() : String

If the input collection contains a single item, this function will return a single String if:

* the item in the input collection is a String
* the item in the input collection is an Integer, Long, Decimal, Date, Time, DateTime, or Quantity the output will contain its String representation *(as shown in the table below)*
* the item is a Boolean, where `true` results in `'true'` and `false` in `'false'`.

If the item is not one of the above types, the result is empty.

The String representation uses the following formats:

|Type |Representation|Examples|
|-|-|-|
|**Boolean** |`true` or `false`| `true.toString()`{:.fhirpath} returns `true`|
|**Integer** |`(-)?#0`{:.formatted}| `42.toString()`{:.fhirpath} returns `42`|
|**Long** |`(-)?#0`{:.formatted}| `42L.toString()`{:.fhirpath} returns `42`|
|**Decimal** |`(-)?#0.0#`{:.formatted}| `3.14.toString()`{:.fhirpath} returns `3.14`|
|**Quantity** |`(-)?#0.0# (('«unit»')|(«unit»))`{:.formatted} | `(53 'km').toString()`{:.fhirpath} returns `53 'km'` *(ucum units include quotes)*<br/>`(4 days).toString()`{:.fhirpath} returns `4 days` *(calendar duration units don't include quotes)*|
|**Date** |`yyyy-MM-DD`{:.formatted}| `@2020-01-01.toString()`{:.fhirpath} returns `2020-01-01`|
|**DateTime** |`yyyy-MM-DDThh:mm:ss.fff(+|-)hh:mm`{:.formatted}| `@2020-01-01T10:00:00.000+10:00.toString()`{:.fhirpath} returns `2020-01-01T10:00:00.000+10:00` and `@2025-11-01.toString()`{:.fhirpath} returns `2025-11-01`|
|**Time** |`hh:mm:ss.fff`{:.formatted}| `@T10:30:00.000.toString()`{:.fhirpath} returns `10:30:00.000`<br/>`@T11:45.toString()`{:.fhirpath} returns `11:45`|
{:.grid}

Note that for partial dates and times, the result will only be specified to the level of precision in the value being converted.

If the input collection contains multiple items, the evaluation of the expression will end and signal an error to the calling environment.

If the input collection is empty, the result is empty.

##### convertsToString() : Boolean

If the input collection contains a single item, this function will return `true` if:

* the item is a String
* the item is an Integer, Long, Decimal, Date, Time, or DateTime
* the item is a Boolean
* the item is a Quantity

If the item is not one of the above types, the result is `false`.

If the input collection contains multiple items, the evaluation of the expression will end and signal an error to the calling environment.

If the input collection is empty, the result is empty.

For example:
``` fhirpath
@2025-01-15.convertsToString() // true
1.convertsToString() // true
Patient.name.first().convertsToString() // false ; not a type that supports toString() (assuming there was at least 1 name)
{}.convertsToString() // empty ; no input collection
```

#### Time Conversion Functions

##### toTime() : Time

If the input collection contains a single item, this function will return a single time if:

* the item is a Time
* the item is a String and is convertible to a Time

If the item is not one of the above types, the result is empty.

If the item is a String, but the string is not convertible to a Time (using the format `hh:mm:ss.fff(+|-)hh:mm`{:.formatted}), the result is empty.

If the item contains a partial time (e.g. `'10:00'`), the result is a partial time.

If the input collection contains multiple items, the evaluation of the expression will end and signal an error to the calling environment.

If the input collection is empty, the result is empty.

For example:
``` fhirpath
'14:34:28'.toTime() // 14:34:28
'10:00'.toTime() // 10:00 ; partial time is preserved
```

##### convertsToTime() : Boolean

If the input collection contains a single item, this function will return `true` if:

* the item is a Time
* the item is a String and is convertible to a Time

If the item is not one of the above types, or is not convertible to a Time (using the format `hh:mm:ss.fff(+|-)hh:mm`{:.formatted}), the result is `false`.

If the input collection contains multiple items, the evaluation of the expression will end and signal an error to the calling environment.

If the input collection is empty, the result is empty.

For example:
``` fhirpath
'14:34:28'.convertsToTime() // true
'not a time'.convertsToTime() // false
```

### String Manipulation

The functions in this section operate on collections with a single item. If there is more than one item, or an item that is not a String, the evaluation of the expression will end and signal an error to the calling environment.

To use these functions over a collection with multiple items, one may use filters like `where()` and `select()`:
``` fhirpath
Patient.name.given.select(substring(0, 1)) // returns a collection containing the first character of all the given names for a patient
```

#### indexOf(substring : String) : Integer

Returns the 0-based index of the first position `substring` is found in the input string, or -1 if it is not found.
The returned index is measured in characters (Unicode scalar values).

If `substring` is an empty string (`''`), the function returns 0.

If the input or `substring` is empty (`{ }`), the result is empty (`{ }`).

If the input collection contains multiple items, the evaluation of the expression will end and signal an error to the calling environment.

For example:
``` fhirpath
'abcdefg'.indexOf('bc') // 1
'abcdefg'.indexOf('x') // -1
'abcdefg'.indexOf('abcdefg') // 0
'a🔥b'.indexOf('🔥') // 1
'a🔥b'.indexOf('b')  // 2  (not 3, because 🔥 is 1 character)
'a\uD83D\uDD25b'.indexOf('b')  // 2  (not 3, because the surrogate pair is 1 character)
```

#### lastIndexOf(substring : String) : Integer
{:.stu}

> **Note:** The contents of this section are Standard for Trial Use (STU)
{: .stu-note }

Returns the 0-based index of the last position `substring` is found in the input string, or -1 if it is not found.
The returned index is measured in characters (Unicode scalar values).
{:.stu}

If `substring` is an empty string (`''`), the function returns the length of the string.
{:.stu}

If the input or `substring` is empty (`{ }`), the result is empty (`{ }`).
{:.stu}

If the input collection contains multiple items, the evaluation of the expression will end and signal an error to the calling environment.
{:.stu}

For example:
{:.stu}
``` fhirpath
'abcdefg'.lastIndexOf('bc') // 1
'abcdefg'.lastIndexOf('x') // -1
'abcdefg'.lastIndexOf('abcdefg') // 0
'abc abc'.lastIndexOf('a') // 4
'0123'.lastIndexOf('') // 4
'0'.lastIndexOf('') // 1
''.lastIndexOf('') // 0
```
{:.stu}

#### substring(start : Integer [, length : Integer]) : String

Returns the part of the string starting at position `start` (zero-based). If `length` is given, will return at most `length` number of characters from the input string.
Both `start` and `length` are measured in characters (Unicode scalar values).

If `start` lies outside, or equal to, the length of the string, the function returns empty (`{ }`). If there are less remaining characters in the string than indicated by `length`, the function returns just the remaining characters.

If the input or `start` is empty, the result is empty.

If an empty `length` is provided, the behavior is the same as if `length` had not been provided.

If a negative or zero `length` is provided, the function returns an empty string (`''`).

If the input collection contains multiple items, the evaluation of the expression will end and signal an error to the calling environment.

For example:
``` fhirpath
'abcdefg'.substring(3) // 'defg'
'abcdefg'.substring(1, 2) // 'bc'
'abcdefg'.substring(6, 2) // 'g'
'abcdefg'.substring(7, 1) // { } (start position is outside the string)
'abcdefg'.substring(-1, 1) // { } (start position is outside the string,
                           //     this can happen when the -1 was the result of a calculation rather than explicitly provided)
'abcdefg'.substring(3, 0) // '' (empty string)
'abcdefg'.substring(3, -1) // '' (empty string)
'abcdefg'.substring(-1, -1) // { } (start position is outside the string)
''.substring(0) // { } (start position is outside the string)
```

#### startsWith(prefix : String) : Boolean

Returns `true` when the input string starts with the given `prefix`.

If `prefix` is the empty string (`''`), the result is `true`.

If the input collection is empty, the result is empty.

If the input collection contains multiple items, the evaluation of the expression will end and signal an error to the calling environment.

For example:
``` fhirpath
'abcdefg'.startsWith('abc') // true
'abcdefg'.startsWith('xyz') // false
```

#### endsWith(suffix : String) : Boolean

Returns `true` when the input string ends with the given `suffix`.

If `suffix` is the empty string (`''`), the result is `true`.

If the input collection is empty, the result is empty.

If the input collection contains multiple items, the evaluation of the expression will end and signal an error to the calling environment.

For example:
``` fhirpath
'abcdefg'.endsWith('efg') // true
'abcdefg'.endsWith('abc') // false
```

#### contains(substring : String) : Boolean

Returns `true` when the given `substring` is a substring of the input string.

If `substring` is the empty string (`''`), the result is `true`.

If the input collection is empty, the result is empty.

If the input collection contains multiple items, the evaluation of the expression will end and signal an error to the calling environment.

For example:
``` fhirpath
'abc'.contains('b') // true
'abc'.contains('bc') // true
'abc'.contains('d') // false
```

> **Note:** The `.contains()` function described here is a string function that looks for a substring in a string. This is different than the [`contains`](#contains-containership--boolean) operator, which is a list operator that looks for an item in a list.

<a name="fn-upper"></a>
#### upper() : String

Returns the input string with all characters converted to upper case.

If the input collection is empty, the result is empty.

If the input collection contains multiple items, the evaluation of the expression will end and signal an error to the calling environment.

For example:
``` fhirpath
'abcdefg'.upper() // 'ABCDEFG'
'AbCdefg'.upper() // 'ABCDEFG'
```

<a name="fn-lower"></a>
#### lower() : String

Returns the input string with all characters converted to lower case.

If the input collection is empty, the result is empty.

If the input collection contains multiple items, the evaluation of the expression will end and signal an error to the calling environment.

For example:
``` fhirpath
'ABCDEFG'.lower() // 'abcdefg'
'aBcDEFG'.lower() // 'abcdefg'
```

#### replace(pattern : String, substitution : String) : String

Returns the input string with all instances of `pattern` replaced with `substitution`. If the substitution is the empty string (`''`), instances of `pattern` are removed from the result. If `pattern` is the empty string (`''`), every character in the input string is surrounded by the substitution, e.g. `'abc'.replace('','x')`{:.fhirpath} becomes `'xaxbxcx'`.

If the input collection, `pattern`, or `substitution` are empty, the result is empty (`{ }`).

If the input collection contains multiple items, the evaluation of the expression will end and signal an error to the calling environment.

For example:
``` fhirpath
'abcdefg'.replace('cde', '123') // 'ab123fg'
'abcdefg'.replace('cde', '') // 'abfg'
'abc'.replace('', 'x') // 'xaxbxcx'
'a\uD83D\uDD25c'.replace('', 'x') // 'xax🔥xcx' ; the unicode character should remain in tact (even if it was constructed via surrogates)
'a🔥c'.replace('', 'x') // 'xax🔥xcx'
```

<a name="fn-matches"></a>
#### matches(regex : String, [flags : String]) : Boolean

Returns `true` when the value matches the given regular expression. §mch1:Regular expressions SHOULD function consistently, regardless of any culture- and locale-specific settings in the environment, SHOULD be case-sensitive, use 'single line' mode and allow Unicode characters.§
The start/end of line markers `^`, `$` can be used to match the entire string.

If the input collection or `regex` are empty, the result is empty (`{ }`).

If the input collection contains multiple items, or the flags parameter contains invalid values, the evaluation of the expression will end and signal an error to the calling environment.

The optional `flags` parameter can be set to:
{:.stu}
* `i` to perform a case-insensitive search (otherwise is case-sensitive)
* `m` - Matches the start and end of each line using ^ and $ (multi-line)<br/>(not only begin/end of string)
{:.stu}

For example:
``` fhirpath
'http://fhir.org/guides/cqf/common/Library/FHIR-ModelInfo|4.0.1'.matches('Library') // returns true
'N8000123123'.matches('^N[0-9]{8}$') // returns false as the string is not an 8 char number (it has 10)
'N8000123123'.matches('N[0-9]{8}')   // returns true as the string has an 8 number sequence in it starting with `N`

'first line\nsecond line'.matches('^second', 'm')   // true
'first line\nsecond line'.matches('^second', '')    // false
'first line\nsecond line'.matches('^SECOND', 'im')  // true ; combine flags
```

#### matchesFull(regex : String, [flags : String]) : Boolean
{:.stu}

> **Note:** The contents of this section are Standard for Trial Use (STU)
{: .stu-note }

Returns `true` when the value completely matches the given regular expression (implying that the start/end of line markers `^`, `$` are always surrounding the regex expression provided).
{:.stu}
§mch2:Regular expressions SHOULD function consistently, regardless of any culture- and locale-specific settings in the environment, SHOULD be case-sensitive, use 'single line' mode and allow Unicode characters.§
{:.stu}

If the input collection or `regex` are empty, the result is empty (`{ }`).
{:.stu}

If the input collection contains multiple items, or the flags parameter contains invalid values, the evaluation of the expression will end and signal an error to the calling environment.
{:.stu}

The optional `flags` parameter can be set to:
{:.stu}
* `i` to perform a case-insensitive search (otherwise is case-sensitive)
* `m` - Matches the start and end of each line using ^ and $ (multi-line)<br/>(not only begin/end of string)
{:.stu}

For example:
{:.stu}
``` fhirpath
'http://fhir.org/guides/cqf/common/Library/FHIR-ModelInfo|4.0.1'.matchesFull('Library') // returns false
'N8000123123'.matchesFull('N[0-9]{8}') // returns false as the string is not an 8 char number (it has 10)
'N8000123123'.matchesFull('N[0-9]{10}') // returns true as the string has an 10 number sequence in it starting with `N`
```
{:.stu}

#### replaceMatches(regex : String, substitution: String, [flags : String]) : String

Matches the input using the regular expression in `regex` and replaces each match with the `substitution` string. The substitution may refer to identified match groups in the regular expression, as illustrated by the example below that uses named capture groups for `month`, `day`, and `year` to perform a conversion from one date format to another.

If the input collection, `regex`, or `substitution` are empty, the result is empty (`{ }`).

If the input collection contains multiple items, the evaluation of the expression will end and signal an error to the calling environment.

The optional `flags` parameter can be set to:
{:.stu}
* `i` to perform a case-insensitive search (otherwise is case-sensitive)
* `m` - Matches the start and end of each line using ^ and $ (multi-line)<br/>(not only begin/end of string)
{:.stu}

This example of `replaceMatches()` will convert a string with a date formatted as MM/dd/yy to dd-MM-yy:
``` fhirpath
'11/30/1972'.replaceMatches('\\b(?<month>\\d{1,2})/(?<day>\\d{1,2})/(?<year>\\d{2,4})\\b',
       '${day}-${month}-${year}')
```

This example locates all the instances of `aa` and surrounds them with double quotes:
``` fhirpath
'aaabaa'.replaceMatches('aa', '"aa"') // returns "aa"ab"aa"
```

> **Note:** Platforms will typically use native regular expression implementations. These are typically fairly similar, but there will always be small differences. As such, FHIRPath does not prescribe a particular dialect, but recommends the use of the [\[PCRE\]](#PCRE) flavor as the dialect most likely to be broadly supported and understood.

#### length() : Integer

Returns the number of characters (Unicode scalar values) in the input string. If the input collection is empty (`{ }`), the result is empty.

If the input collection contains multiple items, the evaluation of the expression will end and signal an error to the calling environment.

Note that `length()` counts characters (Unicode scalar values), not visual characters (grapheme clusters). For example, the string `'é'` encoded as U+0065 + U+0301 (combining form) has a length of 2, while `'é'` encoded as U+00E9 (precomposed form) has a length of 1.
{:.stu}

For example:
``` fhirpath
'abc'.length() // 3
''.length() // 0
'\u0065\u0301'.length() // 2 characters (Unicode scalar values)
'\uD83D\uDD25'.length() // 1 character (single Unicode scalar value encoded using a UTF-16 surrogate pair)
```

#### toChars() : collection

Returns the list of characters in the input string as individual single-character strings. If the input collection is empty (`{ }`), the result is empty.

If the input collection contains multiple items, the evaluation of the expression will end and signal an error to the calling environment.

``` fhirpath
'abc'.toChars()          // { 'a', 'b', 'c' }
'\u0065\u0301'.toChars() // { 'e', '\u0301' } ; the combining form of 'é' returns two characters
'a\uD83D\uDD25b'.toChars()        // { 'a', '🔥', 'b' } ; note that the surrogate pair is a single character in the output collection (don't split the surrogate pairs as separate characters)
```

### Additional String Functions
{:.stu}

> **Note:** The contents of this section are Standard for Trial Use (STU)
{: .stu-note }

#### encode(format : String) : String
{:.stu}

The encode function takes a singleton string and returns the result of encoding that string in the given format. The format parameter defines the encoding format. Available formats are:
{:.stu}

|hex |The string is encoded using hexadecimal characters (base 16) in lowercase |
|=|=|
|base64 |The string is encoded using standard base64 encoding, using A-Z, a-z, 0-9, +, and /, output padded with = |
|urlbase64 |The string is encoded using url base 64 encoding, using A-Z, a-z, 0-9, -, and _, output padded with = |
|ascii | The string has any character above character code 127 replaced with `?`. *This is a lossy encoding, and not reversible via `decode`*
{:.grid}
{:.stu}

Base64 encodings are described in [RFC4648](https://tools.ietf.org/html/rfc4648#section-4).
{:.stu}

If the input is empty, the result is empty.
{:.stu}

If no format is specified, the result is empty.
{:.stu}

For example:
{:.stu}
``` fhirpath
'test'.encode('base64') // returns 'dGVzdA==' (testEncodeBase64A)
'test'.encode('hex') // returns '74657374' (testEncodeHex)
```
{:.stu}

#### decode(format : String) : String
{:.stu}

The decode function takes a singleton encoded string and returns the result of decoding that string according to the given format. The format parameter defines the encoding format. Available formats are listed in the encode function (excluding 'ascii').
{:.stu}

If the input is empty, the result is empty.
{:.stu}

If no format is specified, the result is empty.
{:.stu}

If the content being decoded is not a valid UTF-8 value, the result will be empty.
*§dcd1:It SHOULD NOT replace with the unicode replacement character or allow ill formed UTF-8 content.§*
{:.stu}

For example:
{:.stu}
``` fhirpath
'dGVzdA=='.decode('base64') // returns 'test' (testDecodeBase64A)
'74657374'.decode('hex') // returns 'test' (testDecodeHex)
```
{:.stu}

#### escape(target : String) : String
{:.stu}

The escape function takes a singleton string and escapes it for a given target, as specified in the following table:
{:.stu}

|html |The string is escaped such that it can appear as valid HTML content (at least open bracket (`<`), ampersand (`&`), and quotes (`"`), but ideally anything with a character encoding above 127) |
|=|=|
|json |The string is escaped such that it can appear as a valid JSON string (quotes (`"`) are escaped as (`\"`)); additional escape characters are described in the [String](#string) escape section|
{:.grid}
{:.stu}

If the input is empty, the result is empty.
{:.stu}

If no target is specified, the result is empty.
{:.stu}

For example:
{:.stu}
``` fhirpath
'A&B'.escape('html') // returns 'A&amp;B' (testEscapeHtmlAmpersand)
'"1<2"'.escape('json') // returns '\"1<2\"' (testEscapeJson)
```
{:.stu}

#### unescape(target : String) : String
{:.stu}

The unescape function takes a singleton string and unescapes it for a given target. The available targets are specified in the escape function description.
{:.stu}

If the input is empty, the result is empty.
{:.stu}

If no target is specified, the result is empty.
{:.stu}

For example:
{:.stu}
``` fhirpath
'&quot;1&lt;2&quot;'.unescape('html') // returns '"1<2"' (testUnescapeHtml)
```
{:.stu}

#### trim() : String
{: .stu }

The trim function trims whitespace characters from the beginning and ending of the input string, with whitespace characters as defined in the [Whitespace](#whitespace) lexical category.
{:.stu}

If the input is empty, the result is empty.
{:.stu}

For example:
{:.stu}
``` fhirpath
' 123456 '.trim() // '123456'
'  '.trim() // ''
```
{:.stu}

#### split(separator: String) : collection
{:.stu}

The split function splits a singleton input string into a list of strings, using the given separator.
{:.stu}

If the input is empty, the result is empty.
{:.stu}

If the input string does not contain any appearances of the separator, the result is the input string.
{:.stu}

The following example illustrates the behavior of the `.split` operator:
{:.stu}

``` fhirpath
('A,B,C').split(',') // { 'A', 'B', 'C' }
('ABC').split(',') // { 'ABC' }
'A,,C'.split(',') // { 'A', '', 'C' }
```
{:.stu}

#### join([separator: String]) : String
{:.stu}

The join function takes a collection of strings and _joins_ them into a single string, optionally using the given separator.
{:.stu}

If the input is empty, the result is empty.
{:.stu}

If no separator is specified, the strings are directly concatenated.
{:.stu}

The following example illustrates the behavior of the `.join` operator:
{:.stu}

``` fhirpath
('A' | 'B' | 'C').join() // 'ABC'
('A' | 'B' | 'C').join(',') // 'A,B,C'
```
{:.stu}

### Math
{:.stu}

> **Note:** The contents of this section are Standard for Trial Use (STU)
{: .stu-note }

The functions in this section accept input collections with a single item. Unless otherwise noted, if there is more than one item, or the item is not compatible with the expected type, the evaluation of the expression will end and signal an error to the calling environment.
{:.stu}

Note also that although all functions return collections, if a given function is defined to return a single item, the return type in the description of the function is simplified to just the type of the single item, rather than the list type.
{:.stu}

The math functions in this section enable FHIRPath to be used not only for path selection, but for providing a platform-independent representation of calculation logic in artifacts such as questionnaires and documentation templates. For example:
{:.stu}

``` fhirpath
(%weight/(%height.power(2))).round(1) // note that these variables are decimal values not quantities
```
{:.stu}

This example from a questionnaire calculates the Body Mass Index (BMI) based on the responses to the weight and height elements. For more information on the use of FHIRPath in questionnaires, see the [Structured Data Capture](http://hl7.org/fhir/uv/sdc/) (SDC) implementation guide.
{:.stu}

#### abs() : Integer | Long | Decimal | Quantity
{:.stu}

Returns the absolute value of the input (in the same type). When taking the absolute value of a quantity, the unit is unchanged.
{:.stu}

Accepts input types of Integer, Long, Decimal or Quantity.
{:.stu}

If the input collection is empty, the result is empty.
{:.stu}

If the input collection contains multiple items, the evaluation of the expression will end and signal an error to the calling environment.
{:.stu}

For example:
{:.stu}
``` fhirpath
(-5).abs() // 5
(-5.5).abs() // 5.5
(-5.5 'mg').abs() // 5.5 'mg'
```
{:.stu}

<a name="fn-ceiling"></a>
#### ceiling() : Integer | Quantity
{:.stu}

Returns the first integer greater than or equal to the input.
{:.stu}

Accepts input types of Decimal or Quantity.
{:.stu}

When used with a Decimal input type, the result is an Integer.<br/>
When used with a Quantity, the result is a Quantity with the same units and the value *(Decimal)* set to the integer result calculated.
{:.stu}

If the input collection is empty, the result is empty.
{:.stu}

If the input collection contains multiple items, the evaluation of the expression will end and signal an error to the calling environment.
{:.stu}

For example:
{:.stu}
``` fhirpath
1.ceiling() // 1
1.1.ceiling() // 2
(-1.1).ceiling() // -1
```
{:.stu}

> Note: We may consider a CeilingLong() function to handle Long output types.
{: .stu-note }

#### exp() : Decimal
{:.stu}

Returns _e_ raised to the power of the input.
{:.stu}

Accepts Decimal input types. Integer and Long types are also accepted via [implicit conversion](#conversion) to Decimal. 
{:.stu}

If the input collection is empty, the result is empty.
{:.stu}

If the input collection contains multiple items, the evaluation of the expression will end and signal an error to the calling environment.
{:.stu}

For example:
{:.stu}
``` fhirpath
0.exp() // 1.0
(-0.0).exp() // 1.0
```
{:.stu}

<a name="fn-floor"></a>
#### floor() : Integer | Quantity
{:.stu}

Returns the first integer less than or equal to the input.
{:.stu}

Accepts input types of Decimal or Quantity.
{:.stu}

When used with a Decimal input type, the result is an Integer.<br/>
When used with a Quantity, the result is a Quantity with the same units and the value *(Decimal)* set to the integer result calculated.
{:.stu}

If the input collection is empty, the result is empty.
{:.stu}

If the input collection contains multiple items, the evaluation of the expression will end and signal an error to the calling environment.
{:.stu}

For example:
{:.stu}
``` fhirpath
1.floor() // 1
2.1.floor() // 2
(-2.1).floor() // -3
```
{:.stu}

> Note: We may consider a FloorLong() function to handle Long output types.
{: .stu-note }

#### ln() : Decimal
{:.stu}

Returns the natural logarithm of the input (i.e. the logarithm base _e_).
{:.stu}

Accepts Decimal input types. Integer and Long types are also accepted via [implicit conversion](#conversion) to Decimal. 
{:.stu}

If the input collection is empty, the result is empty.
{:.stu}

If the input collection contains multiple items, the evaluation of the expression will end and signal an error to the calling environment.
{:.stu}

For example:
{:.stu}
``` fhirpath
1.ln() // 0.0
1.0.ln() // 0.0
```
{:.stu}

#### log(base : Decimal) : Decimal
{:.stu}

Returns the logarithm base `base` of the input number.
{:.stu}

Accepts Decimal input types. Integer and Long types are also accepted via [implicit conversion](#conversion) to Decimal. 
{:.stu}

If the input is 0 or negative, the evaluation will end and signal an error to the calling environment.
If the base argument is 0 or negative, the evaluation will end and signal an error to the calling environment.
{:.stu}

If `base` is empty, the result is empty.
{:.stu}

If the input collection is empty, the result is empty.
{:.stu}

If the input collection contains multiple items, the evaluation of the expression will end and signal an error to the calling environment.
{:.stu}

For example:
{:.stu}
``` fhirpath
16.log(2) // 4.0
100.0.log(10.0) // 2.0
```
{:.stu}

#### power(exponent : Integer | Decimal) : Decimal
{:.stu}

Raises a number to the `exponent` power.
{:.stu}

Accepts input types of Decimal, Integer or Long.
{:.stu}

The result is always a Decimal, because raising an Integer to a negative power (such as -1) produces a Decimal result.
{:.stu}

If the power cannot be represented (such as -1 raised to the 0.5), the result is empty.
{:.stu}

If the input is empty, or exponent is empty, the result is empty.
{:.stu}

If the input collection contains multiple items, the evaluation of the expression will end and signal an error to the calling environment.
{:.stu}

For example:
{:.stu}
``` fhirpath
2.power(3) // 8 (as a decimal)
2.5.power(2) // 6.25
2.power(-1) // 0.5
(-1).power(0.5) // empty ({ })
```
{:.stu}

<a name="fn-round"></a>
#### round([precision : Integer]) : Decimal | Quantity
{:.stu}

> [Discussion on this topic](https://chat.fhir.org/#narrow/stream/179266-fhirpath/topic/round.28.29.20for.20negative.20numbers) If you have specific proposals or feedback please log a change request.
{: .stu-note }

Rounds the input to the nearest whole number using a traditional round (i.e. to the nearest whole number), meaning that a decimal value greater than or equal to 0.5 and less than 1.0 will round to 1, and a decimal value less than or equal to -0.5 and greater than -1.0 will round to -1. If specified, the precision argument determines the decimal place at which the rounding will occur. If not specified, the rounding will default to 0 decimal places.
{:.stu}

If specified, the number of digits of precision must be >= 0 or the evaluation will end and signal an error to the calling environment.
{:.stu}

Accepts input types of Decimal, or Quantity.
{:.stu}

When used with a Decimal input type, the result is an Decimal.<br/>
When used with a Quantity, the result is a Quantity with the same units.
{:.stu}

When used with Integer or Long, the arguments will be implicitly converted to Decimal before evaluation.
{:.stu}

If the input collection is empty, the result is empty.
{:.stu}

If the input collection contains multiple items, the evaluation of the expression will end and signal an error to the calling environment.
{:.stu}

For example:
{:.stu}
``` fhirpath
1.round() // 1 (as a decimal)
3.14159.round(3) // 3.142
```
{:.stu}

#### sqrt() : Decimal
{:.stu}

Returns the square root of the input number.
{:.stu}

Accepts Decimal input types. Integer and Long types are also accepted via [implicit conversion](#conversion) to Decimal. 
{:.stu}

If the square root cannot be represented (such as the square root of -1), the result is empty.
{:.stu}

If the input collection is empty, the result is empty.
{:.stu}

If the input collection contains multiple items, the evaluation of the expression will end and signal an error to the calling environment.
{:.stu}

Note that this function is equivalent to raising a number of the power of 0.5 using the power() function.
{:.stu}

For example:
{:.stu}
``` fhirpath
81.sqrt() // 9.0
(-1).sqrt() // empty
```
{:.stu}

<a name="fn-truncate"></a>
#### truncate() : Integer | Quantity
{:.stu}

Returns the integer portion of the input.
{:.stu}

Accepts input types of Decimal or Quantity.
{:.stu}

When used with a Decimal input type, the result is an Integer.<br/>
When used with a Quantity, the result is a Quantity with the same units and the value *(Decimal)* set to the integer result calculated.
{:.stu}

If the input collection is empty, the result is empty.
{:.stu}

If the input collection contains multiple items, the evaluation of the expression will end and signal an error to the calling environment.
{:.stu}

For example:
{:.stu}
``` fhirpath
101.truncate() // 101
1.00000001.truncate() // 1
(-1.56).truncate() // -1
```
{:.stu}

> Note: We may consider a TruncateLong() function to handle Long output types.
{: .stu-note }


### Tree navigation

#### children() : collection

Returns a collection with all immediate child nodes of all items in the input collection. Note that the ordering of the children is undefined and using functions like `first()` on the result may return different results on different platforms.

The following example returns all the children of the input collection:
``` fhirpath
Patient.gender.children() // the FHIR patient gender datatype is a FHIR code, which could have value, id and potentially extensions returned.
```

#### descendants() : collection

Returns a collection with all descendant nodes of all items in the input collection. The result does not include the items in the input collection themselves. This function is a shorthand for `repeat(children())`. Note that the ordering of the children is undefined and using functions like `first()` on the result may return different results on different platforms.

The following example returns all coded answers anywhere in a QuestionnaireResponse:
``` fhirpath
QuestionnaireResponse.descendants().answer.value.ofType(Coding)
```

> **Note:** Many of these functions will result in a set of items of different underlying types. It may be necessary to use [`ofType()`](#fn-oftype) as described in the previous section to maintain type safety. See [Type safety and strict evaluation](#type-safety-and-strict-evaluation) for more information about type safe use of FHIRPath expressions.

### Utility functions

<a name="fn-trace"></a>
#### trace(name : String [, projection: ($this, $index) => any]) : collection

> This is a [scoped function](#scoped-functions): If no `projection` argument is provided, the input collection is logged without the need for scoping. If the `projection` argument is provided, it is evaluated for each item (setting `$this` and `$index` before each iteration) and the result logged. The input collection is returned as the result of the function.<br/>
> The `name` parameter is evaluated before the function is executed and is not re-evaluated for each iteration of the projection.

Adds a String representation of the input collection to the diagnostic log, using the `name` argument as the name in the log. §trc1:This log SHOULD be made available to the user in some appropriate fashion.§ Does not change the input, so returns the input collection as output.

If the `projection` argument is used, the trace would log the result of evaluating the project expression on the input, but still return the input to the trace function unchanged.

The following example traces only the id elements of the result of the where:
``` fhirpath
contained.where(criteria).trace('unmatched', id).empty()
```

<a name="fn-pathname"></a>
#### pathname([short : Boolean]) : collection
{:.stu}
<!-- FHIR-45314 -->

> **Note:** The contents of this section are Standard for Trial Use (STU)
{: .stu-note }

Returns the direct path of each item of the input collection within the input resource (`%rootResource` in FHIR implementations),
using only element names and indexers. *Such that if you used that result on the input resource, you would get that node, and only that node.*
{:.stu}

If an item in the input collection was derived from computation (e.g. via `substring(..)`, `&`, or mathematical operations) rather than navigation it is excluded from the result.
Items that are outside the input resource, such as those navigated to via FHIR's resolve() are also excluded from the result, however if resolve()
references a resource contained within the input Resource then it is included (such as with FHIR bundles, or contained resources).
{:.stu}

The optional `short` parameter permits excluding array indexers if an element is known to not be an array, either in the model, or in the specific instance at runtime.
{:.stu}

If the input collection is empty `({ })`, the result is empty.
{:.stu}

This function could be used to populate fields in a FHIR OperationOutcome.issue.expression field, or assist in debugging complex expressions using it in conjunction with trace.
{:.stu}

For example, validating a FHIR QuestionnaireResponse (against a Questionnaire) could use the following expression to calculate the location of an invalid answer:
{:.stu}
``` fhirpath
item.item.item.where(linkId = 'i508').item.where(linkId='i534').answer.value.pathname()
```
{:.stu}

would return the following string *(which is also a valid fhirpath expression)* if only 1 node was at that location in the input QuestionnaireResponse:
{:.stu}

``` fhirpath
'QuestionnaireResponse.item[2].item[8].item[1].item[1].answer[0].value[0]'
```
{:.stu}

Another example could be the FHIR Observation invariant `obs-7` that roughly checks if components are duplicating codings captured at the top level:
*(not an exact copy of the invariant, but a part of it)*
{:.stu}
``` fhirpath
// trace out the pathname of the components that have duplicated codings
component.code.where(coding.intersect(%resource.code.coding).trace('component', pathname()).exists()).empty()
```
{:.stu}
would return the following strings: (simplifying finding the specific component(s) that were duplicated)
{:.stu}
``` fhirpath
'Observation.component[0].code[0].coding[0]'
'Observation.component[23].code[0].coding[0]'
```
{:.stu}

#### Current date and time functions

The following functions return the current date and time. The timestamp that these functions use is an implementation decision, and §n1:implementations SHOULD consider providing options appropriate for their environment.§ In the simplest case, the local server time is used as the timestamp for these function.

§n2:To ensure deterministic evaluation, these operators SHOULD return the same value regardless of how many times they are evaluated within any given expression (i.e. now() SHOULD always return the same DateTime in a given expression, timeOfDay() SHOULD always return the same Time in a given expression, and today() SHOULD always return the same Date in a given expression.)§

##### now() : DateTime

Returns the current date and time, including timezone offset.

The following example demonstrates that `now()` returns a full DateTime including timezone:
``` fhirpath
now() // Could return values like @2026-02-25T14:31:39.096+11:00, @2026-02-25T03:31:44.306994+00:00, @2026-02-25T03:31:41.197Z
```

##### timeOfDay() : Time

Returns the current time.

For example:
``` fhirpath
timeOfDay() // @T14:34:17.923 ; only the time portion of the result of now(). Equivalent to now().timeOf().
timeOfDay() = timeOfDay() // true ; timeOfDay is deterministic within a single expression
```

##### today() : Date

Returns the current date.

The following example demonstrates that `today()` returns a date in YYYY-MM-DD format:
``` fhirpath
today() // @2026-02-25 ; equivalent to now().toDate()
```

<a name="definevariable"></a>
#### defineVariable(name: String [, projection: collection])
{:.stu}
> **Note:** The contents of this section are Standard for Trial Use (STU)
{: .stu-note }

Defines a variable named `name` that is accessible in subsequent expressions on the output collection and has the value of `projection` if present, otherwise the value of the input collection. In either case the function does not change the input and the output is the same as the input collection.
{:.stu}

> **Note:** This is not a scoped function, it does not change any variables such as `$this` or `$index`.<br/>
>
> This function is the only function that changes the state of the context for processing on the output collection.<br/>
> Whereas [scoped functions](#scoped-functions) only impact the context while evaluating the function, and it's parameters, and the context is restored to the same as before the function was called.
{:.stu}

If the name already exists in the current expression scope, the evaluation will end and signal an error to the calling environment.
{:.stu}

For example:
{:.stu}
``` fhirpath
group.select(
  defineVariable('grp')
  .select(
    element.select(
      defineVariable('src')
      .target.select(
        %grp.source & '#' & %src.code
        & ' ' & equivalence & ' '
        & %grp.target & '#' & code
      )
    )
  )
)
```
{:.stu}

> **Note:** this could be implemented using expression scoping on the variable stack and after expression completion the temporary variable would be popped off the stack.
{:.stu}

#### lowBoundary([precision: Integer]): Decimal | Quantity | Date | DateTime | Time
{:.stu}

The least possible value of the input to the specified precision.
{:.stu}

The function can be used with Decimal, Quantity, Date, DateTime, and Time values, and returns the same type as the value in the input collection. For a Quantity, the function applies the Decimal boundary operation to the numeric value and returns a Quantity with the resulting value and the original unit.
{:.stu}

Integer or Long inputs are implicitly converted to Decimal with zero fractional digits of input precision, and the result is Decimal.
{:.stu}

If no precision is specified, the greatest precision of the type of the input value is used (i.e. at least 8 for Decimal, 4 for Date, at least 17 for DateTime, and at least 9 for Time).
{:.stu}

If the precision is greater than the maximum possible precision of the implementation, the result is empty *(CQL returns null)*.
{:.stu}

If the input collection is empty, the result is empty.
{:.stu}

If the input collection contains multiple items, the evaluation of the expression will end and signal an error to the calling environment.
{:.stu}

For example:
{:.stu}
``` fhirpath
1.587.lowBoundary() // 1.58650000
1.587.lowBoundary(6) // 1.586500
1.587.lowBoundary(2) // 1.58
1.587.lowBoundary(0) // 1
1.lowBoundary(2) // 0.50
2.1 'cm'.lowBoundary(2) // 2.05 'cm'
(-1.587).lowBoundary() // -1.58750000
(-1.587).lowBoundary(6) // -1.587500
(-1.587).lowBoundary(2) // -1.59
(-1.587).lowBoundary(0) // -2
@2014.lowBoundary(6) // @2014-01
@2014-01-01T08.lowBoundary(17) // @2014-01-01T08:00:00.000
@T10:30.lowBoundary(9) // @T10:30:00.000
```
{:.stu}

#### highBoundary([precision: Integer]): Decimal | Quantity | Date | DateTime | Time
{:.stu}

The greatest possible value of the input to the specified precision.
{:.stu}

The function can be used with Decimal, Quantity, Date, DateTime, and Time values, and returns the same type as the value in the input collection. For a Quantity, the function applies the Decimal boundary operation to the numeric value and returns a Quantity with the resulting value and the original unit.
{:.stu}

Integer or Long inputs are implicitly converted to Decimal with zero fractional digits of input precision, and the result is Decimal.
{:.stu}

If no precision is specified, the greatest precision of the type of the input value is used (i.e. at least 8 for Decimal, 4 for Date, at least 17 for DateTime, and at least 9 for Time).
{:.stu}

If the precision is greater than the maximum possible precision of the implementation, the result is empty *(CQL returns null)*.
{:.stu}

If the input collection is empty, the result is empty.
{:.stu}

If the input collection contains multiple items, the evaluation of the expression will end and signal an error to the calling environment.
{:.stu}

For example:
{:.stu}
``` fhirpath
1.587.highBoundary() // 1.58750000
1.587.highBoundary(6) // 1.587500
1.587.highBoundary(2) // 1.59
1.587.highBoundary(0) // 2
1.highBoundary(2) // 1.50
2.1 'cm'.highBoundary(2) // 2.15 'cm'
(-1.587).highBoundary() // -1.58650000
(-1.587).highBoundary(6) // -1.586500
(-1.587).highBoundary(2) // -1.58
(-1.587).highBoundary(0) // -1
@2014.highBoundary(6) // @2014-12
@2014-01-01T08.highBoundary(17) // @2014-01-01T08:59:59.999
@T10:30.highBoundary(9) // @T10:30:59.999
```
{:.stu}

#### precision() : Integer
{:.stu}

If the input collection contains a single item, this function will return the number of digits of precision.
{:.stu}

The function can be used with Decimal, Quantity, Date, DateTime, and Time values. For a Quantity, it returns the number of digits of precision in the numeric value, including trailing fractional zeroes.
{:.stu}

If the input collection is empty, the result is empty.
{:.stu}

If the input collection contains multiple items, the evaluation of the expression will end and signal an error to the calling environment.
{:.stu}

For Decimal values, the function returns the number of digits of precision after the decimal place in the input value.
{:.stu}

For example with decimal values:
{:.stu}
``` fhirpath
1.58700.precision() // 5
100.precision() // 0 ; no digits after the decimal place (integer implicitly converted to a decimal for processing)
(1.00 'mg').precision() // 2
```
{:.stu}

For Date and DateTime values, the function returns the number of digits of precision in the input value:
{:.stu}

``` fhirpath
@2014.precision() // 4
@2014-01-05T10:30:00.000.precision() // 17
@T10:30.precision() // 4
@T10:30:00.000.precision() // 9
```
{:.stu}

#### Extract Date/DateTime/Time components
{:.stu}

##### yearOf(): Integer
{:.stu}
If the input collection contains a single Date or DateTime, this function will return the year component.
{:.stu}

If the input collection is empty, or the value is not a Date or DateTime, the result is empty.
{:.stu}

If the input collection contains multiple items, the evaluation of the expression will end and signal an error to the calling environment.
{:.stu}

For example:
{:.stu}
``` fhirpath
@2014-01-05T10:30:00.000.yearOf() // 2014
```
{:.stu}

##### monthOf(): Integer
{:.stu}

If the input collection contains a single Date or DateTime, this function will return the month component.
{:.stu}

If the input collection is empty, the month is not present in the value, or the value is not a Date or DateTime, the result is empty.
{:.stu}

If the input collection contains multiple items, the evaluation of the expression will end and signal an error to the calling environment.
{:.stu}

For example:
{:.stu}
``` fhirpath
@2014-01-05T10:30:00.000.monthOf() // 1
```
{:.stu}

If the component isn't present in the value, then the result is empty:
{:.stu}
``` fhirpath
@2012.monthOf() // {} an empty collection
```
{:.stu}

##### dayOf(): Integer
{:.stu}

If the input collection contains a single Date or DateTime, this function will return the day component.
{:.stu}

If the input collection is empty, the day is not present in the value, or the value is not a Date or DateTime, the result is empty.
{:.stu}

If the input collection contains multiple items, the evaluation of the expression will end and signal an error to the calling environment.
{:.stu}

For example:
{:.stu}
``` fhirpath
@2014-01-05T10:30:00.000.dayOf() // 5
```
{:.stu}

##### hourOf(): Integer
{:.stu}

If the input collection contains a single DateTime or Time, this function will return the hour component.
{:.stu}

If the input collection is empty, the hour is not present in the value, or the value is not a DateTime or Time, the result is empty.
{:.stu}

If the input collection contains multiple items, the evaluation of the expression will end and signal an error to the calling environment.
{:.stu}

For example:
{:.stu}
``` fhirpath
@2012-01-01T03:30:40.002-07:00.hourOf() // 3
@2012-01-01T16:30:40.002-07:00.hourOf() // 16
```
{:.stu}

##### minuteOf(): Integer
{:.stu}

If the input collection contains a single DateTime or Time, this function will return the minute component.
{:.stu}

If the input collection is empty, the minute is not present in the value, or the value is not a DateTime or Time, the result is empty.
{:.stu}

If the input collection contains multiple items, the evaluation of the expression will end and signal an error to the calling environment.
{:.stu}

For example:
{:.stu}
``` fhirpath
@2012-01-01T12:30:40.002-07:00.minuteOf() // 30
```
{:.stu}

##### secondOf(): Integer
{:.stu}

If the input collection contains a single DateTime or Time, this function will return the second component.
{:.stu}

If the input collection is empty, the second is not present in the value, or the value is not a DateTime or Time, the result is empty.
{:.stu}

If the input collection contains multiple items, the evaluation of the expression will end and signal an error to the calling environment.
{:.stu}

For example:
{:.stu}
``` fhirpath
@2012-01-01T12:30:40.002-07:00.secondOf() // 40
```
{:.stu}

##### millisecondOf(): Integer
{:.stu}

If the input collection contains a single DateTime or Time, this function will return the millisecond component.
{:.stu}

If the input collection is empty, the millisecond is not present in the value, or the value is not a DateTime or Time, the result is empty.
{:.stu}

If the input collection contains multiple items, the evaluation of the expression will end and signal an error to the calling environment.
{:.stu}

For example:
{:.stu}
``` fhirpath
@2012-01-01T12:30:00.002-07:00.millisecondOf() // 2
```
{:.stu}

##### timezoneOffsetOf(): Decimal
{:.stu}

If the input collection contains a single DateTime, this function will return the timezone offset component. It is expressed as the number of hours difference from UTC, with fractional hours expressed as decimal values (e.g. -7.5 for UTC-7:30).
{:.stu}

If the input collection is empty, the timezone offset is not present in the value, or the value is not a DateTime, the result is empty.
{:.stu}

If the input collection contains multiple items, the evaluation of the expression will end and signal an error to the calling environment.
{:.stu}

For example:
{:.stu}
``` fhirpath
@2012-01-01T12:30:00.000-07:00.timezoneOffsetOf() // -7.0
@2012-01-01T12:30:00.000+08:45.timezoneOffsetOf() // 8.75 Eucla, Western Australia
```
{:.stu}

##### dateOf(): Date
{:.stu}

If the input collection contains a single Date or DateTime, this function will return the date component (up to the precision present in the input value).
{:.stu}

If the input collection is empty, or the value is not a Date or DateTime, the result is empty.
{:.stu}

If the input collection contains multiple items, the evaluation of the expression will end and signal an error to the calling environment.
{:.stu}

For example:
{:.stu}
``` fhirpath
@2012-01-01T12:30:00.000-07:00.dateOf() // @2012-01-01
```
{:.stu}

##### timeOf(): Time
{:.stu}

If the input collection contains a single DateTime, this function will return the time component.
{:.stu}

If the input collection is empty, the time is not present in the value, or the value is not a DateTime, the result is empty.
{:.stu}

If the input collection contains multiple items, the evaluation of the expression will end and signal an error to the calling environment.
{:.stu}

For example:
{:.stu}
``` fhirpath
@2012-01-01T12:30:00.000-07:00.timeOf() // @T12:30:00.000
```
{:.stu}

### Date and Time Interval Functions
{:.stu}

> **Note:** The contents of this section are Standard for Trial Use (STU)
{: .stu-note }

If there is more than one input item, or an incompatible item, the evaluation of the expression will end and signal an error to the calling environment.
{:.stu}

#### duration(value: Date | DateTime | Time, precision: String): Integer
{:.stu}

Returns the number of whole calendar periods at the specified `precision` between the given input value and the `value` argument. If the input value is after the `value` argument, the result is negative. The result of this operation is always an integer; any fractional periods are dropped.
{:.stu}

For input and value types of `date` values, the `precision` argument must be one of: `year`, `month`, `week`, or `day`.
{:.stu}

For input and value types of `datetime` values, the `precision` argument must be one of: `year`, `month`, `week`, `day`, `hour`, `minute`, `second`, or `millisecond`.
{:.stu}

For input and value types of `time` values, the `precision` argument must be one of: `hour`, `minute`, `second`, or `millisecond`.
{:.stu}

If the input value or `value` argument are of less precision than the specified `precision`, the result is empty.
{:.stu}

§dur1:When computing the duration between DateTime values with different timezone offsets, implementations SHOULD normalize the timezone when a `precision` of `hour`, `minute`, `second`, or `millisecond` is requested.§
{:.stu}

If either the input or `value` argument is empty, the result is empty.
{:.stu}

The following examples illustrate the behavior of the duration function:
{:.stu}
```fhirpath
@2025-01-02.duration(@2025-01-07, 'week') // 0 ; hasn't passed 7 days duration
@2025-01-01.duration(@2025-09-01, 'year') // 0 ; baby is 9 months old
@2024-12-01.duration(@2025-09-01, 'year') // 0 ; baby is 10 months old
```
{:.stu}


#### difference(value: Date | DateTime | Time, precision: String): Integer
{:.stu}

Returns the number of boundaries crossed for the specified `precision` between the input value and the `value` arguments. If the input value is after the `value` argument, the result is negative. The result of this operation is always an integer; any fractional boundaries are dropped.
{:.stu}

For input and value types of `date` values, the `precision` argument must be one of: `year`, `month`, `week`, or `day`.
{:.stu}

For input and value types of `datetime` values, the `precision` argument must be one of: `year`, `month`, `week`, `day`, `hour`, `minute`, `second`, or `millisecond`.
{:.stu}

For input and value types of `time` values, the `precision` argument must be one of: `hour`, `minute`, `second`, or `millisecond`.
{:.stu}

If the input value or `value` argument are of less precision than the specified `precision`, the result is empty.
{:.stu}

For calculations involving weeks, Sunday is considered to be the first day of the week for the purposes of determining the number of boundaries crossed.
{:.stu}

§df1:When computing the difference between `datetime` values with different timezone offsets, implementations SHOULD normalize the timezone when a `precision` of `hour`, `minute`, `second`, or `millisecond` is requested.§
{:.stu}

If either the input or `value` argument is empty, the result is empty.
{:.stu}

The following examples illustrate the behavior of the difference function:
{:.stu}
```fhirpath
@2025-01-02.difference(@2025-01-07, 'week') // 1 ; crossed a week boundary (Sunday)
@2025-01-01.difference(@2025-09-01, 'year') // 0 ; baby is 9 months old, but born this year
@2024-12-01.difference(@2025-09-01, 'year') // 1 ; baby is 10 months old, but born last year
```
{:.stu}


## Operations

Operators are allowed to be used between any kind of path expressions (e.g. expr op expr). Like functions, operators will generally propagate an empty collection in any of their operands.
This is true even when comparing two empty collections using the equality operators.

For example:
``` fhirpath
{} = {}
true > {}
{} != 'dummy'
```
all result in `{}`.

### Equality

#### <a name="equals"></a>= (Equals)

Returns `true` if the left collection is equal to the right collection:

As noted above, if either operand is an empty collection, the result is an empty collection. Otherwise:

If both operands are collections with a single item, they must be of the same type (or be [implicitly convertible](#conversion) to the same type), and:

* For primitives:
  * `String`: comparison is based on Unicode values
  * `Integer`: values must be exactly equal
  * `Long`: values must be exactly equal
  * `Decimal`: values must be equal, trailing zeroes after the decimal are ignored
  * `Boolean`: values must be the same
  * `Date`: must be exactly the same<br/>*(see [Date/Time Equality](#datetime-equality) for more details)*
  * `DateTime`: must be exactly the same, respecting the timezone offset (though +00:00 = -00:00 = Z)<br/>*(see [Date/Time Equality](#datetime-equality) for more details)*
  * `Time`: must be exactly the same<br/>*(see [Date/Time Equality](#datetime-equality) for more details)*
* `Quantity`: value comparison based on decimal as described above, after [conversion](#unit-conversions) to a common unit if required.<br/>*(See [Quantity Equality](#quantity-equality) for more details)*
* For complex types, equality requires all child elements to be equal, recursively.

If both operands are collections with multiple items, check the equality of each pair of items in order:

* if the result is `false` for any pair, returns `false`
* if the result is `true` for all pairs, returns `true`
* otherwise returns empty ( `{ }` )

Note that this implies that if the collections have a different number of items to compare, the result will be `false`.

Typically, this operator is used with single fixed values as operands. This means that `Patient.telecom.system = 'phone'`{:.fhirpath} will result in an error if there is more than one `telecom` with a `use`. Typically, you'd want `Patient.telecom.where(system = 'phone')`{:.fhirpath}

If one or both of the operands is the empty collection, this operation returns an empty collection.

For example:
```fhirpath
1.10 = 1.1 // true ; zeros after the decimal place are ignored
1.2 / 1.8 = 0.67 // false ; division results in a decimal with recurring digits, 
                 //        these are not equal to 0.67 
0.0 = 0 // true ; zeros after the decimal place are ignored
{} = {} // empty ; as explicitly defined above
name = name // true ; its the same object, and thus will have all the same properties recursively
(1 | 2 | 3) = (3 | 2 | 1) // false ; all items are in both collections, but order doesn't match
(1 | 2 | 3) = (1 | 2 | 3) // true ; all items are in both collections, and order matches
'a' = 'A' // false ; case is NOT ignored for equality
23 = 23 '1' // true ; integer implicitly converts to Quantity with unit '1', thus compares exactly here
```
***Note:** The collection examples above are illustrative only. The `|` operator does not guarantee the order of its result, so engines may legitimately return these items in a different order. In practice, many engines preserve the construction order for simple collections like these.*

##### Quantity Equality

When comparing quantities for equality, the dimensions of each quantity must be the same, but not necessarily the same unit.
This is referred to as the units being commensurable in UCUM.
For example, units of `'cm'` and `'m'` can be compared, but units of `'cm2'` and `'cm'` cannot.

When the units are different the quantity values must be [**converted**](#unit-conversions) to the same unit, or a common unit before comparison.
Any commensurable target unit may be chosen, since trailing zeroes after the decimal point are ignored when comparing the resulting decimal values *(see [Quantity Equivalence](#quantity-equivalence) for the contrasting rule, where the choice of unit affects the rounded comparison)*.
If this process returns empty (e.g. because the units are not valid, or not commensurable), then the result of the equality comparison is empty (`{ }`).

§q6:Implementations are not required to fully support operations on units, but they SHALL at least respect units, recognizing when units differ.§

Attempting to operate on quantities with invalid units will result in empty (`{ }`).

Once the units are the same, the values can be compared using simple decimal comparison, ignoring trailing zeroes after the decimal point (as described above).

For example:
```fhirpath
1 'cm' = 10.0 'mm' // true ; UCUM conversion gives the same decimal value (ignoring trailing zeros after the decimal place)
1 'cm' = 1 'm' // false ; convert to a common unit (1 'm' → 100 'cm'); 1 ≠ 100
1 'cm' = 1 's' // empty ({ }) ; different dimensions are not commensurable
23 'Cel' = 73.4 '[degF]' // true ; comparison of ucum "special" units on non-ratio scales
```

When comparing time-valued quantities the quantity values must be converted to the least granular unit
*(without changing code system)*, then compare as described above with other quantities.<br/>
This is particularly important when the time-valued quantities involve calendar units to ensure consistency across implementations,
as calendar conversions are not transitive 
(e.g. 1 year ~ 12 months and 1 month = 30 days, but 1 year ≠ 360 days).
{:.stu}

As noted in the [Time-valued Quantities](#time-valued-quantities) section, years and months are not equal across UCUM definite durations and calendar units. Hence when the arguments are a mix of these, the result is empty as they are not comparable.<br/>
Note that explicit conversion using [`toQuantity()`](#fn-toquantity) will change code-systems to intentionally perform this equality.
{:.stu}

For example:
{:.stu}
``` fhirpath
1 'h' = 3600 's'  // true ; ucum conversion
1 hour = 3600 's' // true ; 3600 's' → 1 'h' (ucum), and 1 'h' = 1 hour (calendar)
1 year = 1 'a'    // empty ({ }) ; calendar/UCUM year is not comparable
1 year.toQuantity('a') = 1 'a' // true ; intentionally converting to UCUM units
1 year = 12 months // true ; calendar unit conversion
1 year = 12 'mo' // empty ({ }) ; 12 'mo' → 1 'a'; calendar/UCUM year is not comparable
1 week = 1 'wk'  // true ; week and 'wk' are equal by definition
1 second = 1 's' // true ; second and 's' are equal by definition
```
{:.stu}

Where the time-valued quantities are being compared, convert to the least granular unit (the one appearing higher in the [time-valued unit conversion table](#fn-toquantity-conversion-factors), matching across code systems):
{:.stu}
``` fhirpath
7 days = 1 'wk'  // true ; convert to week: 7 days → 1 week (calendar), and 1 week = 1 'wk'
1 week = 7 'd'   // true ; convert to 'wk': 7 'd' → 1 'wk' (UCUM); 1 'wk' = 1 week
```
{:.stu}

##### Date/Time Equality

For `Date`, `DateTime` and `Time` equality, the comparison is performed by considering each precision in order, beginning with years (or hours for time values), and respecting timezone offsets. If the values are the same, comparison proceeds to the next precision; if the values are different, the comparison stops and the result is `false`. If one input has a value for the precision and the other does not, the comparison stops and the result is empty (`{ }`); if neither input has a value for the precision, or the last precision has been reached, the comparison stops and the result is `true`. For the purposes of comparison, seconds and milliseconds are considered a single precision using a decimal, with decimal equality semantics.

For example:
``` fhirpath
@2012 = @2012 // true
@2012 = @2013 // false
@2012-01 = @2012 // empty ({ }) ; different date precision
@2012-01-01T10:30 = @2012-01-01T10:30 // true
@2012-01-01T10:30 = @2012-01-01T10:31 // false
@2012-01-01T10:30:31 = @2012-01-01T10:30 // empty ({ }) ; different datetime precision
@2012-01-01T10:30:31.0 = @2012-01-01T10:30:31 // true
@2012-01-01T10:30:31.1 = @2012-01-01T10:30:31 // false
```


For `DateTime` values that do not have a timezone offsets, whether or not to provide a default timezone offset is a policy decision. In the simplest case, no default timezone offset is provided, but some implementations may use the client's or the evaluating system's timezone offset.

To support comparison of DateTime values, either both values have no timezone offset specified, or both values are converted to a common timezone offset. The timezone offset to use is an implementation decision. In the simplest case, it's the timezone offset of the local server.

The following examples illustrate expected behavior:
``` fhirpath
@2017-11-05T01:30:00.0-04:00 > @2017-11-05T01:15:00.0-05:00 // false
@2017-11-05T01:30:00.0-04:00 < @2017-11-05T01:15:00.0-05:00 // true
@2017-11-05T01:30:00.0-04:00 = @2017-11-05T01:15:00.0-05:00 // false
@2017-11-05T01:30:00.0-04:00 = @2017-11-05T00:30:00.0-05:00 // true
```

Additional functions to support more sophisticated timezone offset comparison (such as .toUTC()) may be defined in a future version.

#### ~ (Equivalent)

Returns `true` if the collections are the same. In particular, comparing empty collections for equivalence `{ } ~ { }`{:.fhirpath} will result in `true`.

The equivalence operator returns either `true` or `false`, never an empty collection.

If both operands are collections with a single item, they must be of the same type (or [implicitly convertible](#conversion) to the same type), and:

* For primitives
  * `String`: the strings must be the same, ignoring case and locale, and normalizing whitespace.<br/>*(see [String Equivalence](#string-equivalence) for more details)*
  * `Integer`: exactly equal
  * `Long`: exactly equal
  * `Decimal`: values must be equal, comparison is done on values rounded to the precision of the least precise operand. Trailing zeroes after the decimal are ignored in determining precision.
  * `Date`, `DateTime` and `Time`: values must be equal, except that if the input values have different levels of precision, the comparison returns `false`, not empty (`{ }`).<br/>*(see [Date/Time Equivalence](#datetime-equivalence) for more details)*
  * `Boolean`: the values must be the same
* `Quantity`: value comparison based on decimal as described above, after [**conversion**](#unit-conversions) to the least granular unit before comparison.<br/>*(See [Quantity Equivalence](#quantity-equivalence) for more details)*
* For complex types, equivalence requires all child elements to be equivalent, recursively.

If both operands are collections with multiple items:

* Each item must be equivalent
* Comparison is not order dependent

Note that this implies that if the collections have a different number of items to compare, or if one input is a value and the other is empty (`{ }`), the result will be `false`.

For example:
```fhirpath
1.10 ~ 1.1  // true ; round to 1 decimal place, then compare values
1.2 / 1.8 ~ 0.67 // true ; division results in a decimal with recurring digits, 
                 //        equivalence rounds to the 2 digits and then compares the values
0.0 ~ 0     // true ; implicit conversion to decimals, then rounding to the least precise operand (0) results in both sides being 0
{} ~ {}     // true ; as explicitly defined above
name ~ name // true ; its the same object, and thus will have all the same properties recursively
(1 | 2 | 3) ~ (3 | 2 | 1) // true ; all items are in both collections irrespective of order
'a' ~ 'A'   // true ; case is ignored for equivalence
23 ~ 23 '1' // true ; integer implicitly converts to Quantity with unit '1', thus compares exactly here
```
***Note:** The collection examples above are illustrative only. The `|` operator does not guarantee the order of its result, so engines may legitimately return these items in a different order. In practice, many engines preserve the construction order for simple collections like these.*

##### Quantity Equivalence

When comparing quantities for equivalence, the dimensions of each quantity must be the same, but not necessarily the same unit.
This is referred to as the units being commensurable in UCUM.
For example, units of `'cm'` and `'m'` can be compared, but units of `'cm2'` and `'cm'` cannot.

When the units are different the quantity values must be [**converted**](#unit-conversions) to the least granular unit before comparison.<br/>
*(using the least granular unit ensures that the precision of the converted value is not artificially increased by the conversion, and thus doesn't impact the rounded decimal comparison)*<br/>
If this process returns empty (e.g. because the units are not valid, not commensurable, or conversion is not supported), then the result of the equivalence comparison is `false`.
{:.stu}

§q7:Implementations are not required to fully support operations on units, but they SHALL at least respect units, recognizing when units differ.§

Attempting to operate on quantities with invalid units will result in `false`.

Once the units are the same, the values can be compared using decimal equivalence: values must be equal, comparison is done on values rounded to the precision of the least precise operand. Trailing zeroes after the decimal are ignored in determining precision (as described above).

For example:
```fhirpath
2.1 'cm' ~ 21 'mm'  // true ; convert to least granular 'cm' (2.1 ~ 2.1), round to least precise and compare (2.1 = 2.1)
21 'mm' ~ 2 'cm'    // true ; convert to least granular 'cm' (2.1 ~ 2), round to least precise and compare (2 = 2)
4 'g' ~ 4000 'mg'   // true ; convert to least granular 'g' (4 ~ 4.000), round to least precise and compare (4 = 4)
4 'g' ~ 4040 'mg'   // true ; convert to least granular 'g' (4 ~ 4.040), round to least precise and compare (4 = 4)
1 '[in_i]' ~ 2.5 'cm' // true ; convert to least granular '[in_i]' (1 ~ 0.98..), round to least precise and compare (1 = 1)
23 'Cel' ~ 73.4 '[degF]' // true ; comparison of UCUM "special" units on non-ratio scales (23 'Cel' = 73.4 '[degF]' exactly)
1 'cm' ~ 1 's' // false ; incompatible dimensions (length vs time), so quantities are not equivalent
(1 'cm' | 2 'g') ~ (1 's' | 2 'g') // false ; collections require equivalent items regardless of order; 2 'g' matches, but 1 'cm' and 1 's' are not equivalent
```

As noted in the [Time-valued Quantities](#time-valued-quantities) section, all units are considered either equal or equivalent across UCUM definite durations and calendar units.
Hence when the arguments are a mix of these, the results can be compared.
{:.stu}

For example:
{:.stu}
``` fhirpath
1 year ~ 1 'a' // true ; year and 'a' are equivalent by definition
1 year ~ 12 'mo' // true ; convert to least granular 'a': 12 'mo' → 1 'a' (UCUM), and 'a' ~ year by definition
1 year ~ 11 months // true ; convert to least granular year (1 ~ 0.9166666), round to least precise and compare (1 = 1)
1 second ~ 1 's' // true ; second and 's' are equal by definition (thus equivalent)
```
{:.stu}

##### Date/Time Equivalence

For `Date`, `DateTime` and `Time` equivalence, the comparison is the same as for equality, with the exception that if the input values have different levels of precision, the result is `false`, rather than empty (`{ }`). As with equality, the second and millisecond precisions are considered a single precision using a decimal, with decimal equivalence semantics.

For example:
``` fhirpath
@2012 ~ @2012 // true
@2012 ~ @2013 // false
@2012-01 ~ @2012 // false ; different precision
@2012-01-01T10:30 ~ @2012-01-01T10:30 // true
@2012-01-01T10:30 ~ @2012-01-01T10:31 // false
@2012-01-01T10:30:31 ~ @2012-01-01T10:30 // false ; different precision
@2012-01-01T10:30:31.0 ~ @2012-01-01T10:30:31 // true
@2012-01-01T10:30:31.1 ~ @2012-01-01T10:30:31 // false
```

##### String Equivalence

For strings, equivalence returns `true` if the strings are the same value while ignoring case and locale, and normalizing whitespace.

Normalizing whitespace means treating as equivalent all characters in the Unicode White_Space character class (as defined in https://www.unicode.org/Public/UCD/latest/ucd/PropList.txt) -
*this set includes the characters in the whitespace lexical category*.
{:.stu}

> **Note:** Multiple whitespace characters in sequence are not collapsed to a single character. 
> This could be done by using `replaceMatches('\\s+', ' ')` on both sides, which would normalize the whitespace to a single space.
> However, not all regex implementations cover the same set of whitespace characters with `\s`; this inconsistency is outside the scope of FHIRPath.<br/>
> To simply trim the start and end of content, use the [`trim()`](#trim--string) function.

For example:
```
'a b' ~ 'a\tb' // true
'a     b' ~ 'a b' // false ; the whitespace isn't collapsed
'a b' ~ 'A B' // true ; case insensitive test
```

#### != (Not Equals)

The converse of the equals operator, returning `true` if equal returns `false`; `false` if equal returns `true`; and empty (`{ }`) if equal returns empty. In other words, `A != B`{:.fhirpath} is short-hand for `(A = B).not()`{:.fhirpath}.

#### !~ (Not Equivalent)

The converse of the equivalent operator, returning `true` if equivalent returns `false` and `false` is equivalent returns `true`. In other words, `A !~ B`{:.fhirpath} is short-hand for `(A ~ B).not()`{:.fhirpath}.

### Comparison

* The comparison operators are defined for strings, integers, longs, decimals, quantities, dates, datetimes and times.
* If one or both of the arguments is an empty collection, a comparison operator will return an empty collection.
* Both arguments must be collections with single values, and the evaluator will throw an error if either collection has more than one item.
* Both arguments must be of the same type (or [implicitly convertible](#conversion) to the same type), and the evaluator will throw an error if the types differ.
* When comparing integers or longs to decimals, the integer/long value will be implicitly converted to a decimal to make comparison possible.<br/>
  *arguments are compared ignoring trailing zeroes after the decimal point (same as with equality)*
* String ordering is strictly lexical and is based on the Unicode value of the individual characters.

When comparing quantities, the dimensions of each quantity must be the same, but not necessarily the unit.
This is referred to as the units being commensurable in UCUM.
For example, units of `'cm'` and `'m'` can be compared, but units of `'cm2'` and `'cm'` cannot. 

When quantity units are different the quantities must be [**converted**](#unit-conversions) to the same unit, or a common unit before comparison.
If this process returns empty (e.g. because the units are not valid, or not commensurable), then the result of the comparison is empty (`{ }`).
{:.stu}

§cmp1:Implementations are not required to fully support operations on units, but they SHALL at least respect units, recognizing when units differ.§

Attempting to compare quantities with invalid units will result in empty (`{ }`).

Once the units are the same, the values can be compared using simple decimal comparison, ignoring trailing zeroes after the decimal point (as described above).

As noted in the [Time-valued Quantities](#time-valued-quantities) section, years and months are not comparable across UCUM definite durations and calendar units. Hence when the arguments are a mix of these, the result is empty as they are considered un-comparable.<br/>
Note that explicit conversion using [`toQuantity()`](#fn-toquantity) will change code-systems to intentionally perform this comparison.

For example:
``` fhirpath
1 year > 1 'a' // empty ({ }) ; these units are not comparable
10 seconds > 1 's' // true
2 year.toQuantity('a') > 1 'a' // true ; as the units have been explicitly converted, these are now comparable
6 months > 1 year  // false ; convert to 'year' (0.5 > 1) and compare
```

For partial Date, DateTime, and Time values, the comparison is performed by comparing the values at each precision, beginning with years, and proceeding to the finest precision specified in either input, and respecting timezone offsets. If one value is specified to a different level of precision than the other, the result is empty (`{ }`) to indicate that the result of the comparison is unknown. As with equality and equivalence, the second and millisecond precisions are considered a single precision using a decimal, with decimal comparison semantics.

See the [Equals](#equals) operator for discussion on respecting timezone offsets in comparison operations.

#### &gt; (Greater Than)

The greater than operator (`>`) returns `true` if the first operand is strictly greater than the second. The operands must be of the same type, or convertible to the same type using an [implicit conversion](#conversion).

For example:
``` fhirpath
10 > 5 // true
10 > 5.0 // true; note the 10 is converted to a decimal to perform the comparison
'abc' > 'ABC' // true
4 'm' > 4 'cm' // true (or { } if the implementation does not support unit conversion)

@2018-03-01 > @2018-01-01 // true ; same precision
@2018-03 > @2018-03-01 // empty ({ }) ; different precisions
@2018-03-01T10:30:00 > @2018-03-01T10:00:00 // true
@2018-03-01T10 > @2018-03-01T10:30 // empty ({ }) ; different precisions
@2018-03-01T10:30:00 > @2018-03-01T10:30:00.0 // false (values are equal to seconds, trailing zeroes after the decimal are ignored)

@T10:30:00 > @T10:00:00 // true
@T10 > @T10:30 // empty ({ }) ; different precisions
@T10:30:00 > @T10:30:00.0 // false ; values are equal to seconds, trailing zeroes after the decimal are ignored
```

#### &lt; (Less Than)

The less than operator (`<`) returns `true` if the first operand is strictly less than the second. The operands must be of the same type, or convertible to the same type using [implicit conversion](#conversion).

For example:
``` fhirpath
10 < 5 // false
10 < 5.0 // false ; note the 10 is converted to a decimal to perform the comparison
'abc' < 'ABC' // false
4 'm' < 4 'cm' // false (or { } if the implementation does not support unit conversion)

@2018-03-01 < @2018-01-01 // false
@2018-01-01 < @2018-01-01 // false ; same precision
@2018-03 < @2018-03-01 // empty ({ }) ; different precisions
@2018-03-01T10:30:00 < @2018-03-01T10:00:00 // false
@2018-03-01T10 < @2018-03-01T10:30 // empty ({ }) ; different precisions
@2018-03-01T10:30:00 < @2018-03-01T10:30:00.0 // false ; values are equal to seconds, trailing zeroes after the decimal are ignored

@2018-01-01T16:00:00+11:00 < @2018-01-01T15:00:00.0+10:00 // false (same moment in diff timezones)
@2018-01-01T16:00:00+12:00 < @2018-01-01T15:00:00.0+10:00 // true (4pm+12 is less than 5pm+10 when timezones are considered)

@T10:30:00 < @T10:00:00 // false
@T10 < @T10:30 // empty ({ }) ; different precisions
@T10:30:00 < @T10:30:00.0 // false ; values are equal to seconds, trailing zeroes after the decimal are ignored
```

#### &lt;= (Less or Equal)

The less or equal operator (`<=`) returns `true` if the first operand is less than or equal to the second. The operands must be of the same type, or convertible to the same type using [implicit conversion](#conversion).

For example:
``` fhirpath
10 <= 5 // false
10 <= 5.0 // false ; note the 10 is converted to a decimal to perform the comparison
'abc' <= 'ABC' // false
4 'm' <= 4 'cm' // false (or { } if the implementation does not support unit conversion)

@2018-03-01 <= @2018-01-01 // false
@2018-01-01 <= @2018-01-01 // true ; equal with same precision
@2018-03 <= @2018-03-01 // empty ({ }) ; different precisions
@2018-03-01T10:30:00 <= @2018-03-01T10:00:00 // false
@2018-03-01T10 <= @2018-03-01T10:30 // empty ({ }) ; different precisions
@2018-03-01T10:30:00 <= @2018-03-01T10:30:00.0 // true ; values are equal to seconds, trailing zeroes after the decimal are ignored

@2018-01-01T16:00:00+11:00 <= @2018-01-01T15:00:00.0+10:00 // true (same moment in diff timezones)
@2018-01-01T16:00:00+12:00 <= @2018-01-01T15:00:00.0+10:00 // true (4pm+12 is less than 5pm+10 when timezones are considered)

@T10:30:00 <= @T10:00:00 // false
@T10 <= @T10:30 // empty ({ }) ; different precisions
@T10:30:00 <= @T10:30:00.0 // true
```

#### &gt;= (Greater or Equal)

The greater or equal operator (`>=`) returns `true` if the first operand is greater than or equal to the second. The operands must be of the same type, or convertible to the same type using [implicit conversion](#conversion).

For example:
``` fhirpath
10 >= 5 // true
10 >= 5.0 // true ; note the 10 is converted to a decimal to perform the comparison
'abc' >= 'ABC' // true
4 'm' >= 4 'cm' // true (or { } if the implementation does not support unit conversion)

@2018-03-01 >= @2018-01-01 // true
@2018-01-01 >= @2018-01-01 // true ; equal with same precision
@2018-03 >= @2018-03-01 // empty ({ }) ; different precisions
@2018-03-01T10:30:00 >= @2018-03-01T10:00:00 // true
@2018-03-01T10 >= @2018-03-01T10:30 // empty ({ }) ; different precisions
@2018-03-01T10:30:00 >= @2018-03-01T10:30:00.0 // true ; values are equal to seconds, trailing zeroes after the decimal are ignored

@T10:30:00 >= @T10:00:00 // true
@T10 >= @T10:30 // empty ({ }) ; different precisions
@T10:30:00 >= @T10:30:00.0 // true ; values are equal to seconds, trailing zeroes after the decimal are ignored
```

<a name="fn-comparable"></a>
#### comparable(other : Quantity) : Boolean
{:.stu}

> **Note:** The contents of this section are Standard for Trial Use (STU)
{: .stu-note }

Returns `true` if the input Quantity can be compared with the `other` Quantity and their relationship to each other determined.
Comparable means that both have values, and the units are the same (irrespective of the system), or both have `code` and `system` values,
and the `system` is recognized by the FHIRPath implementation, and the codes are comparable within that system
(e.g. `'d'` (days) and `'h'` (hours), or `'[in_i]'` (inches) and `'cm'` (centimeters)). 
This is referred to as the units being commensurable in UCUM.
{:.stu}

If either or both inputs are empty, or either input is not a single Quantity value, the result is empty (`{ }`).
{:.stu}

> Returning true from this function indicates that a result from [equality](#equality) or [comparison](#comparison) functions will succeed, and not return empty.<br/>
> These functions consider [**unit conversion**](#unit-conversions) where required.
{:.stu}

For example:
{:.stu}
``` fhirpath
1 'mg'.comparable(2 'mg') // true ; these types are comparable
1 'm'.comparable(20 'cm') // true ; these types are both metric distance measures
2 '1'.comparable(3) // true ; the integer will implicitly convert to a Quantity with unit `'1'` which is the same system/code so is comparable
1.comparable(2) // true ; these will both convert to quantities with the same system/code, hence are comparable
```
{:.stu}
``` fhirpath
1 'in_i'.comparable(1 'cm') // true ; these UCUM units can be compared/converted
1 year.comparable(1 'a') // false ; these units are equivalent, not equal hence not comparable
1 'Cel'.comparable(1 '[degF]') // true ; these ucum "special" units are comparable
```
{:.stu}

This function can be used to guard [comparison operations](#comparison) to prevent returning empty results when the quantities are not comparable:
{:.stu}

``` fhirpath
iif(Observation.value.comparable(2 'mg'), Observation.value < 2 'mg', false)
```
{:.stu}

### Types

#### is _type specifier_

If the left operand is a collection with a single item and the second operand is a type identifier, this operator returns `true` if the type of the left operand is the type specified in the second operand, or a subclass thereof. If the input value is not of the type, this operator returns `false`. If the identifier cannot be resolved to a valid type identifier, the evaluator will throw an error. If the input collections contains more than one item, the evaluator will throw an error. In all other cases this operator returns `false`.

A _type specifier_ is an identifier that must resolve to the name of a type in a model. Type specifiers can have qualifiers, e.g. `FHIR.Patient`, where the qualifier is the name of the model.

The following example returns `true` if all Observation resources in the bundle have a status of finished:
``` fhirpath
Bundle.entry.resource.all($this is Observation implies status = 'finished')
```

#### is(type : _type specifier_)

The `is()` function is supported for backwards compatibility with previous implementations of FHIRPath. Just as with the `is` keyword, the `type` argument is an identifier that must resolve to the name of a type in a model. For implementations with compile-time typing, this requires special-case handling when processing the argument to treat it as a type specifier rather than an identifier expression:

``` fhirpath
Bundle.entry.resource.all($this.is(Observation) implies status = 'finished')
```

> **Note:** The `is()` function is defined for backwards compatibility only and may be deprecated in a future release.

#### as _type specifier_

If the left operand is a collection with a single item and the second operand is an identifier, this operator returns the value of the left operand if it is of the type specified in the second operand, or a subclass thereof. If the identifier cannot be resolved to a valid type identifier, the evaluator will throw an error. If there is more than one item in the input collection, the evaluator will throw an error. Otherwise, this operator returns the empty collection.

A _type specifier_ is an identifier that must resolve to the name of a type in a model. Type specifiers can have qualifiers, e.g. `FHIR.Patient`, where the qualifier is the name of the model.

For example:
``` fhirpath
Observation.component.where(value as Quantity > 30 'mg')
```

#### as(type : _type specifier_)

The `as()` function is supported for backwards compatibility with previous implementations of FHIRPath. Just as with the `as` keyword, the `type` argument is an identifier that must resolve to the name of a type in a model. For implementations with compile-time typing, this requires special-case handling when processing the argument to treat is a type specifier rather than an identifier expression:

``` fhirpath
Observation.component.where(value.as(Quantity) > 30 'mg')
```

> **Note:** The `as()` function is defined for backwards compatibility only and may be deprecated in a future release.

### Collections

#### | (union collections)
Merge the two collections into a single collection, eliminating any duplicate values *(items are considered duplicates if and only if the [equals](#equals) (`=`) operator returns `true`. i.e. `false` and empty both indicate that the items are not duplicates, and thus appear in the output collection)*.

There is no expectation of order in the resulting collection.

See the [union](#unionother-collection) function for more detail.

#### in (membership) : Boolean
If the left operand is a collection with a single item, this operator returns `true` if the item is in the right operand using equality semantics *(items are considered equal if and only if the [equals](#equals) (`=`) operator returns `true`. i.e. `false` and empty both indicate that the items are not equal)*. If the left-hand side of the operator is empty, the result is empty, if the right-hand side is empty, the result is `false`. If the left operand has multiple items, an exception is thrown.

The following example returns `true` if `'Joe'` is in the list of given names for the Patient:

``` fhirpath
'Joe' in Patient.name.given
```

#### contains (containership) : Boolean
If the right operand is a collection with a single item, this operator returns `true` if the item is in the left operand using equality semantics *(items are considered equal if and only if the [equals](#equals) (`=`) operator returns `true`. i.e. `false` and empty both indicate that the items are not equal)*. If the right-hand side of the operator is empty, the result is empty, if the left-hand side is empty, the result is `false`. This is the converse operation of `in`.

The following example returns `true` if the list of given names for the Patient has `'Joe'` in it:

``` fhirpath
Patient.name.given contains 'Joe'
```

### Boolean logic
For all boolean operators, the collections passed as operands are first evaluated as Booleans (as described in [Singleton Evaluation of Collections](#singleton-evaluation-of-collections)). The operators then use three-valued logic to propagate empty operands.

> **Note:** To ensure that FHIRPath expressions can be freely rewritten by underlying implementations, there is no expectation that an implementation respect short-circuit evaluation. With regard to performance, §b1:implementations MAY use short-circuit evaluation to reduce computation§, but authors should not rely on such behavior, and §b2:implementations SHALL NOT change semantics with short-circuit evaluation.§ If short-circuit evaluation is needed to avoid effects (e.g. runtime exceptions), use the [`iif()`](#iif) function.

#### and

Returns `true` if both operands evaluate to `true`, `false` if either operand evaluates to `false`, and the empty collection (`{ }`) otherwise.

|and |true |false |empty |
| - | - | - | - |
|**true** |`true` |`false` |empty (`{ }`) |
|**false** |`false` |`false` |`false` |
|**empty** |empty (`{ }`) |`false` |empty (`{ }`) |
{:.grid}

#### or

Returns `false` if both operands evaluate to `false`, `true` if either operand evaluates to `true`, and empty (`{ }`) otherwise:

|or |true |false |empty |
| - | - | - | - |
|**true** |`true` |`true` |`true` |
|**false** |`true` |`false` |empty (`{ }`) |
|**empty** |`true` |empty (`{ }`) |empty (`{ }`) |
{:.grid}

#### not() : Boolean

Returns `true` if the input collection evaluates to `false`, and `false` if it evaluates to `true`. Otherwise, the result is empty (`{ }`):

|not |
|-|
|**true** |`false` |
|**false** |`true` |
|**empty** |empty (`{ }`) |
{:.grid}

#### xor

Returns `true` if exactly one of the operands evaluates to `true`, `false` if either both operands evaluate to `true` or both operands evaluate to `false`, and the empty collection (`{ }`) otherwise:

|xor |true |false |empty |
| - | - | - | - |
|**true** |`false` |`true` |empty (`{ }`) |
|**false** |`true` |`false` |empty (`{ }`) |
|**empty** |empty (`{ }`) |empty (`{ }`) |empty (`{ }`) |
{:.grid}

#### implies

If the left operand evaluates to `true`, this operator returns the boolean evaluation of the right operand. If the left operand evaluates to `false`, this operator returns `true`. Otherwise, this operator returns `true` if the right operand evaluates to `true`, and the empty collection (`{ }`) otherwise.

|implies |true |false |empty |
| - | - | - | - |
|**true** |`true` |`false` |empty (`{ }`) |
|**false** |`true` |`true` |`true` |
|**empty** |`true` |empty (`{ }`) |empty (`{ }`) |
{:.grid}

The implies operator is useful for testing conditionals. For example, if a given name is present, then a family name must be as well:

``` fhirpath
Patient.name.given.exists() implies Patient.name.family.exists()
CareTeam.onBehalfOf.exists() implies (CareTeam.member.resolve() is Practitioner)
StructureDefinition.contextInvariant.exists() implies StructureDefinition.type = 'Extension'
```

Note carefully that if the left side of an implies evaluates to empty, the result of the operation is the right side. This is often not the intended result, so the use of operators that ensure a value (such as `~`, instead of `=`) is recommended for testing boolean conditions, as illustrated in the following examples:
``` fhirpath
Medication.status ~ 'active' implies form.exists()

// if using the following expression in a constraint, it won't have the expected behavior of only requiring form if status was active.
// (using equal '=' would evaluate and return the right argument if the left argument (status) is missing from the input)
Medication.status = 'active' implies form.exists() // bad constraint expression

// More complex conditions
(type ~ 'incident' and severity ~ 'high') implies reviewDate.exists()

// Works with boolean too, but not special-cased
wasNotGiven ~ true implies reasonNotGiven.exists()
```

§imp1:Note that implies MAY use short-circuit evaluation in the case that the first operand evaluates to `false`.§

### Math

The math operators require each operand to be a single item. Both operands must be of the same type (using [implicit conversion](#conversion) where required). Each operator below specifies which types are supported.

Math operations on Date/Time types is defined in the [Date/Time Arithmetic](#datetime-arithmetic) section.

If there is more than one item, or incompatible items, the evaluation of the expression will end and signal an error to the calling environment.

As with the other operators, the math operators will return an empty collection if one or both of the operands are empty.

§m1:Implementations are not required to fully support operations on units, but they SHALL at least respect units, recognizing when units differ.§

§m2:Implementations that do support units SHALL do so as specified by the [\[UCUM\]](#UCUM) specification.§

UCUM does not support math operations with ["special"](#UCUM-special) units on non-ratio scales in UCUM (e.g. Fahrenheit (degF) or Celsius (Cel)) where a function is required to transform the value. 
Attempting to operate on quantities with invalid or "special" units will result in empty (`{ }`).
{:.stu}

§m3:Implementations that to not support processing UCUM units SHALL return empty when unable to correctly handle the units.§
{:.stu}

Operations that cause arithmetic overflow or underflow will result in empty (`{ }`).

#### * (multiplication)

Multiplies both arguments (supported for Integer, Long, Decimal, and Quantity) with the result being the same as the input type *(after any [implicit conversions](#conversion) to make both operands the same type)*.

For multiplication involving quantities, the resulting quantity will have an appropriate unit as determined by application of the [\[UCUM\]](#UCUM) specification.

Multiplication involving calendar units (apart from the UCUM default unit `'1'`) will return empty.

> **Note:** Performing multiplication with mixed type parameters of either Integer or Decimal and Quantity will result in an [Implicit conversion](#conversion) of the Integer/Decimal argument to a Quantity (with the UCUM default unit `'1'`).
> This effectively makes the operation a simple decimal multiplication of the 2 values, and returns a Quantity with the same unit as the Quantity argument.<br/>
> For example, `2 * 2 'cm'` would be evaluated as `2 '1' * 2 'cm'`, which would return `4 'cm'`, as would `2 'cm' * 2`.<br/>
> This is a common use case for math operations with quantities, and allows for simple math operations on quantities without needing to explicitly convert the non-quantity argument to a quantity.

For example:
``` fhirpath
12 'cm' * 3 'cm'  // 36 'cm2' ; multiply 12 by 3 and ucum handling for these units results in square cms.
3 'cm' * 12 'cm2' // 36 'cm3' ; multiply 12 by 3 and ucum handling for these units results in cubic cms.
10 'm/s' * 10 's' // 100 'm' ; multiply 10 by 10 and ucum handling for these units results in meters as seconds cancel out.
3 * 2 'cm'        // 6 'cm' ; via implicit conversion of 3 to the quantity 3 '1', and results using cm.
42 * 1 'm'        // 42 'm' ; via implicit conversion of 42 to the quantity 42 '1', and results using m, 
                  //        this is a convenient way to provide the units for a numeric value.
12 day * 45 'm'   // empty ( { } ) ; multiplication with calendar units is not supported
```

#### / (division)

Divides the left operand *(numerator)* by the right operand *(denominator)* (supported for Integer, Long, Decimal, and Quantity).
The result of a division is always either Decimal *(Numeric inputs)* or Quantity *(for Quantity inputs)*.
For integer division, use the `div` operator.

For division involving quantities, the resulting quantity will have an appropriate unit as determined by application of the [\[UCUM\]](#UCUM) specification.
e.g. `km` / `h` => `km/h`

Division involving calendar units (apart from the UCUM default unit `'1'`) will return empty

If an attempt is made to divide by zero, the result is empty.

For example:
```fhirpath
4 / 2  // 2 ; as a decimal value
2 / 4  // 0.5
12 / 0 // empty ({ })
0 / 0  // empty ({ }) ; div by zero error result reported as empty (not reporting an error)
```

> **Note:** Performing division with mixed type parameters of either Integer, Long or Decimal and Quantity will result in an [Implicit conversion](#conversion) of the numeric argument to a Quantity (with the UCUM default unit `'1'`).

Division examples involving quantities:
``` fhirpath
12 'cm2' / 3 'cm' // 4.0 'cm'
120 'm' / 60 's'  // 2 'm/s'
60 / 1 's'        // 60 '/s' ; note the unit here is not "seconds", it is "per second", as the seconds are on the denominator as determined by the UCUM specification.
60 's' / 2        // 30 's' ; simple division of the integer value
```

#### + (addition)

For Integer, Long, Decimal, and Quantity, adds the operands. For strings, concatenates the right operand to the end of the left operand.

The resulting datatype is the same as the input datatype *(after any [implicit conversions](#conversion) to make both operands the same type)*.

Addition for Date/Time types is defined in the [Date/Time Arithmetic](#datetime-arithmetic) section.

When adding quantities, the dimensions of each quantity must be the same, but not necessarily the unit. 
This is referred to as the units being commensurable in UCUM.
For example, units of 'cm' and 'm' can be added, but units of 'kg' and 'cm' cannot.

When the units of quantity arguments are different, the quantity values must be [**converted**](#unit-conversions) to the most granular unit, then simple addition on the values can be performed.
If this process returns empty (e.g. because the units are not valid, or not commensurable), then the result of the addition is empty (`{ }`).
{:.stu}

> **Note:** Performing addition with mixed type parameters of either Integer, Long or Decimal and Quantity will result in an [Implicit conversion](#conversion) of the Numeric argument to a Quantity (with the UCUM default unit `'1'`).
This means that unless the unit of the quantity was `'1'` too, the addition will return empty as the units are not commensurate.

§ad1:Implementations are not required to fully support operations on units, but they SHALL at least respect units, recognizing when units differ.§

Attempting to operate on quantities with invalid units will result in empty (`{ }`).

For example:
``` fhirpath
1 + 2          // 3 ; simple numeric addition
1 + 5L          // 6L ; implicit conversion to long then simple numeric addition
5L + 4.5       // 9.5 ; implicit conversion from long to decimal then addition
3 'm' + 3 'cm' // 303 'cm' ; via ucum conversion to most granular unit
3 'cm' + 3 'm' // 303 'cm' ; via ucum conversion to most granular unit
(3 'm' + 3 'cm').toQuantity('m') // 3.03 'm' ; as above except explicitly convert the result to meters
2 + 2 'cm'     // empty ({ }) ; implicit conversion to UCUM default units 2 '1', these units are not commensurate
2 + 2 '1'      // 4 '1' ; implicit conversion, then simple addition of same unit values
```

> **Note:** Quantity addition with month or year units requires explicit conversion, otherwise the result is empty.
> While calendar unit conversion is performed implicitly for [equality](#quantity-equality) and [comparison](#comparison)
> (e.g. `1 year = 12 months` returns `true`), quantity addition and subtraction require the author to convert explicitly
> using [`toQuantity()`](#fn-toquantity), since the calendar conversion factors are approximate ([equivalent](#quantity-equivalence))
> and the author should be intentional about accepting that approximation in a computed result.
{:.stu}

When both operands are time-valued quantities, the [most granular](#unit-conversions) rule applies as with other quantities,
where the most granular of the two units is the one appearing lower in the [time-valued unit conversion table](#fn-toquantity-conversion-factors).<br/>
When the operands come from different time-valued unit systems, the result is expressed in calendar units.
{:.stu}

Quantity addition examples with time based units:
``` fhirpath
2 minutes + 60 seconds  // 180 seconds ; calendar conversion to most granular unit (seconds)
60 's' + 2 minutes      // 180 seconds ; cross-system: most granular unit conversion (2 minutes to 120 seconds, then add), result in calendar units
1 'wk' + 2 days    // 9 days ; cross-system, result in calendar units
1 year + 12 months // empty ( {} ) ; month/year unit conversion is equivalent and needs explicit unit conversion
1 year + 12 'mo'   // empty ( {} ) ; year/month requires explicit conversion (UCUM 'mo' doesn't change this)
1 year.toQuantity('month') + 12 months // 24 months ; explicit conversion to months, then addition with the same unit
1 week + 14 days   // 21 days ; calendar conversion to most granular unit (days)
3 'd' + 1 'wk'     // 10 'd' ; UCUM conversion from 'wk' to 'd' then add
```

#### - (subtraction)

Subtracts the right operand from the left operand (supported for Integer, Long, Decimal, and Quantity).

The resulting datatype is the same as the input datatype *(after any [implicit conversions](#conversion) to make both operands the same type)*.

Subtraction for Date/Time types is defined in the [Date/Time Arithmetic](#datetime-arithmetic) section.

Handling subtraction of quantities is the same as adding with a negative value, as such refer to [addition](#-addition) for quantity unit conversion handling.

For example:
``` fhirpath
3 'm' - 3 'cm'    // 297 'cm' ; ucum conversion to centimeters (most granular unit)
3 'cm' - 3 'm'    // -297 'cm' ; ucum conversion to centimeters (most granular unit)
1 minute - 30 's' // 0.5 minute ; seconds converted to minutes, then value subtraction
```

#### div

Performs truncated division of the left operand by the right operand (supported for Integer, Long and Decimal). In other words, the division that ignores any remainder.

The resulting datatype is the same as the input datatype *(after any [implicit conversions](#conversion) to make both operands the same type)*.

For example:
``` fhirpath
5 div 2 // 2 (integer)
5.5 div 0.7 // 7 (decimal)
5 div 0 // empty ({ })
5L div 2 // 2L (long)
```

#### mod

Computes the remainder of the truncated division of its arguments (supported for Integer, Long and Decimal).

The resulting datatype is the same as the input datatype *(after any [implicit conversions](#conversion) to make both operands the same type)*.

For example:
``` fhirpath
5 mod 2 // 1
5.5 mod 0.7 // 0.6
5 mod 0 // empty ({ })
```

#### &amp; (String concatenation)

For strings, will concatenate the strings, where an empty operand is taken to be the empty string. This differs from `+` on two strings, which will result in an empty collection when one of the operands is empty. This operator is specifically included to simplify treating an empty collection as an empty string, a common use case in string manipulation.

For example:
``` fhirpath
'ABC' + 'DEF' // 'ABCDEF'
'ABC' + { } + 'DEF' // { }
'ABC' & 'DEF' // 'ABCDEF'
'ABC' & { } & 'DEF' // 'ABCDEF'
```

### Date/Time Arithmetic

Date and time arithmetic operators are used to add time-valued quantities to date/time values. The left operand must be a `Date`, `DateTime`, or `Time` value, and the right operand must be a [`Quantity`](#quantity) with a time-valued unit.

The decimal portion of the time-valued quantity is only applied for second or millisecond precisions; for all other precisions, the decimal portion is ignored, since date/time arithmetic is performed with calendar duration semantics.

Within FHIRPath, §dta1:calculations involving date/times and calendar durations SHALL use calendar semantics as specified in [\[ISO8601\]](#ISO8601).§ Specifically:

| Datatype(s) | Quantity Unit(s) | Description |
| ----------- | ------- | ----- |
| `Date`, `DateTime` | `year`, `years` | The year, positive or negative, is added to the year component of the date or time value. If the resulting year is out of range, an error is thrown. If the month and day of the date or time value is not a valid date in the resulting year, the last day of the calendar month is used.<br/>*(using `'a'` will signal an error)* |
| `Date`, `DateTime` | `month`, `months` | The month, positive or negative is divided by 12, and the integer portion of the result is added to the year component. The remaining portion of months is added to the month component. If the resulting date is not a valid date in the resulting year, the last day of the resulting calendar month is used.<br/>*(using `'mo'` will signal an error)* |
| `Date`, `DateTime` | `week`, `weeks`, or `'wk'` | The week, positive or negative, is multiplied by 7, and the resulting value is added to the day component, respecting calendar month and calendar year lengths. |
| `Date`, `DateTime` | `day`, `days`, or `'d'` | The day, positive or negative, is added to the day component, respecting calendar month and calendar year lengths. |
| `DateTime`, `Time` | `hour`, `hours`, or `'h'` | The hours, positive or negative, are added to the hour component, with each 24 hour block counting as a calendar day, and respecting calendar month and calendar year lengths. |
| `DateTime`, `Time` | `minute`, `minutes`, or `'min'` | The minutes, positive or negative, are added to the minute component, with each 60 minute block counting as an hour, and respecting calendar month and calendar year lengths. |
| `DateTime`, `Time` | `second`, `seconds`, or `'s'` | The seconds, positive or negative, are added to the second component, with each 60 second block counting as a minute, and respecting calendar month and calendar year lengths. |
| `DateTime`, `Time` | `millisecond`, `milliseconds`, or `'ms'` | The milliseconds, positive or negative, are added to the millisecond component, with each 1000 millisecond block counting as a second, and respecting calendar month and calendar year lengths. |
{:.grid}

> **Note:** For all but years and months, calendar durations are both equal and equivalent to the corresponding UCUM definite-time duration unit.
> Note that due to the possibility of leap seconds, this is not totally accurate, however, for practical reasons, implementations typically ignore leap seconds when performing date/time arithmetic.

If there is more than one item, an item of an incompatible type, or an unsupported unit for the type, the evaluation of the expression will end and signal an error to the calling environment. This includes attempting to add date components to a Time.

If either or both arguments are empty (`{ }`), the result is empty (`{ }`).

Partial input values may require [unit conversion](#fn-toquantity-conversion-factors) where the unit being added is not present in the input value.
If the date/time value only has years present then when adding month quantities; use the direct conversion from months to years, otherwise convert the quantity to days, then to years (chaining as needed).
For all other partial precisions, convert as required chaining conversions where required.
Examples of this are in the operations below.
{:.stu}

§dta2:Implementers SHOULD produce a warning when decimal fractions are ignored in date/time arithmetic operations.§<br/>
§dta3:Authors SHOULD consider applying appropriate rounding functions ([`round()`](#fn-round), [`floor()`](#fn-floor), [`truncate()`](#fn-truncate), or [`ceiling()`](#fn-ceiling)) to quantity-valued inputs with decimal values before using them in date/time arithmetic expressions where quantity values might not be whole numbers.§
{:.stu}

As `Time` is cyclic, using arithmetic operations `+` or `-` on `Time` types can result in overflowing the time value, which will wrap around the beginning of the day.
So adding 1 hour to `@T23:30:00` will wrap around to `@T00:30:00`, which is consistent with the behaviour of `DateTime` values.
{:.stu}

#### + (addition)

Returns the value of the given `Date`, `DateTime`, or `Time`, incremented by the time-valued quantity, subject to all the rules and calendar semantics described [above](#datetime-arithmetic).

For Example:
```fhirpath
@1973-12-25 + 7 days    // @1974-01-01
@1973-12-25 + 7.9 days  // @1974-01-01 ; same as above as the decimal is truncated (though a warning may be triggered)
@1973-12-25 + 1 week    // @1974-01-01
@2019-03-01 + 24 months // @2021-03-01 ; month value overflows into years
@2026-01-31 + 1 month   // @2026-02-28 ; 31st day isn't valid in february, so revert to last day in the month
@2026-01-01T13:00:00 + 30 minutes // @2026-01-01T13:30:00
@1973-12-25T00:00:00.000+10:00 + 42.53 seconds // @1973-12-25T00:00:42.530+10:00
@1973-12-25 + 1 'd'     // @1973-12-26 ; ucum days handled as calendar units
@T23:30:00 + 1 hour     // @T00:30:00 ; overflow midnight
@T01:00:00 + 48 hour    // @T01:00:00 ; cycles through twice, but resulting time is the same
```

For partial date/time values where the time-valued quantity is more precise than the partial date/time, the operation is performed by converting the time-valued quantity to the highest precision in the partial (truncating any decimal fraction) and then adding to the date/time value. For example:
``` fhirpath
@2014 + 24 months  // @2016 ; 24/12 → 2 years (converting the quantity value to the highest precision in the partial date value)
@2014 + 23 months  // @2015 ; 23 months only constitutes one year (23/12 = 1.9167 → 1 year, remainder ignored)
@2016 + 365 days   // @2017 ; even though 2016 is a leap-year, the time-valued quantity (`365 days`) is converted to `1 year`, a standard calendar year of 365 days
@2014 + 11 months  // @2014 ; 11/12 = 0.9167 → 0 years, remainder ignored
@2026-02 + 5 weeks // @2026-03 ; 5 weeks is converted to 35 days, which are then converted to months 35/30 = 1.1667 → 1 month, remainder ignored
@2026-02 + 4 weeks // @2026-02 ; 4 weeks is converted to 28 days, which are then converted to months 28/30 = 0.9333 → 0 month, remainder ignored
```


#### - (subtraction)

Returns the value of the given `Date`, `DateTime`, or `Time`, decremented by the time-valued quantity, subject to all the rules and calendar semantics described [above](#datetime-arithmetic).

For example:
```fhirpath
@T00:30:00 - 1 hour // @T23:30:00
@T01:00:00 - 2 hours // @T23:00:00
```

For partial date/time values where the time-valued quantity is more precise than the partial date/time, the operation is performed by converting the time-valued quantity to the highest precision in the partial (truncating any decimal fraction) and then subtracting from the date/time value.
{:.stu}

For example:
``` fhirpath
@2014 - 24 months       // @2012 ; even though the date/time value is not specified to the level of precision of the time-valued quantity
@2019-03-01 - 24 months // @2017-03-01
@2014 - 1 month         // @2014 ; Partial date calculation 1/12 = 0.0833 → 0 years, remainder ignored
@2026-02 - 1 day        // @2026-02 ; Partial date calculation 1/30 = 0.0333 → 0 months, remainder ignored
```

<a name="unary-operators"></a>
### Unary operators (`+` and `-`)
{:.stu}

> **Note:** The contents of this section are Standard for Trial Use (STU)
{: .stu-note }

The unary operators support a single item input operand of type Integer, Long, Decimal, or Quantity. The output type is the same as the input type.
Using with any incompatible type will end and signal an error to the calling environment.
{:.stu}

The `+` operator returns the value of its operand unchanged.
{:.stu}

The `-` operator will negate the numeric value. If the value is a Quantity, the unit remains unchanged.
{:.stu}
> **Note:** The CQL language describes the `-` operator as unary negation.
{:.stu}

If the result of negating the number cannot be represented, the result is empty (`{ }`).
{:.stu}

If the input collection is empty, the result is empty (`{ }`).
{:.stu}

For example:
{:.stu}
``` fhirpath
+5 // a simple literal 5 numeric value
-4 // a simple negative value
-Account.balance.amount // The negated account balance
```
{:.stu}

### Operator precedence

Precedence of operations, in order from high to low:

``` txt
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

As customary, precedence may be established explicitly using parentheses (`( )`).

As an example, consider the following expression:

``` fhirpath
-7.combine(3)
```

Because the invocation operator (`.`) has a higher precedence than the unary negation (`-`), the unary negation will be applied to the result of the combine of 7 and 3, resulting in an error (because unary negation cannot be applied to a list):

``` fhirpath
-(7.combine(3)) // ERROR
```

Use parentheses to ensure the unary negation applies to the `7`:

``` fhirpath
(-7).combine(3) // -7, 3
```

## Aggregates

> **Note:** The contents of this section are Standard for Trial Use (STU)
{: .stu-note }

FHIRPath supports a general-purpose aggregate function to enable the calculation of aggregates such as sum, min, and max to be expressed:
{:.stu}

> **Note:** While the `aggregate()` function is powerful and flexible, authors are encouraged to use the built-in aggregate functions for sum, min, max, and avg described below where possible.
They are more concise, easier to read, and handle input types more effectively, unless you need to handle specific edge cases.
{:.stu}

<a name="aggregate"></a>
### aggregate(aggregator : ($total, $this, $index) => collection [, init : collection]) : collection
{:.stu}
Performs general-purpose aggregation by evaluating the aggregator expression for each item of the input collection. Within this expression, the standard iteration variables of `$this` and `$index` can be accessed, but also a `$total` aggregation variable.
{:.stu}

> This is a [scoped function](#scoped-functions): The `init` argument is evaluated once at the start to initialize the `$total` variable.<br/> The `aggregator` argument is then evaluated for each item (setting `$this`and `$index` for each), and has access to the current value of `$total` available. The result of the evaluation is then assigned to `$total`.<br/> The final value of `$total` is returned as the result of the function.<br/>  The `init` argument is evaluated once before setting `$this` and `$index`, so will be evaluated on the outer context, and will have access to outer `$this` values.
{:.stu}

The value of the `$total` variable is set to `init`, or empty (`{ }`) if no `init` value is supplied, and is set to the result of the aggregator expression after every iteration.<br/>
The result of the aggregate function is the value of `$total` after the last iteration.
{:.stu}

Using this function, sum can be expressed as:
{:.stu}

``` fhirpath
value.aggregate($this + $total, 0)
```
{:.stu}

Min could be expressed as:
{:.stu}

``` fhirpath
value.aggregate(iif($total.empty(), $this, iif($this < $total, $this, $total)))
```
{:.stu}

and average could be expressed as:
{:.stu}

``` fhirpath
value.aggregate($total + $this, 0) / value.count()
```
{:.stu}

### sum() : Integer | Long | Decimal | Quantity
{:.stu}
Returns the sum of all items in the input collection (in the same type).
{:.stu}

Accepts input collections with items of type: Integer, Long, Decimal or Quantity.
{:.stu}

§sum1:All items in the input collection SHALL be the same type, otherwise an exception is thrown.§
{:.stu}

If the input collection is empty (`{ }`), the result is empty.
{:.stu}

For example:
{:.stu}
``` fhirpath
( 1.0 | 2.0 | 3.0 | 4.0 | 5.0 ).sum() // 15.0
( 1.0 'mg' | 2.0 'mg' | 3.0 'mg' | 4.0 'mg' | 5.0 'mg' ).sum() // 15.0 'mg'
```
{:.stu}

### min() : Integer | Long | Decimal | Quantity | Date | DateTime | Time | String
{:.stu}
Returns the minimum item in the input collection. Comparison semantics are defined by the [Comparison Operators](#comparison) for the type of value being aggregated.
{:.stu}

Accepts input collections with items of type: Integer, Long, Decimal, Quantity, Date, DateTime, Time, or String.
{:.stu}

§min1:All items in the input collection SHALL be the same type, otherwise an exception is thrown.§
{:.stu}

If the input collection is empty (`{ }`), the result is empty.
{:.stu}

For example:
{:.stu}
``` fhirpath
( 2 | 4 | 8 | 6 ).min() // 2
( 2L | 4L | 8L | 6L ).min() // 2L
( @2012-12-31 | @2013-01-01 | @2012-01-01 ).min() // @2012-01-01
```
{:.stu}

### max() : Integer | Long | Decimal | Quantity | Date | DateTime | Time | String
{:.stu}
Returns the maximum item in the input collection. Comparison semantics are defined by the [Comparison Operators](#comparison) for the type of value being aggregated.
{:.stu}

Accepts input collections with items of type: Integer, Long, Decimal, Quantity, Date, DateTime, Time, or String.
{:.stu}

§max1:All items in the input collection SHALL be the same type, otherwise an exception is thrown.§
{:.stu}

If the input collection is empty (`{ }`), the result is empty.
{:.stu}

For example:
{:.stu}
``` fhirpath
( 2 | 4 | 8 | 6 ).max() // 8
( 2L | 4L | 8L | 6L ).max() // 8L
( @2012-12-31 | @2013-01-01 | @2012-01-01 ).max() // @2013-01-01
```
{:.stu}

### avg() : Decimal | Quantity
{:.stu}
Returns the average of all items in the input collection (in the same type).
{:.stu}

Accepts input collections with items of type: Decimal or Quantity.
{:.stu}

When used with Integer or Long, the arguments will be implicitly converted to Decimal before evaluation.
{:.stu}

§avg1:All items in the input collection SHALL be the same type, otherwise an exception is thrown.§
{:.stu}

If the input collection is empty (`{ }`), the result is empty.
{:.stu}

For example:
{:.stu}
``` fhirpath
( 5.5 | 4.7 | 4.8 ).avg() // 5.0
( 5.5 'cm' | 4.7 'cm' | 4.8 'cm' ).avg() // 5.0 'cm'
```
{:.stu}

## Lexical Elements
FHIRPath defines the following lexical elements:

|Element|Description|
|-|-|
|**Whitespace**|Whitespace defines the separation between tokens in the language|
|**Comment**|Comments are ignored by the language, allowing for descriptive text|
|**Literal**|Literals allow basic values to be represented within the language|
|**Symbol** |Symbols such as `+`, `-`, `*`, and `/` |
|**Keyword** |Grammar-recognized tokens such as `and`, `or` and `in` |
|**Identifier** |Labels such as type names and element names |
{:.grid}

### Whitespace
FHIRPath defines _tab_ (`\t`), _space_ (`` ``), _line feed_ (`\n`) and _carriage return_ (`\r`) as _whitespace_, meaning they are only used to separate other tokens within the language. Any number of whitespace characters can appear, and the language does not use whitespace for anything other than delimiting tokens.

### Comments
FHIRPath defines two styles of comments, _single-line_, and _multi-line_. A single-line comment consists of two forward slashes, followed by any text up to the end of the line:

``` fhirpath
2 + 2 // This is a single-line comment
```

To begin a multi-line comment, the typical forward slash-asterisk token is used. The comment is closed with an asterisk-forward slash, and everything enclosed is ignored:

``` fhirpath
/*
This is a multi-line comment
Any text enclosed within is ignored
*/
```

### Literals
Literals provide for the representation of values within FHIRPath. The following types of literals are supported:

|Literal|Description |
|-|-|
|**Empty** (`{ }`)|The empty collection|
|**[Boolean](#boolean)**|The boolean literals (`true` and `false`)|
|**[Integer](#integer)**|Sequences of digits in the range 0..2<sup>32</sup>-1|
|**[Decimal](#decimal)**|Sequences of digits with a decimal point, in the range (-10<sup>28</sup>+1)/10<sup>8</sup>..(10<sup>28</sup>-1)/10<sup>8</sup>|
|**[String](#string)**|Strings of any character enclosed within single-ticks (`'`)|
|**[Date](#date)**|The at-symbol (`@`) followed by a date (`yyyy-MM-DD`{:.formatted})|
|**[DateTime](#datetime)**|The at-symbol (`@`) followed by a datetime (`yyyy-MM-DDThh:mm:ss.fff(+|-)hh:mm`{:.formatted}) |
|**[Time](#time)**|The at-symbol (`@`) followed by a time (`Thh:mm:ss.fff(+|-)hh:mm`{:.formatted}) |
|**[Quantity](#quantity)**|An integer or decimal literal followed by a datetime precision specifier, or a [\[UCUM\]](#UCUM) unit specifier|
{: .grid}

For a more detailed discussion of the semantics of each type, refer to the link for each type.

### Symbols
Symbols provide structure to the language and allow symbolic invocation of common operators such as addition. FHIRPath defines the following symbols:

|Symbol|Description |
|-|-|
|`()`|Parentheses for delimiting groups within expressions|
|`[]`|Brackets for indexing into lists and strings|
|`{}`|Braces for delimiting exclusively empty lists|
|`.`|Period for qualifiers, accessors, and dot-invocation|
|`,`|Comma for delimiting items in a syntactic list|
|`= != <= < > >=`|Comparison operators for comparing values|
|`+ - * / \| &`|Arithmetic and other operators for performing computation|
{:.grid}

### Keywords
Keywords are tokens that are recognized by the parser and used to build the various language constructs. FHIRPath defines the following keywords:

|`$index` |`div` |`milliseconds` |`true`
|=|=|=|=|
|`$this` |`false` |`minute` |`week`
|`$total` |`hour` |`minutes` |`weeks`
|`and` |`hours` |`mod` |`xor`
|`as` |`implies` |`month` |`year`
|`contains` |`in` |`months` |`years`
|`day` |`is` |`or` |`second`
|`days` |`millisecond` |`seconds` |
{:.grid}

In general, keywords within FHIRPath are also considered _reserved_ words, meaning that it is illegal to use them as identifiers. FHIRPath keywords are reserved words, with the exception of the following keywords that may also be used as identifiers:

|`as` |`contains` |
|`is` | |
{:.grid}

If necessary, identifiers that clash with a reserved word can be delimited using a backtick (`` ` ``):

``` fhirpath
Patient.text.`div`.empty()
```

The `div` element of the `Patient.text` must be offset with backticks (`` ` ``) because `div` is both a keyword and a reserved word.

### Identifiers
Identifiers are used as labels to allow expressions to reference elements such as model types and properties. FHIRPath supports two types of identifiers, _simple_ and _delimited_.

A simple identifier is any alphabetical character or an underscore, followed by any number of alpha-numeric characters or underscores. For example, the following are all valid simple identifiers:

``` fhirpath
Patient
_id
valueDateTime
_1234
```

A delimited identifier is any sequence of characters enclosed in backticks (`` ` ``):

``` fhirpath
`QI-Core Patient`
`US-Core Diagnostic Request`
`us-zip`
```

The use of backticks allows identifiers to contains spaces, commas, and other characters that would not be allowed within simple identifiers. This allows identifiers to be more descriptive, and also enables expressions to reference models that have element or type names that are not valid simple identifiers.

FHIRPath [escape sequences](#string) for strings also work for delimited identifiers.

As with simple identifiers, when a delimited identifier is used at the root of a FHIRPath expression, it follows the same [type resolution rules](#path-selection) as described in the Path selection section.

### Case-Sensitivity
FHIRPath is a case-sensitive language, meaning that case is considered when matching keywords in the language. However, because FHIRPath can be used with different models, the case-sensitivity of type and element names is defined by each model.

## Environment variables

A token introduced by a % refers to a value that is passed into the evaluation engine by the calling environment. Using environment variables, authors can avoid repetition of fixed values and can pass in external values and data.

The following environmental values are set for all contexts:

``` fhirpath
%ucum       // (string) url for UCUM (http://unitsofmeasure.org, per http://hl7.org/fhir/ucum.html)
%context    // The original node that was passed to the evaluation engine before starting evaluation
```

Implementers should note that using additional environment variables is a formal extension point for the language. Various usages of FHIRPath may define their own externals, and implementers should provide some appropriate configuration framework to allow these constants to be provided to the evaluation engine at run-time. E.g.:

``` fhirpath
%`us-zip` = '[0-9]{5}(-[0-9]{4}){0,1}'
```

Note that the identifier portion of the token is allowed to be either a simple identifier (as in `%ucum`), or a delimited identifier to allow for alternative characters (as in ``%`us-zip` ``).

Note also that these tokens are not restricted to simple types, and they may have values that are not defined fixed values known prior to evaluation at run-time, though there is no way to define these kind of values in implementation guides.

Attempting to access an undefined environment variable will result in an error, but accessing a defined environment variable that does not have a value specified results in empty (`{ }`).

> **Note:** For backwards compatibility with some existing implementations, the token for an environment variable may also be a string, as in `%'us-zip'`, with no difference in semantics.

## Types and Reflection

### Models

Because FHIRPath is defined to work in multiple contexts, each context provides the definition for the structures available in that context. These structures are the *model* available for FHIRPath expressions. For example, within FHIR, the FHIR data types and resources are the model. To prevent namespace clashes, the type names within each model are prefixed (or namespaced) with the name of the model. For example, the fully qualified name of the Patient resource in FHIR is `FHIR.Patient`. The system types defined within FHIRPath directly are prefixed with the namespace `System`.

To allow type names to be referenced in expressions such as the `is` and `as` operators, the language includes a _type specifier_, an optionally qualified identifier that must resolve to the name of a model type.

When resolving a type name, the context-specific model is searched first. If no match is found, the `System` model (containing only the built-in types defined in the [Literals](#literals) section) is searched.

### Reflection
{:.stu}

> **Note:** The contents of this section are Standard for Trial Use (STU)
{: .stu-note }

FHIRPath supports limited reflection to provide the ability for expressions to access basic type information of values. 
The ability to access child elements or walk type trees to parent definitions using this mechanism is outside the scope of this specification.
The [`ofType()`](#fn-oftype) function can be used with type-names or their subtypes to filter content.
{:.stu}

#### Structures
{:.stu}
``` typescript
TypeInfo { baseType: string }
SimpleTypeInfo extends TypeInfo { namespace: string, name: string }
ClassInfo extends TypeInfo { namespace: string, name: string }
```
{:.stu}

> **Note:** These structures are a subset of the abstract metamodel used by the [Clinical Quality Language Tooling](https://github.com/cqframework/clinical_quality_language).
> Although the SimpleTypeInfo and ClassInfo appear the same here, implementations may have additional information (as is the case in the referenced CQL structures).
{:.stu}


#### type() : collection
{:.stu}
The `type` function returns the type information for each item of the input collection, using concrete subtypes of `TypeInfo`.
{:.stu}

> Note: using `X.ofType(Y)` or `X as Y` to filter content is better supported than using `X.type().name = 'Y'`.
{:.stu}

If the input collection is empty (`{ }`), or access to type information is unavailable, the result is empty.
{:.stu}

For primitive types such as `String` and `Integer`, the result is a `SimpleTypeInfo`:
{:.stu}

``` fhirpath
('John' | 'Mary').type()
```
{:.stu}

Results in:
{:.stu}
``` typescript
[
  SimpleTypeInfo { namespace: 'System', name: 'String', baseType: 'System.Any' },
  SimpleTypeInfo { namespace: 'System', name: 'String', baseType: 'System.Any' }
]
```
{:.stu}
*Note: The base type for primitives is defined as `System.Any`.*
{:.stu}


For complex types such as a FHIR CodeableConcept, the result is a `ClassInfo`:
{:.stu}

``` fhirpath
Patient.maritalStatus.type()
```
{:.stu}

Results in:
{:.stu}

``` typescript
{
  ClassInfo { namespace: 'FHIR', name: 'CodeableConcept', baseType: 'FHIR.Element' }
}
```
{:.stu}


## Type safety and strict evaluation

Strongly typed languages are intended to help authors avoid mistakes by ensuring that the expressions describe meaningful operations. For example, a strongly typed language would typically disallow the expression:

``` fhirpath
1 + 'John'
```

because it performs an invalid operation, namely adding numbers and strings. However, there are cases where the author knows that a particular invocation may be safe, but the compiler is not aware of, or cannot infer, the reason. In these cases, type-safety errors can become an unwelcome burden, especially for experienced developers.

Because FHIRPath may be used in different situations and environments requiring different levels of type safety, implementations may make different choices about how much type checking should be done at compile-time versus run-time, and in what situations. Some implementations requiring a high degree of type-safety may choose to perform strict type-checking at compile-time for all invocations. On the other hand, some implementations may be unconcerned with compile-time versus run-time checking and may choose to defer all correctness checks to run-time.

For example, since some functions and most operators will only accept a single item as input (and throw a run-time exception otherwise):

``` fhirpath
Patient.name.given + ' ' + Patient.name.family
```

will work perfectly fine, as long as the patient has a single name, but will fail otherwise. It is in fact "safer" to formulate such statements as either:

``` fhirpath
Patient.name.select(given + ' ' + family)
```

which would return a collection of concatenated first and last names, one for each name of a patient. Of course, if the patient turns out to have multiple given names, even this statement will fail and the author would need to choose the first name in each collection explicitly:

``` fhirpath
Patient.name.first().select(given.first() + ' ' + family.first())
```

It is clear that, although more robust, the last expression is also much more elaborate, certainly in situations where, because of external constraints, the author is sure names will not repeat, even if the unconstrained object model allows repetition.

Apart from throwing exceptions, unexpected outcomes may result because of the way the equality operators are defined. The expression

``` fhirpath
Patient.name.given = 'Wouter'
```

will return `false` as soon as a patient has multiple names, even though one of those may well be 'Wouter'. Again, this can be corrected:

``` fhirpath
Patient.name.where(given = 'Wouter').exists()
```

but is still less concise than would be possible if constraints were well known in advance.

### Compile-time checks
In cases where compile-time checking like this is desirable, implementations may choose to protect against such cases by employing strict typing. Based on the definitions of the operators and functions involved in the expression, and given the types of the inputs, a compiler can analyze the expression and determine whether "unsafe" situations can occur.

Unsafe uses are:

* A function that requires an input collection with a single item is called on an output that is not guaranteed to have only one item.
* A function is passed an argument that is not guaranteed to be a single value.
* A function is passed an input value or argument that is not of the expected type
* An operator that requires operands to be collections with a single item is called with arguments that are not guaranteed to have only one item.
* An operator has operands that are not of the expected type
* Equality operators are used on operands that are not both collections or collections containing a single item of the same type.

There are a few constructs in the FHIRPath language where the compiler cannot determine the type:

* The `children()` and `descendants()` functions
* The `resolve()` function
* An element that is polymorphic (e.g. a choice type in FHIR such as `value[x]`)

Note that the `resolve()` function is defined by the FHIR context, it is not part of FHIRPath directly. For more information see the [FHIRPath](https://hl7.org/fhir/fhirpath.html#functions) section of the FHIR specification.

Authors can use the `as` operator or [`ofType()`](#fn-oftype) function directly after such constructs to inform the compiler of the expected type.

In cases where a compiler finds places where a collection of multiple items can be present while just a single item is expected, the author will need to make explicit how repetitions are dealt with. Depending on the situation one may:

* Use `first()`, `last()` or indexer (`[ ]`) to select a single item
* Use `select()` and `where()` to turn the expression into one that evaluates each of the repeating items individually (as in the examples above)

### Run-time behavior
{:.stu}
In the absence of strongly-typed compile-time checks, the implementation may still provide strong type-checks at run-time.
The difference between strongly typed compile-time checks and strongly-typed run-time checks is that the latter will only throw errors when the data does not conform to type requirements, where the former checks that any data is guaranteed to be conformant due to the construction of the expression.
{:.stu}

However, §rt1:the default run-time behavior of a compliant FHIRPath implementation SHOULD be weakly typed.§ This means:
{:.stu}

return an empty collection ({ }) when a path references an element that does not exist on the context node
{:.stu}

## Formal Specifications

### Formal Syntax

The formal syntax for FHIRPath is specified as an [Antlr 4.0](http://www.antlr.org/) grammar file (g4) and included in this specification at the following link:

[grammar.html](grammar.html)

> **Note:** If there are discrepancies between this documentation and the grammar included at the above link, the grammar is considered the source of truth.

### Model Information

The model information returned by the reflection function `type()`  is specified as an XML Schema document (xsd) and included in this specification at the following link:

[modelinfo.xsd](modelinfo.xsd)

> **Note:** The model information file included here is not a normative aspect of the FHIRPath specification. It is the same model information file used by the [Clinical Quality Framework Tooling](http://github.com/cqframework/clinical_quality_language) and is included for reference as a simple formalism that meets the requirements described in the [Reflection](#reflection) section above.

As discussed in the section on case-sensitivity, each model used within FHIRPath determines whether or not identifiers in the model are case-sensitive. This information is provided as part of the model information and §mdl1:tooling SHOULD respect the case-sensitive settings for each model.§

### URI and Media Types
To uniquely identify the FHIRPath language, the following URI is defined:

``` txt
http://hl7.org/fhirpath
```

In addition, a media type is defined to support describing FHIRPath content:

``` txt
text/fhirpath
```

> **Note:** The appendices are included for informative purposes and are not a normative part of the specification.



## References

<a name="bibliography"></a>
- <a name="ANTLR"></a>[ANTLR] Another Tool for Language Recognition (ANTLR) <http://www.antlr.org/>{:target="_blank"}
- <a name="ISO8601"></a>[ISO8601] Date and time format - ISO 8601. <https://www.iso.org/iso-8601-date-and-time-format.html>{:target="_blank"}
- <a name="CQL"></a>[CQL] HL7 Cross-Paradigm Specification: Clinical Quality Language, Release 1, STU Release 1.3. <http://www.hl7.org/implement/standards/product_brief.cfm?product_id=400>{:target="_blank"}
- <a name="MOF"></a>[MOF] Meta Object Facility. <https://www.omg.org/spec/MOF/>{:target="_blank"}, version 2.5.1, November 2016
- <a name="XMLRE"></a>[XMLRE] Regular Expressions. XML Schema 1.1. <https://www.w3.org/TR/xmlschema11-2/#regexs>{:target="_blank"}
- <a name="PCRE"></a>[PCRE] Pearl-Compatible Regular Expressions. <http://www.pcre.org/>{:target="_blank"}
- <a name="UCUM"></a>[UCUM] Unified Code for Units of Measure (UCUM) <http://unitsofmeasure.org/ucum.html>{:target="_blank"}, Version 2.1, Revision 442 (2017-11-21)
- <a name="UCUM-special"></a>[UCUM-special] Unified Code for Units of Measure (UCUM) - Special Units on non-ratio Scales <https://ucum.org/ucum#section-Special-Units-on-non-ratio-Scales>{:target="_blank"}
- <a name="FHIR"></a>[FHIR] HL7 Fast Healthcare Interoperability Resources <http://hl7.org/fhir>{:target="_blank"}
- [grammar.html](grammar.html)
- [modelinfo.xsd](modelinfo.xsd)
- <a name="fluent"></a>[Fluent] Fluent interface pattern. <https://en.wikipedia.org/wiki/Fluent_interface>{:target="_blank"}



## Dependencies

{% include dependency-table-short.xhtml %}

## IP Statements

{% include ip-statements.xhtml %}

<a name="hl7v2"></a>
## Use of FHIRPath on HL7 Version 2 messages
{: .appendix }

FHIRPath can be used against HL7 V2 messages. This UML diagram summarizes the
Object Model on which the FHIRPath statements are written:

![Class Model for HL7 V2](v2-class-model.png){: height="456",width="760"}

In this Object Model:

* The object graph always starts with a message.
* Each message has a list of segments.
* In addition, Abstract Message Syntax is available through the groups() function, for use where the message follows the Abstract Message Syntax sufficiently for the parser to reconcile the segment list with the structure.
* The names of the groups are the names published in the specification, e.g. 'PATIENT_OBSERVATION' (with spaces, where present, replaced by underscores. In case of doubt, consult the V2 XML schemas).
* Each Segment has a list of fields, which each have a list of "Cells". This is necessary to allow for repeats, but users are accustomed to just jumping to Element - use the function elements() which returns all repeats with the given index.
* A "cell" can be either an Element, a Component or a Sub-Components. Elements can contain Components, which can contain Sub-Components. Sub-Sub-Components are not allowed.
* Calls may have a simple text content, or a series of (sub-)components. The simple() function returns either the text, if it exists, or the return value of simple() from the first component
* A V2 data type (e.g. ST, SN, CE etc) is a profile on Cell that specifies whether it has simple content, or complex content.
* todo: this object model doesn't make provision for non-syntax escapes in the simple content (e.g. `\.b\`).
* all the lists are 1 based. That means the first item in the list is numbered 1, not 0.

Some example queries:

``` fhirpath
Message.segment.where(code = 'PID').field[3].element.first().simple()
```

Get the value of the first component in the first repeat of PID-3

``` fhirpath
Message.segment[2].elements(3).simple()
```

Get a collection  with is the string values of all the repeats in the 3rd element of the 2nd segment. Typically, this assumes that there are no repeats, and so this is a simple value.

``` fhirpath
Message.segment.where(code = 'PID').field[3].element.where(component[4].value = 'MR').simple()
```

Pick out the MR number from PID-3 (assuming, in this case, that there's only one PID segment in the message. No good for an A17). Note that this returns the whole Cell - e.g. `|value^^MR|`, though often more components will be present)

``` fhirpath
Message.segment.where(code = 'PID').elements(3).where(component[4].value = 'MR').component[1].text
```

Same as the last, but pick out just the MR value

``` fhirpath
Message.group('PATIENT').group('PATIENT_OBSERVATION').item.ofType(Segment)
  .where(code = 'OBX' and elements(2).exists(components(2) = 'LN')))
```

Return any OBXs from the patient observations (and ignore others e.g. in a R01 message) segments that have LOINC codes.
Note that if the parser cannot properly parse the Abstract Message Syntax, group() must fail with an error message.

## FHIRPath Tooling and Implementation
{: .appendix }

The list of known tooling and implementation projects for the FHIRPath language has been moved to the [HL7 confluence site](https://confluence.hl7.org/display/FHIRI/FHIRPath+Implementations){:target="_blank"}


## Cross Version Analysis

{% include cross-version-analysis-inline.xhtml %}

## Global Profiles

{% include globals-table.xhtml %}
