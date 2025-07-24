# Grahame's Answers on Scope of Combine

## Initial Bug Discussion

**Grahame Grieve:** GF#17275 is actually a bug in my FHIRPath somewhere. Maybe in my head... 

```fhirpath
(concept.code | descendants().concept.code).isDistinct()
```

**Grahame Grieve:** because, of course (!), `|` is a set union, and the outcomes are guaranteed to be unique....

**Grahame Grieve:** so it should be 

```fhirpath
concept.code.combine(descendants().concept.code).isDistinct()
```

## Syntax Concerns

**Brian Postlethwaite:** Yes, one of my favourite dislikes ;)

**Brian Postlethwaite:** I still don't like the syntax on the combine operation. It just feels weird to me, the scope of the collection inside the call, isn't the thing to the left of the function, it is its parent. (`descendants()` isn't called on the items of code, its called on the items of concept)

**Bryn Rhodes:** It is a bit strange that it's not like other iterators in that regard (like `.select()` and `.where()`) and there's no visual cue in the syntax to say that.

## Comparing Syntax Patterns

**Brian Postlethwaite:** Are these the same thing (ignoring the isdistinct part)?

```fhirpath
Patient.name.select(given | family)
Patient.name.select(given.combine(family))
```

**Brian Postlethwaite:** the `.` operator isn't applying to the property

**Brian Postlethwaite:** I know its too late, wish I'd complained earlier, and been in the ballot pool - Thanks HL7Au

**Bryn Rhodes:** Yeah, those would be the same (minus the distinct), because the identifiers within the select will both resolve on the currently iterated name element.

**Brian Postlethwaite:** `Patient.name.select(given.combine($this.family))`

**Bryn Rhodes:** Yeah, that at least gives a visual cue

## Functions and $this Context

**Brian Postlethwaite:** Is it the only function where the `$this` isn't the thing on the left of the `.`?

**Brian Postlethwaite:** In this example the `$this` is name, and not given

**Bryn Rhodes:** No, `.subsetOf()`, `.supersetOf()`, `.intersect()`, `.exclude()`, `.union()` (synonym for `|`), and all of the singleton functions like `.indexOf()`, `.startsWith`, etc.

**Bryn Rhodes:** By number, functions that introduce an iterating context are actually the minority.

**Brian Postlethwaite:** Thanks Bryn, I'm going to have to go investigate how Ewout has done this then!

## Implementation Questions

**Paul Lynch:** Coincidentally, I am trying to implement subsetOf today. @Bryn Rhodes Does "$this" really apply in subsetOf? The documentation under "5. Functions" says, "If the function takes an expression as a parameter.... These expressions may refer to the special $this element...." The function "subsetOf" is not one of those listed as taking an "expression", but rather the documentation (at http://hl7.org/fhirpath/#existence) says it takes a "collection".

**Bryn Rhodes:** You're right, `$this` does not apply in a `.subsetOf()`; the list I provided above is operators where `$this` does not apply (because they're not "iterators").

**Paul Lynch:** @Brian Postlethwaite I don't think

```fhirpath
Patient.name.select(given.combine(family))
```

is correct syntax. The "select" function expects "projection: expression" as an argument, but you are passing it the result of "combine", which is a collection.

**Bryn Rhodes:** The entire argument `given.combine(family)` is the "expression" argument.

**Bryn Rhodes:** So that expression is evaluated for every member of the Patient.name collection.

**Paul Lynch:** Okay, that makes sense. Thanks.

## Feature Request

**Brian Postlethwaite:** The whole focus vs `$this` discussion makes this thread quite interesting now...

But coming back here I was wondering if people had any thoughts on adding a parameter to isDistinct to log the duplicate values out?

e.g. `isDictinct([traceDuplicates: string])` and this would get the engine to call `trace('traceDuplicates', dupValue)` for each value that is a duplicate in the input collection.

@Bryn Rhodes

## Implementation Status

**Brian Postlethwaite:** Relevant to this discussion:

**Brian Postlethwaite:** #implementers > R6 eld-30

**Brian Postlethwaite:** And checking the invariants that were created in R4, they include use of `$this`, so no issue with that. And I believe that the static checks we now have pick these things up and are consistent.