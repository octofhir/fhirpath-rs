# Grahame's Answers on Explicit $this

## Questions and Answers

### Q1: Can I access $this on top level - i.e. is this valid expression `$this.name.given`?

**Grahame Grieve:** Yes

### Q2: Are these expressions equivalent: `name.where(use = 'official')` and `name.where($this.use = 'official')`?

**Grahame Grieve:** Yes

### Q3: For the Brian's example `'123456789'.substring(length()-4)` is that expression does the "right" thing `'123456789'.select($this.substring($this.length()-4))`?

**Grahame Grieve:** Maybe? That does something else. The right thing would be `'123456789'.substring('123456789'.length()-4)`

### Q4: In general can I think about `Patient.name.where(use='official').given` as `$this.Patient.name.where($this.use = 'official').given`?

**Grahame Grieve:** No. The type name like "Patient." is only allowed at the entry / root of the expression

### Q5: What if 'Patient' is a property of the object like `{Patient: { ....}}` - should I determine this during "semantic analysis"?

**Grahame Grieve:** That's a tricky point. In my library, if we're at the entry, and the first character is uppercase, it's automatically treated as a className, and I agree that's broken if you use my library on a class model with capitals for field names. (and I should probably fix it to look at properties first)

### Q6: I'm thinking about better AST for new fhirpath parser to be honest. And my guess that probably all chains like `Patient.name` or `name.given` have at the root "implicit" `$this` i.e. `$this.Patient.name` or `$this.name.given`. Is this right assumption?

**Grahame Grieve:** I think so

---

**Nikolai Ryzhikov 🐬:** Thank you!