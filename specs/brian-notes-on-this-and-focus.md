# Understanding FHIRPath's $this and focus

Proposed wording and background/discussion for FHIRPath issues: FHIR-44601 and FHIR-44774

How $this and the focus/context of function parameter evaluation can be confusing.
The documentation isn't as clear as it could be, and different engines handle them differently. This is a quick summary of how fhirpath intends it to work.

Note: The HAPI engine has been updated to implement the focus and $this correctly according to the assumptions in this post.
https://github.com/hapifhir/org.hl7.fhir.core/commit/12cf45470bb12f85f5b987753756f2b08b65bf8b

I've done a walkthrough demonstration of the functionality on YouTube to try and help guide the discussion: https://youtu.be/sQgQJol84dY

What is well understood from the specification (and clear):
$this (and $index) is set when entering a function that has as expression type parameter.
https://build.fhir.org/ig/HL7/FHIRPath/branches/BP-2025-ballot-recon/index.html#functions
Specifically where, select, exists, all, repeat and aggregate
iif, trace and defineVariable are also in this category, however this is probably not intended…
The scope of the $this variable is only within the arguments of the function, and is not available outside the function (in a fluent sense). The previous definition of $this will then be available again.
The fluent nature of chaining functions/properties flows the focus through the expression
What is less clear:
the focus is the input collection to the function, and is the evaluation context of the function, and its arguments (not actually documented).
$this is set at the beginning of evaluating an expression, and is also the %context to begin with.
Parameters to functions can be expressions themselves, even though the type isn't marked as expression - text is there, but the presence of the :expression indicator confuses that others don't notice the subtle text.
Functions with more than 1 parameter (including an expression type):
Which ones are re-evaluated? aggregate does init once, then aggregator for each iteration. iif has 3 parameters, no clarity on when to re-evaluated each time? (though is an error to call with a collection, so kinda mute - and messy that it touches $this at all), trace has 2 arguments, …
What is not explicit/clear at all:
What the context of argument execution? (is it $this or focus? - not documented, but implied focus)
What is the relationship between $this and focus, particularly in the context of functions that have an expression type parameter.
How to handle $this and focus in the context of functions that don't have an expression type parameter, but are set based functions (e.g. union, intersect, except, unionAll, etc.)
When evaluating arguments for a function that has any arguments with the :expression indicator, all arguments are evaluated for each iteration of the focus, the results then aggregated (or whatever it's logic is).
does the focus remain the collection, or the item under iteration? Current engines have $this and focus the same here. (and I think should)
$this doesn't change apart from expression type functions.
Proposal summary:
Update the Functions introductory section to the proposed wording below
remove the : expression indicator to mark functions that set $this and $index and instead mark them as iterative functions and refer them to the new section (below) that describes this behaviour, and enumerate the functions there too.
Specifically: where, select, exists, all, repeat and aggregate
Update the parameters to the functions exists, all, where and iif to be boolean (instead of type expression) - the iterative function is the new indicator, and these expressions need to return a boolean value
Update the "projection" parameter of the functionsselect, repeat, trace, aggregate and defineVariable to be collection (or something else consistent), would there be any benefit to these all having the same name? I don't like using expression as that confuses things to make it feel more like the other parameters aren't expressions too. Maybe no type declared?
remove the iterative function marker from iif, trace and defineVariable, as they are not iterators, and should't set $this or $index. The trace/defineVariable are just indicating that it's an expression that returns anything, not some specific type (just as select does) The iif function is one that could need to refer to $this, not to set it.
introduce the concept of $focus and $this as two different variables, where $focus is the current focus (or item in an iterative function), as per the tracker on iif
https://jira.hl7.org/browse/FHIR-44601
Note: Should the aggregate function be declared as a generic type?
aggregate<T>(aggregator: T, init: T): T (and the variable $total will be of type T)
That would then be something that could be statically tested too. This could also be used in select, although that returns a collection of T?

Proposed specification wording updates:
5. Functions
Functions are distinguished from path navigation names by the fact that they are followed by () with zero or more arguments. Throughout this specification, the word parameter is used to refer to the definition of a parameter as part of the function definition, while the word argument is used to refer to the values passed as part of a function invocation. With a few minor exceptions (e.g. current date and time functions), functions in FHIRPath operate on a collection of values (referred to as the input collection) and produce another collection as output (referred to as the output collection).
However, for many functions, passing an input collection with more than one item is defined as an error condition. Each function defines its behavior for input collections of any cardinality (0, 1, or many).
This approach allows functions to be chained, successively operating on the results of the previous function in order to produce the desired final result.

The following sections describe the functions supported in FHIRPath, detailing the expected types of parameters and type of collection returned by the function:

Although the function parameters are defined with a specific type, they can be fhirpath expressions that will return the specific type (where a type is defined)
If a function expects the argument passed to a parameter to be a single value (e.g. startsWith(prefix: String)) and it is passed an argument that evaluates to a collection with multiple items, or to a collection with an item that is not of the required type (or cannot be converted to the required type), the evaluation of the expression will end and an error will be signaled to the calling environment.
Square bracket notation [] is used in function signatures to indicate optional parameters. (e.g. toQuantity([unit : String]) : Quantity)
If a parameter doesn't require a specific type, and supports collections, then the parameter type will be defined as collection
Note that although all functions return collections, if a given function is defined to return a single element, the return type is simplified to just the type of the single element, rather than the list type.

Iterative functions
Some functions are marked as iterative functions. This type of function is evaluated for each item on the input collection separately, re-evaluating all parameters on each iteration when required (i.e. the argument's expression uses variables).
During the evaluation of each item the $focus, $this and $index variables are set to the current item being processed, and it's index.

The scope of the $this and $index variables is within the parameters of the iterative function. If nesting iterative functions, the scope of the variable is restored to the outer scope.

These are the fhirpath defined iterative functions:
where, select, exists, all, repeat and aggregate

For example:

// Retrieve a list of formatted names for the patient
// (with no unwanted padding white-space)
Patient.name.select(given.join(' ').combine($this.family, true).join(', '))

// Observation values that are outside a specific range
Observation.value.where($this < 90 or $this > 110)

// textual list of required participants in an appointment
Appointment.participant.select(required.iff($focus, $this.actor.display + ' (required)'))
Special variables
Variable	Description
$focus	(STU) The current input collection (or in iterative functions, the currently processing item in the input collection)
(Previous versions of fhirpath could simulate the $focus value by using select($this))
Mostly useful where an argument to a function needs to use the input collection, most commonly the iif function.
It is similar to self, this or me in other programming languages.
$this	Set at the beginning of execution of an expression as the initial context (See %context below)
Re-set to the current item being processed in iterative functions.
Refer to the iterative functions section for additional details
$index	Set at the beginning of execution of an expression to 0
Re-set to the index of the current item being processed in iterative functions.
Refer to the iterative functions section for additional details
$total	Only available inside the parameters of the aggregate function. Holds the running total during processing, and at the end will be the result returned by the function.
%resource	The current resource being processed (that contains the property in $focus)
When passing through resolve() or into a contained resource will be changed to the new resource context.
%context	The entry/starting point for execution of the fhirpath expression.
Often used in fhirpath invariants.
(Does not change during execution)
%rootResource	The top level fhir resource. Usually a Bundle, or resource that has contained resources (or Parameters resource).
Though processing on regular fhir resources this is also the same as %resource.
(Does not change during execution)
Cross reference functions and their argument context across implementations
Function	Firely	HAPI <=27	HAPI 28+	FhirPath.js	Python
paramList (used)
quantity