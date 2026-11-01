
### Release 3 STU3 Change Summary
**Summary of Changes:**
* Added new functions: `coalesce()`, `sort()`, `repeatAll()`, `defineVariable()`, `matchesFull()`, `indexOf()`, `lastIndexOf()`, `comparable()`, `pathname()`
* Added aggregate functions: `sum()`, `min()`, `max()`, `avg()`
* Added date/time component extraction functions: `yearOf()`, `monthOf()`, `dayOf()`, `hourOf()`, `minuteOf()`, `secondOf()`, `millisecondOf()`, `timezoneOffsetOf()`, `dateOf()`, `timeOf()`
* Added boundary and precision functions: `lowBoundary()`, `highBoundary()`, `precision()`
* Added support for Long (integer64) type with required `L` suffix
* Added constructor syntax support for type instantiation
* Added date/time subtraction operations
* Added `combine()` function with `preserveOrder` parameter
* Added case-sensitivity options to regex functions
* Clarified singleton evaluation rules and function behavior for multi-item collections
* Clarified iteration context and scoped function behavior
* Clarified `iif()` delayed argument evaluation semantics
* Clarified date/time arithmetic edge cases (overflow, fractions, subtraction)
* Clarified quantity unit handling and UCUM conformance
* Clarified equality semantics for collections
* Clarified rounding behavior for negative numbers
* Improved conversion documentation and `toString()` representations
* Defined calendar duration units code system
* Corrected numerous examples throughout
* Updated implementation references
* Added authors and contributors list
* Typographical corrections throughout
* Documentation source format changed from asciidoc to markdown
* Simplified reflection capabilities - removed element information returned from `type()`
* Added many new examples for functions that did not previously have examples


#### Issues Released in 3.0.0-ballot
*(issues addressed before Jan 2025 ballot)*

