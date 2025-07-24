# Discussion on FHIRPath Scoping Rules

## The Question

**Brian Postlethwaite:** What should this return?
(evaluated in the context of the patient resource)

```fhirpath
'123456789'.substring(length()-4)
```

## Initial Responses

**Grahame Grieve:** 6789 surely?

**Nikolai Ryzhikov 🐬:** It depends on result of this expression:
```fhirpath
Patient.telecom.where(use = 'official').union(Patient.contact.telecom.where(use = 'official')).exists().not()
```

**Nikolai Ryzhikov 🐬:** 
- `'123456789'.select(substring(length()-4))` ?
- or `'123456789'.substring($this.length()-4)`

**Nikolai Ryzhikov 🐬:** I would love if all methods will have the same scoping (focus) rules. Please, do not make rules like - 'if starts with Patient it is root otherwise current focus ($this)'

**Grahame Grieve:** Given that the spec is normative, and widely implemented, it's too late to have discussions like that. But we did, back in the day, and it's all clearly documented.

**Nikolai Ryzhikov 🐬:** What's documented? Different scoping rules? or if start with Patient?

**Grahame Grieve:** Scoping and resolution.

## The Real Question Emerges

**Brian Postlethwaite:** p.s. It's not clearly documented, and that was the reason for the question. And also multiple engines are returning different results.

**Grahame Grieve:** Is it the scope question, or about substring()?

**Gino Canessa:** I assumed it was about substring. Since I typically expect zero-index strings, I would default assume:

- `'123456789'` length = 9
- `'123456789'.substring(5)` yields `'6789'`

That said, I believe FHIRPath uses one-index strings, which would match Grahame's answer. I have no feedback on the scope question (if that is in scope).

**Brian Postlethwaite:** The real question here is what was the input collection to the length function.

**Grahame Grieve:** The scope of length() is the scope that '123456789' is in.

**Grahame Grieve:** So... I was wrong.

**Gino Canessa:** In fairness, I think we were all wrong 😅

**Chris Moesel:** I wasn't wrong because I didn't say anything. Just another time where I did myself (and everyone else) a favor by keeping my mouth shut. 😉

**Brian Postlethwaite:** So what's your answer then? (Chris)

**Grahame Grieve:** What's the context?

**Brian Postlethwaite:** You've given your answer Grahame 😉

**Chris Moesel:** Oh shoot. I thought you guys already decided. I guess I should have kept my mouth shut longer! Without looking at the spec, I would have guessed that length() was scoped to 123456789. But now I have to look at the spec (which apparently isn't clear on it, but I'll look anyway).

