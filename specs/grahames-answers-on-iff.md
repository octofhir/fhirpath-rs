# Grahame's Answers on iif() Function

## Conversation about chaining iif() and context behavior

**Nikolai Ryzhikov 🐬:** Can we chain iif from left aka `Patient.name.iif(...)`?

**Nikolai Ryzhikov 🐬:** Same question about `today()` and `defineVariable(...)`

**Grahame Grieve:** how else would it work?

**Nikolai Ryzhikov 🐬:** I don't know :) Just asking - because i want my parser fails on `x.iif()` or `x.today()`

**Grahame Grieve:** `x.iif()` should succeed

**Grahame Grieve:** I'm ambivalent about `x.today()`

**Nikolai Ryzhikov 🐬:** How `x.iif()` changes behavior of iif? redefine `$this` inside iif into `x`?

**Grahame Grieve:** I don't know that and I was just wondering about it, in fact

**Grahame Grieve:** I hope the answer is no

**Nikolai Ryzhikov 🐬:** Why do we need `x.iif` at all in that case?

**Grahame Grieve:** for branching, like always?

**Nikolai Ryzhikov 🐬:** What could be the "real" example of chained iif?

**Nikolai Ryzhikov 🐬:** All examples I saw was "standalone" iif

**Grahame Grieve:** `Patient.name.iif(text.exists(), text, family+given)`

**Nikolai Ryzhikov 🐬:** Oh that means that in branches `$this` is from focus? ie branches context is `Patient.name`?

**Grahame Grieve:** I don't know the answer to that, and wondered

**Grahame Grieve:** but I wanted to explain a use case

**Nikolai Ryzhikov 🐬:** I mean branches behaves like select or where

**Nikolai Ryzhikov 🐬:** Why condition expression (`iif(condition, ..,..)`) is not in the same context as branches?

**Nikolai Ryzhikov 🐬:** or you mean text is from name not from resource

**Nikolai Ryzhikov 🐬:** got it - that means chained iif get/set context (`$this`) from left expression

**Grahame Grieve:** oh, we have a test case:

```xml
<test name="testIif11" inputfile="patient-example.xml">
    <expression>('context').iif($this = 'context','true-result', 'false-result')</expression>
    <output type="string">true-result</output>
</test>
```

**Grahame Grieve:** oh: The function iif can only be used on a singleton value but found 3

**Grahame Grieve:** that makes it trickier to use

**Grahame Grieve:** so this patient:

```xml
<?xml version="1.0" encoding="UTF-8"?>
<Patient xmlns="http://hl7.org/fhir">
    <name>
        <use value="official"/>
        <text value="Pater J Chalmers"/>
        <family value="Chalmers"/>
        <given value="Peter"/>
        <given value="James"/>
    </name>
    <name>
        <use value="usual"/>
        <given value="Jim"/>
    </name>
</Patient>
```

And this FHIRPath:
```
Patient.name.first().iif(text.exists(), text, family+given.first())
```

gives a single string `Pater J Chalmers`

**Nikolai Ryzhikov 🐬:** Should it fail if not on singleton?