* [FHIR-44803](https://jira.hl7.org/browse/FHIR-44803): Spelling errors and typos
* [FHIR-37705](https://jira.hl7.org/browse/FHIR-37705): Clarify behavior when the 2nd argument to substring() is negative
* [FHIR-32882](https://jira.hl7.org/browse/FHIR-32882): Support for IndexOf and LastIndexOf operators
* [FHIR-36270](https://jira.hl7.org/browse/FHIR-36270): Add matchesFull to support full matching
* [FHIR-40810](https://jira.hl7.org/browse/FHIR-40810): The link to the .NET FhirPath implementation is out of date.
* [FHIR-40620](https://jira.hl7.org/browse/FHIR-40620): The list of implementations is obsolete
* [FHIR-20678](https://jira.hl7.org/browse/FHIR-20678): Quantity format for toString -- support for non-UCUM units?
* [FHIR-46808](https://jira.hl7.org/browse/FHIR-46808): Series of typos in the normative parts of the spec
* [FHIR-20937](https://jira.hl7.org/browse/FHIR-20937): Double-escaped +
* [FHIR-21234](https://jira.hl7.org/browse/FHIR-21234): Add functions to date and decimal for lowBoundary(), highBoundary() and precision()
* [FHIR-25189](https://jira.hl7.org/browse/FHIR-25189): Clarify behavior of `is` when left operand is an empty collection
* [FHIR-25403](https://jira.hl7.org/browse/FHIR-25403): Incorrect example of substring usage in the function invocation section
* [FHIR-26376](https://jira.hl7.org/browse/FHIR-26376): Is keyword is repeated in the list of keywords that can be identifiers
* [FHIR-26542](https://jira.hl7.org/browse/FHIR-26542): Add support for FHIR R5 integer64 type
* [FHIR-26554](https://jira.hl7.org/browse/FHIR-26554): Support date and time extractors in FHIRPath
* [FHIR-26610](https://jira.hl7.org/browse/FHIR-26610): Typo at description of toQuantity
* [FHIR-27033](https://jira.hl7.org/browse/FHIR-27033): Singleton Evaluation of Collections rules are not clear
* [FHIR-27764](https://jira.hl7.org/browse/FHIR-27764): Clarify return value of aggregate
* [FHIR-27859](https://jira.hl7.org/browse/FHIR-27859): Clarify format of UCUM argument toQuantity
* [FHIR-27890](https://jira.hl7.org/browse/FHIR-27890): Correct example for "union" function
* [FHIR-28144](https://jira.hl7.org/browse/FHIR-28144): Inconsistent result between compare and equal for un-comparable quantities
* [FHIR-28449](https://jira.hl7.org/browse/FHIR-28449): Please correct iff() earlier in the document to iif() and link the reference to the definition
* [FHIR-28927](https://jira.hl7.org/browse/FHIR-28927): Define code system for calendar duration units
* [FHIR-31018](https://jira.hl7.org/browse/FHIR-31018): Example errors?
* [FHIR-31551](https://jira.hl7.org/browse/FHIR-31551): Add EOF marker to grammar
* [FHIR-32113](https://jira.hl7.org/browse/FHIR-32113): small spelling mistake in endsWith (endsWith)
* [FHIR-34208](https://jira.hl7.org/browse/FHIR-34208): typo on day range in 1.4. Conventions
* [FHIR-36091](https://jira.hl7.org/browse/FHIR-36091): 'is' examples suck
* [FHIR-36257](https://jira.hl7.org/browse/FHIR-36257): Clarify expected behavior of matches and replaceMatches
* [FHIR-36271](https://jira.hl7.org/browse/FHIR-36271): Correct replaceMatches example
* [FHIR-36335](https://jira.hl7.org/browse/FHIR-36335): Ensure all function behavior is defined for input collections with more than one item
* [FHIR-36494](https://jira.hl7.org/browse/FHIR-36494): Clarify scope of collection passed to union()
* [FHIR-36588](https://jira.hl7.org/browse/FHIR-36588): Clarify that output of repeat() should be unique items
* [FHIR-37423](https://jira.hl7.org/browse/FHIR-37423): Typo, iff should be iif
* [FHIR-41033](https://jira.hl7.org/browse/FHIR-41033): Updated split functionality edge case A,,C
* [FHIR-41160](https://jira.hl7.org/browse/FHIR-41160): Clarify case of output content produced by encode('hex')
* [FHIR-41382](https://jira.hl7.org/browse/FHIR-41382): Introduce a function to "stash" variables into scope for use further down the expression
* [FHIR-44701](https://jira.hl7.org/browse/FHIR-44701): First batch of sample fhirpath expressions uses a `-` for a comment where should be a `//`
* [FHIR-27757](https://jira.hl7.org/browse/FHIR-27757): More string manipulation functions


#### Issues Released in 3.0.0
*(resolved during ballot reconciliation)*

* [FHIR-49237](https://jira.hl7.org/browse/FHIR-49237):  toString if type is not applicable, then result should be empty, not false
* [FHIR-28122](https://jira.hl7.org/browse/FHIR-28122):  Conversion table misses bool->quantity
* [FHIR-49172](https://jira.hl7.org/browse/FHIR-49172):  Clarify that the "C" referred to is the C programming language
* [FHIR-49521](https://jira.hl7.org/browse/FHIR-49521):  Clarify the usage of regex match groups by referencing the example
* [FHIR-49527](https://jira.hl7.org/browse/FHIR-49527):  Add return types to section titles in 6.4
* [FHIR-49236](https://jira.hl7.org/browse/FHIR-49236):  update all uses of true and false where referring to the boolean type should be tagged
* [FHIR-25194](https://jira.hl7.org/browse/FHIR-25194):  FHIRPath test xml is not valid xml
* [FHIR-27055](https://jira.hl7.org/browse/FHIR-27055):  Behavior for comparing Date, Time, and DateTime primitives is unclear - highlight through examples
* [FHIR-28242](https://jira.hl7.org/browse/FHIR-28242):  Test cases are inconsistent with fhir-test-cases
* [FHIR-43294](https://jira.hl7.org/browse/FHIR-43294):  Add a SQL COALESCE-like function to FhirPath
* [FHIR-49154](https://jira.hl7.org/browse/FHIR-49154):  Remove the test source files from the specification and refer directly to the github test repo
* [FHIR-49155](https://jira.hl7.org/browse/FHIR-49155):  Include an Append function that does not effect the ordering of the content via preserveOrder parameter
* [FHIR-49171](https://jira.hl7.org/browse/FHIR-49171):  Include a worked example for combine to contrast difference to union and |
* [FHIR-49530](https://jira.hl7.org/browse/FHIR-49530):  Date/time arithmetic - subtraction below 0 (and also addition over 24 hours)
* [FHIR-44939](https://jira.hl7.org/browse/FHIR-44939):  Correct description of power()
* [FHIR-49170](https://jira.hl7.org/browse/FHIR-49170):  Function Invocations to indicate that some functions only work on a collection with a single element
* [FHIR-49522](https://jira.hl7.org/browse/FHIR-49522):  Length() - number of characters in the input string
* [FHIR-36708](https://jira.hl7.org/browse/FHIR-36708):  Add repeatAll function
* [FHIR-40557](https://jira.hl7.org/browse/FHIR-40557):  Clarify remark about unit resulting from multiplication of Quantities
* [FHIR-41343](https://jira.hl7.org/browse/FHIR-41343):  Return empty or error when doing math with "special" UCUM units
* [FHIR-49227](https://jira.hl7.org/browse/FHIR-49227):  LastIndexOf with an empty string should return the length of the string
* [FHIR-49235](https://jira.hl7.org/browse/FHIR-49235):  Clarify the regex to be a part of the bullet list
* [FHIR-49117](https://jira.hl7.org/browse/FHIR-49117):  Remove incorrect comment "// example is wrong"
* [FHIR-49124](https://jira.hl7.org/browse/FHIR-49124):  typos in encode function parameter descriptions
* [FHIR-49139](https://jira.hl7.org/browse/FHIR-49139):  typos \<= (from markdown conversion - backslash delimiter for < no longer required)
* [FHIR-51613](https://jira.hl7.org/browse/FHIR-51613):  convertsToString() wrong return time documented
* [FHIR-51832](https://jira.hl7.org/browse/FHIR-51832):  optional parameters are marked with square brackets
* [FHIR-19896](https://jira.hl7.org/browse/FHIR-19896):  Allow two dates to be subtracted
* [FHIR-42969](https://jira.hl7.org/browse/FHIR-42969):  Create a way to specify case-insensitive/case-sensitive search, and clarify the default (with existing regex functions)
* [FHIR-51432](https://jira.hl7.org/browse/FHIR-51432):  Correct examples for <= and >= (and add more examples)
* [FHIR-52019](https://jira.hl7.org/browse/FHIR-52019):  fhirpath grammar for long to require L suffix (not optional)
* [FHIR-49513](https://jira.hl7.org/browse/FHIR-49513):  Wording correction
* [FHIR-49520](https://jira.hl7.org/browse/FHIR-49520):  Include an additional ReplaceMatches example
* [FHIR-52023](https://jira.hl7.org/browse/FHIR-52023):  hourOf, minuteOf, secondOf, millisecondOf should not apply to date
* [FHIR-52050](https://jira.hl7.org/browse/FHIR-52050):  Union function signature missing return type
* [FHIR-49512](https://jira.hl7.org/browse/FHIR-49512):  Add sort functionality
* [FHIR-49511](https://jira.hl7.org/browse/FHIR-49511):  Escaped backticks in paths permitted via escaping
* [FHIR-49519](https://jira.hl7.org/browse/FHIR-49519):  Table in 5.5.2.1 is confusing
* [FHIR-51576](https://jira.hl7.org/browse/FHIR-51576):  Quantity should preserve the units that were input
* [FHIR-49156](https://jira.hl7.org/browse/FHIR-49156):  Refine how the toDate operation is evaluated on a datetime
* [FHIR-49515](https://jira.hl7.org/browse/FHIR-49515):  What is System.Long and where is it defined?
* [FHIR-53131](https://jira.hl7.org/browse/FHIR-53131):  toString representations table needs improvement, And made all the use of the custom output formatter styling consistent
* [FHIR-46816](https://jira.hl7.org/browse/FHIR-46816):  Correct examples for lowBoundary and highBoundary
* [FHIR-49137](https://jira.hl7.org/browse/FHIR-49137):  timezoneOffsetOf return value units clarifies as hours
* [FHIR-49442](https://jira.hl7.org/browse/FHIR-49442):  Change log must be added (remove history toolbar - already in publishing box)
* [FHIR-49445](https://jira.hl7.org/browse/FHIR-49445):  add list of authors/contributors, organization to home pages
* [FHIR-49518](https://jira.hl7.org/browse/FHIR-49518):  Clarify iif function delayed argument evaluation
* [FHIR-28173](https://jira.hl7.org/browse/FHIR-28173):  Add sum(), min(), max(), count(), avg()
* [FHIR-49526](https://jira.hl7.org/browse/FHIR-49526):  Math section input requirements unclear and should include Quantity
* [FHIR-49529](https://jira.hl7.org/browse/FHIR-49529):  Date time arithmetic with fractions - additional guidance
* [FHIR-53076](https://jira.hl7.org/browse/FHIR-53076):  Equality semantics for collections
* [FHIR-53159](https://jira.hl7.org/browse/FHIR-53159):  What should rounding to for negative numbers?
* [FHIR-44774](https://jira.hl7.org/browse/FHIR-44774):  Better describe "iteration context" and impact on input collections/function execution - Scoped Functions
* [FHIR-53196](https://jira.hl7.org/browse/FHIR-53196):  Update grammar to split NUMERIC into DECIMAL | INTEGER
* [FHIR-33044](https://jira.hl7.org/browse/FHIR-33044):  Support constructor syntax
* [FHIR-49112](https://jira.hl7.org/browse/FHIR-49112):  Version and Changes pages added to menu
* [FHIR-49533](https://jira.hl7.org/browse/FHIR-49533):  Appendix A, B and section 17-19 moved to separate pages
* [FHIR-38007](https://jira.hl7.org/browse/FHIR-38007):  Better documentation for 'implies'
* [FHIR-54718](https://jira.hl7.org/browse/FHIR-54718):  Grammar delimiter for " (double quotes) is missing for the "fragment" rule
* [FHIR-34315](https://jira.hl7.org/browse/FHIR-34315):  Partial Date arithmetic for adding weeks
* [FHIR-53656](https://jira.hl7.org/browse/FHIR-53656):  Add polarity operator definition (unary operators +/-)
* [FHIR-53957](https://jira.hl7.org/browse/FHIR-53957):  Migrate the FHIR function `comparable` to fhirpath
* [FHIR-52957](https://jira.hl7.org/browse/FHIR-52957):  Removed inconsistencies in equality/equivalence of calendar/definite durations
* [FHIR-53883](https://jira.hl7.org/browse/FHIR-53883):  Mixed use of the term element and property to refer to child nodes of an object
* [FHIR-49523](https://jira.hl7.org/browse/FHIR-49523):  Support encoding a string as ascii (replacing chars over code 127 with '?')
* [FHIR-53554](https://jira.hl7.org/browse/FHIR-53554):  Clarify Unicode support
* [FHIR-53660](https://jira.hl7.org/browse/FHIR-53660):  Clarify default behavior when accessing non existent data elements
* [FHIR-49516](https://jira.hl7.org/browse/FHIR-49516):  Inconsistent use of empty and error with quantity processing
* [FHIR-45314](https://jira.hl7.org/browse/FHIR-45314):  Add a new function `pathname()` that returns the location(s) of the focused item(s)
* [FHIR-54485](https://jira.hl7.org/browse/FHIR-54485):  toDate and toDateTime consider the fhir mapping language formatting codes for string parsing
* [FHIR-49531](https://jira.hl7.org/browse/FHIR-49531):  Incorrect example in reflection - significant simplification of functionality, removed elements properties.
* [FHIR-55962](https://jira.hl7.org/browse/FHIR-55962):  Difference and duration functions precision parameter should be string not identifier
* [FHIR-55911](https://jira.hl7.org/browse/FHIR-55911):  Missing what to do when equality returns empty for functions like distinct
* [FHIR-44601](https://jira.hl7.org/browse/FHIR-44601):  clarifying behaviour of iif
* [FHIR-55931](https://jira.hl7.org/browse/FHIR-55931):  Matches and matchesFull should report error for invalid flags parameters
* [FHIR-55946](https://jira.hl7.org/browse/FHIR-55946):  Extract Date/DateTime/Time components need to indicate what to do when invalid input type
* [FHIR-55893](https://jira.hl7.org/browse/FHIR-55893):  resolve() function referenced, but not defined.
* [FHIR-55935](https://jira.hl7.org/browse/FHIR-55935):  decode() and invalid Unicode
* [FHIR-55471](https://jira.hl7.org/browse/FHIR-55471):  Arithmetic and toQuantity are not NOT explicitly define for Long
* [FHIR-55473](https://jira.hl7.org/browse/FHIR-55473):  FHIRPath String Equivalence Ambiguities
* [FHIR-56286](https://jira.hl7.org/browse/FHIR-56286):  Add an example to clarify result of ''.substring(0)
* [FHIR-49528](https://jira.hl7.org/browse/FHIR-49528):  Date time arithmetic with UCUM
* [FHIR-56190](https://jira.hl7.org/browse/FHIR-56190):  Extensive clarifications of handling quantities, and specifically unit conversions and calendar/time valued quantities
* [FHIR-49525](https://jira.hl7.org/browse/FHIR-49525):  Quantity math and unit conversion
* [FHIR-53249](https://jira.hl7.org/browse/FHIR-53249):  Quantity equivalence is ambiguous
* [FHIR-53250](https://jira.hl7.org/browse/FHIR-53250):  Calendar duration equivalence has conflicts
* [FHIR-56938](https://jira.hl7.org/browse/FHIR-56938):  Clarify relationship of calendar durations code system
* [FHIR-57038](https://jira.hl7.org/browse/FHIR-57038):  Add Long support for equality and equivalence
* [FHIR-57251](https://jira.hl7.org/browse/FHIR-57251):  Definition of repeatAll incorrectly describes evaluation
* [FHIR-57256](https://jira.hl7.org/browse/FHIR-57256):  Clarify behavior of Date addition

#### Issues Included since 3.0.0 (targetting 3.1)
* [FHIR-57822](https://jira.hl7.org/browse/FHIR-57822):  Typo in page title
* [FHIR-58739](https://jira.hl7.org/browse/FHIR-58739):  Include support for anonymous types in the instance selector
* [FHIR-49510](https://jira.hl7.org/browse/FHIR-49510):  Conformance language is not consistently used
* [FHIR-59626](https://jira.hl7.org/browse/FHIR-59626):  Quantity equivalence returns false rather than empty for invalid or incomparable units
* [FHIR-59620](https://jira.hl7.org/browse/FHIR-59620):  Clarify Quantity and Integer support for FHIRPath boundary functions