**Brian Postlethwaite:** That's what we're looking for, what does the spec say (or your gut - if you decide the spec isn't explicit) it should do.

**Grahame Grieve:** I don't see where we say. How can that be possible?

**Bryn Rhodes:** Well, I think we do say, but implicitly.

**Chris Moesel:** OK. Well, to be clear, my gut was:
```fhirpath
'123456789'.substring(length()-4) 
--> '123456789'.substring(9-4) 
--> '123456789'.substring(5) 
--> '56789'
```

## Spec Documentation Issues

**Grahame Grieve:** We say somewhere that the parameters that take expressions are different because of context, but I can't find that.

**Nikolai Ryzhikov 🐬:** My first impression was that only where and select as high order functions are scoped by focus and the rest methods by context.

**Brian Postlethwaite:** It's not there, that's why.

**Bryn Rhodes:** We say in functions that when a parameter is typed as an expression, it's an iterator and arguments are evaluated in that context and have access to $this.

**Grahame Grieve:** Where do we say that?

**Bryn Rhodes:** https://hl7.org/fhirpath/#functions

**Brian Postlethwaite:** The 2 parts of the spec are here:
- https://hl7.org/fhirpath/2025Jan/index.html#functions
- https://hl7.org/fhirpath/2025Jan/index.html#function-invocations

That text is there, but that text says nothing about what happens when there isn't an expression parameter (which is this sample test case).

**Bryn Rhodes:** The implication is that only iterative functions do that, but we're not explicit about that.

**Grahame Grieve:** I don't see where it actually address the question at hand.

**Bryn Rhodes:** 
> If the function takes an expression as a parameter, the function will evaluate the expression passed for the parameter with respect to each of the items in the input collection. These expressions may refer to the special $this and $index elements, which represent the item from the input collection currently under evaluation, and its index in the collection, respectively.

**Grahame Grieve:** And where does it say that the context has changed?

**Bryn Rhodes:** 
> For example, in name.given.where($this > 'ba' and $this < 'bc') the where() function will iterate over each item in the input collection (elements named given) and $this will be set to each item when the expression passed to where() is evaluated.

**Grahame Grieve:** Still doesn't say it. Just says what $this is.

**Brian Postlethwaite:** My interpretation of the context changing came from this sentence in the functions section:

> Correspondingly, arguments to the functions can be any FHIRPath expression, though functions taking a single item as input require these expressions to evaluate to a collection containing a single item of a specific type. This approach allows functions to be chained, successively operating on the results of the previous function in order to produce the desired final result.

"successively operating on the results of the previous function"
(which precedes Bryn's section on expression parameter, refining a special case for $this)

**Grahame Grieve:** That's obscure.

## Technical Clarifications

**Chris Moesel:** There are all sorts of tricky things here if you don't know the spec well (and in some cases even if you do):

1. **Is indexing 0-based or 1-based?** Like Gino, I originally thought 0-based, but then second-guessed myself and seemed to remember it being 1-based. But... Answer: the spec clearly says it is 0-based (unless the model indicates otherwise)!

2. **Does length() take on the same scope as the function whose body calls it or a higher-level scope?** Answer: (still figuring it out, I guess)

3. **Is length() counting the number of characters in the string (9 characters) or the length of the collection (1 string)?** Answer: Trick question. length() only counts characters in a string; use count() to count items in a collection.

**Bryn Rhodes:** Agreed can definitely be more explicit.

**Chris Moesel:** @Bryn Rhodes - what used 1-based indices? I swear there was _something_. Am I thinking of the QDM model in CQL?

**Brian Postlethwaite:** (the 0 based indexing and count vs length were accidental here, but also makes it a good test case for learning fhirpath - once we all agree and it's well documented)

**Grahame Grieve:** The index operator is 0-based unless the underlying model says different, and v2 does.

**Bryn Rhodes:** Right it's V2 that's one-based.

**Grahame Grieve:** But that doesn't affect strings.

## Test Cases and Implementation Issues

**Grahame Grieve:** Anyway, @Nikolai Ryzhikov 🐬 now we know the answer to your question above: read the test cases.

**Brian Postlethwaite:** Test cases in the tests project hadn't covered this case till the last few weeks when I've added some in, and depending on the interpretation of this answer, might be wrong.
(The engines returning different results has demonstrated this too)

**Grahame Grieve:** But they must. Take this:
```fhirpath
defineVariable('n1', name.first()).where(active.not()) | defineVariable('n1', name.skip(1).first()).select(%n1.given)
```
That can only be evaluated correctly if the scope of name and active is correct.

**Brian Postlethwaite:** All the arguments to function in the test cases (before I recently added a couple) were static values like constants which then don't touch things.

**Brian Postlethwaite:** In this case, those are all expression based functions, so not relevant to the test case here.

**Grahame Grieve:** Well, ok. They are all expression based except for this test case.

**Grahame Grieve:** I baffled how we can be asking this 10 years in.

**Brian Postlethwaite:** Yup, and where the confusion comes in.

**Brian Postlethwaite:** Grahame Grieve said: "I baffled how we can be asking this 10 years in"

Indeed. It's been that unclear, and no-ones chased it down till now. And not noticed that the engines are not consistent on this edge case.

**Grahame Grieve:** I think the bit that baffles me is that it's an edge case.

## Impact Analysis and Future Work

**Brian Postlethwaite:** p.s. I'm doing analysis across all the IGs in search parameters and invariants to try and find cases where it comes up, and thus impacted by different engines. Though I think the place it's likely to be hit is going to be in questionnaires where there is more manipulation going on.

**Nikolai Ryzhikov 🐬:** Maybe scoping rules deserves it's own section with examples in spec? Somewhere in functions introduction?

**Brian Postlethwaite:** I'm working on a new section for it yes.