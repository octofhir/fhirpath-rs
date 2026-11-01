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